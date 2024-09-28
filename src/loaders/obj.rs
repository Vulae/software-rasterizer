use std::{error::Error, io::Read};

use crate::{math::vector3::Vec3, mesh::Mesh};

pub fn load_mesh(mut file: impl Read) -> Result<Mesh, Box<dyn Error>> {
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<(usize, usize, usize)> = Vec::new();

    let mut str = String::new();
    file.read_to_string(&mut str)?;

    str.lines()
        .map(|line| {
            match &line
                .split(' ')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()[..]
            {
                ["v", x, y, z] | ["v", x, y, z, _] => {
                    vertices.push(Vec3::new(x.parse()?, y.parse()?, z.parse()?));
                }
                ["f", i @ ..] => {
                    let i = i
                        .iter()
                        .map(|i| {
                            // 'vertex_index', 'vertex_index/uv_index',
                            // 'vertex_index/uv_index/normal_index',
                            // 'vertex_index//normal_index'
                            Ok(i.split('/').next().unwrap().parse::<usize>()?)
                        })
                        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
                    if i.len() != 3 {
                        unimplemented!();
                    }
                    // Indices are 1-based
                    indices.push((i[0] - 1, i[1] - 1, i[2] - 1));
                }
                _ => {}
            }
            Ok(())
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    Ok(Mesh::new(0, vertices, None, indices))
}
