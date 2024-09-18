#![allow(unused)]

use crate::{
    mesh::Mesh,
    raycast::{Ray, RayIntersection},
    vector::Vec3,
};

#[derive(Debug)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
}

impl Scene {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
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
