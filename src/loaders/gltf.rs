// https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#glb-file-format-specification

#![allow(unused)]

use std::collections::HashMap;
use std::{error::Error, io::Read};

use serde::Deserialize;
use termion::terminal_size_pixels;

use crate::math::vector3::Vec3;
use crate::mesh::Mesh;
use crate::reader::Reader;
use crate::scene::Scene;

#[derive(Debug, Deserialize)]
struct JsonScene {
    name: String,
    nodes: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct JsonNode {
    name: String,
    mesh: usize,
}

#[derive(Debug, Deserialize)]
struct JsonMesh {
    name: String,
    primitives: Vec<JsonMeshPrimitive>,
}

#[derive(Debug, Deserialize)]
struct JsonMeshPrimitive {
    attributes: HashMap<String, usize>,
    indices: usize,
    material: Option<usize>,
    mode: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct JsonAccessor {
    bufferView: usize,
    componentType: usize,
    count: usize,
    // TODO: What are min and max???
    r#type: String,
    // TODO: Should I support this?
    sparse: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct JsonBufferView {
    buffer: usize,
    byteLength: usize,
    byteOffset: usize,
    target: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct JsonBuffer {
    byteLength: usize,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct JsonRoot {
    scene: usize,
    scenes: Vec<JsonScene>,
    nodes: Vec<JsonNode>,
    meshes: Vec<JsonMesh>,
    accessors: Vec<JsonAccessor>,
    bufferViews: Vec<JsonBufferView>,
    buffers: Vec<JsonBuffer>,
}

#[derive(Debug, Clone, Copy)]
enum AccessorComponent {
    Int(i64),
    Float(f32),
}

impl From<AccessorComponent> for f32 {
    fn from(val: AccessorComponent) -> Self {
        match val {
            AccessorComponent::Int(n) => n as f32,
            AccessorComponent::Float(n) => n,
        }
    }
}

// TODO: Please don't panic!
impl From<AccessorComponent> for usize {
    fn from(value: AccessorComponent) -> Self {
        match value {
            AccessorComponent::Int(n) => n as usize,
            AccessorComponent::Float(_) => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
enum AccessorValue {
    Scalar(AccessorComponent),
    Vector(Box<[AccessorComponent]>),
    Matrix(usize, Box<[AccessorComponent]>),
}

// TODO: Please don't panic!
impl From<AccessorValue> for usize {
    fn from(value: AccessorValue) -> Self {
        match value {
            AccessorValue::Scalar(num) => num.into(),
            _ => panic!(),
        }
    }
}

// TODO: Please don't panic!
impl From<AccessorValue> for Vec3 {
    fn from(value: AccessorValue) -> Self {
        match value {
            AccessorValue::Vector(vec) => {
                if vec.len() < 3 {
                    panic!();
                }
                Vec3::new(vec[0].into(), vec[1].into(), vec[2].into())
            }
            _ => panic!(),
        }
    }
}

impl JsonRoot {
    fn read_accessor(&self, buffers: &[Box<[u8]>], index: usize) -> Box<[AccessorValue]> {
        // TODO: I was lazy and didn't care about error handling.
        // FIXME: Doesn't follow: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#data-alignment
        let accessor = &self.accessors[index];
        let view = &self.bufferViews[accessor.bufferView];
        let buffer = &buffers[view.buffer];

        fn read_component(
            reader: &mut Reader<false, impl Read>,
            component_type: usize,
        ) -> AccessorComponent {
            match component_type {
                5120 => AccessorComponent::Int(reader.read_prim::<i8>().unwrap() as i64),
                5121 => AccessorComponent::Int(reader.read_prim::<u8>().unwrap() as i64),
                5122 => AccessorComponent::Int(reader.read_prim::<i16>().unwrap() as i64),
                5123 => AccessorComponent::Int(reader.read_prim::<u16>().unwrap() as i64),
                5125 => AccessorComponent::Int(reader.read_prim::<u32>().unwrap() as i64),
                5126 => AccessorComponent::Float(reader.read_prim::<f32>().unwrap()),
                _ => panic!(),
            }
        }

        fn read_value(
            reader: &mut Reader<false, impl Read>,
            value_type: &str,
            component_type: usize,
        ) -> AccessorValue {
            match value_type {
                "SCALAR" => AccessorValue::Scalar(read_component(reader, component_type)),
                "VEC2" => AccessorValue::Vector(
                    (0..2)
                        .map(|_| read_component(reader, component_type))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                "VEC3" => AccessorValue::Vector(
                    (0..3)
                        .map(|_| read_component(reader, component_type))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                "VEC4" => AccessorValue::Vector(
                    (0..4)
                        .map(|_| read_component(reader, component_type))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                "MAT2" => AccessorValue::Matrix(
                    2,
                    (0..4)
                        .map(|_| read_component(reader, component_type))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                "MAT3" => AccessorValue::Matrix(
                    3,
                    (0..9)
                        .map(|_| read_component(reader, component_type))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                "MAT4" => AccessorValue::Matrix(
                    4,
                    (0..16)
                        .map(|_| read_component(reader, component_type))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                _ => panic!(),
            }
        }

        let mut reader = Reader::new_le(std::io::Cursor::new(
            &buffer[view.byteOffset..(view.byteOffset + view.byteLength)],
        ));

        (0..accessor.count)
            .map(|_| read_value(&mut reader, &accessor.r#type, accessor.componentType))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

#[derive(Debug)]
enum Chunk {
    Json(JsonRoot),
    Bin(Box<[u8]>),
}

pub fn load_scene(file: impl Read) -> Result<Scene, Box<dyn Error>> {
    let mut reader = Reader::new_le(file);

    if &reader.read_prim::<[u8; 4]>()? != b"glTF" {
        panic!("File is not glTF file");
    }
    if reader.read_prim::<u32>()? != 2 {
        panic!("Unsupported glTF version");
    }
    let gltf_length = reader.read_prim::<u32>()?;
    let mut gltf_offset = 12;

    let mut chunks: Vec<Chunk> = Vec::new();
    while gltf_offset < gltf_length {
        let chunk_length = reader.read_prim::<u32>()?;
        let chunk_type = reader.read_prim::<[u8; 4]>()?;
        let chunk_data = reader.read_buf(chunk_length as usize)?;

        match &chunk_type {
            b"JSON" => chunks.push(Chunk::Json(serde_json::from_slice(&chunk_data).unwrap())),
            b"BIN\0" => chunks.push(Chunk::Bin(chunk_data.into_boxed_slice())),
            _ => {}
        }

        gltf_offset += 8 + chunk_length;
    }

    let mut json = None;
    let mut bin = None;

    chunks.into_iter().for_each(|c| match c {
        Chunk::Json(j) => json = Some(j),
        Chunk::Bin(b) => bin = Some(b),
    });

    let json = json.unwrap();
    let buffers = vec![bin.unwrap()];

    let mut scene = Scene::new();

    json.scenes[json.scene].nodes.iter().for_each(|node_index| {
        let node = &json.nodes[*node_index];
        // TODO: Temporary thing for testing
        if node.name.to_lowercase().contains("skybox") {
            return;
        }
        let mesh = &json.meshes[node.mesh];
        mesh.primitives.iter().for_each(|primitive| {
            let vertices: Vec<Vec3> = IntoIterator::into_iter(json
                .read_accessor(&buffers, *primitive.attributes.get("POSITION").unwrap()))
                .map(|v| v.clone().into()) // ???
                .collect();
            let indices: Vec<usize> = IntoIterator::into_iter(json
                .read_accessor(&buffers, primitive.indices))
                .map(|v| v.clone().into()) // ???
                .collect();
            let indices: Vec<(usize, usize, usize, termion::color::Rgb)> = indices
                .chunks(3)
                .map(|chunk| {
                    (
                        chunk[0],
                        chunk[1],
                        chunk[2],
                        termion::color::Rgb(255, 255, 255),
                    )
                })
                .collect();

            scene
                .meshes
                .push(Mesh::new_from_vertices_indices(&vertices, &indices));
        });
    });

    Ok(scene)
}
