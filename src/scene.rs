#![allow(unused)]

use crate::{
    material::Material,
    mesh::Mesh,
    raycast::{Ray, RayIntersection},
};

#[derive(Debug)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        self.meshes
            .iter()
            .flat_map(|mesh| mesh.triangles.iter())
            .filter_map(|triangle| triangle.intersect(ray))
            .reduce(|closest, intersection| {
                if intersection.distance < closest.distance {
                    intersection
                } else {
                    closest
                }
            })
    }
}
