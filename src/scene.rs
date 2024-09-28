#![allow(unused)]

use crate::{
    material::Material,
    mesh::Mesh,
    raycast::{Ray, RayIntersection},
};

#[derive(Debug)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Box<dyn Material>>,
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
            .filter_map(|mesh| mesh.intersect(ray))
            .reduce(|closest, intersection| {
                if intersection.distance < closest.distance {
                    intersection
                } else {
                    closest
                }
            })
    }
}
