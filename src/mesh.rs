#![allow(unused)]

use std::error::Error;

use crate::{
    math::vector3::Vec3,
    raycast::{Ray, RayIntersection},
    uv::Uv,
};

pub fn triangle_normal(v0: &Vec3, v1: &Vec3, v2: &Vec3) -> Vec3 {
    Vec3::cross(&(*v1 - *v0), &(*v2 - *v0)).normalized()
}

#[derive(Debug)]
pub struct Mesh {
    pub material_index: usize,
    pub position: Vec<Vec3>,
    pub texcoord: Vec<Uv>,
    pub indices: Vec<(usize, usize, usize)>,
}

impl Mesh {
    pub fn new(
        material_index: usize,
        position: Vec<Vec3>,
        texcoord: Option<Vec<Uv>>,
        indices: Vec<(usize, usize, usize)>,
    ) -> Self {
        Self {
            material_index,
            texcoord: texcoord
                .unwrap_or_else(|| position.iter().map(|_| Uv::new(0.0, 0.0)).collect()),
            position,
            indices,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        self.indices
            .iter()
            .filter_map(|(i0, i1, i2)| {
                // https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm#Rust_implementation
                let v0 = self.position[*i0];
                let v1 = self.position[*i1];
                let v2 = self.position[*i2];

                let e1 = v1 - v0;
                let e2 = v2 - v1;

                let ray_cross_e2 = Vec3::cross(&ray.direction, &e2);
                let det = Vec3::dot(&e1, &ray_cross_e2);

                if det > -f32::EPSILON && det < f32::EPSILON {
                    return None;
                }

                let inv_det = 1.0 / det;
                let s = ray.position - v0;
                let u = inv_det * Vec3::dot(&s, &ray_cross_e2);
                #[allow(clippy::manual_range_contains)]
                if u < 0.0 || u > 1.0 {
                    return None;
                }

                let s_cross_e1 = Vec3::cross(&s, &e1);
                let v = inv_det * Vec3::dot(&ray.direction, &s_cross_e1);
                if v < 0.0 || u + v > 1.0 {
                    return None;
                }

                let t = inv_det * Vec3::dot(&e2, &s_cross_e1);

                if t > f32::EPSILON {
                    Some(RayIntersection {
                        distance: t,
                        position: ray.position + ray.direction * t,
                        normal: triangle_normal(&v0, &v1, &v2),
                    })
                } else {
                    None
                }
            })
            .reduce(|closest, intersection| {
                if intersection.distance < closest.distance {
                    intersection
                } else {
                    closest
                }
            })
    }
}
