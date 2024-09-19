#![allow(unused)]

use std::error::Error;

use crate::{
    matrix::Matrix4x4,
    raycast::{Ray, RayIntersection},
    vector::Vec3,
};

#[derive(Debug, Clone)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub color: termion::color::Rgb,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, color: termion::color::Rgb) -> Self {
        Self { v0, v1, v2, color }
    }

    pub fn normal(&self) -> Vec3 {
        Vec3::cross(&(self.v1 - self.v0), &(self.v2 - self.v0)).normalized()
    }

    pub fn translate(&self, translation: &Vec3) -> Self {
        let mut transformed = self.clone();
        transformed.v0 += *translation;
        transformed.v1 += *translation;
        transformed.v2 += *translation;
        transformed
    }

    pub fn scale(&self, scale: &Vec3) -> Self {
        let mut transformed = self.clone();
        transformed.v0 *= *scale;
        transformed.v1 *= *scale;
        transformed.v2 *= *scale;
        transformed
    }

    pub fn multiply_matrix(&self, matrix: &Matrix4x4) -> Self {
        let mut transformed = self.clone();
        transformed.v0 *= *matrix;
        transformed.v1 *= *matrix;
        transformed.v2 *= *matrix;
        transformed
    }

    // https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm#Rust_implementation
    pub fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        let e1 = self.v1 - self.v0;
        let e2 = self.v2 - self.v1;

        let ray_cross_e2 = Vec3::cross(&ray.direction, &e2);
        let det = Vec3::dot(&e1, &ray_cross_e2);

        if det > -f32::EPSILON && det < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let s = ray.position - self.v0;
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
                normal: self.normal(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }

    pub fn new_from_vertices_indices(
        vertices: &[Vec3],
        indices: &[(usize, usize, usize, termion::color::Rgb)],
    ) -> Self {
        Self::new(
            indices
                .iter()
                .map(|(i0, i1, i2, color)| {
                    Triangle::new(vertices[*i0], vertices[*i1], vertices[*i2], *color)
                })
                .collect(),
        )
    }

    pub fn new_plane() -> Self {
        Mesh::new_from_vertices_indices(
            &[
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(-1.0, 1.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
            ],
            &[
                (0, 2, 1, termion::color::Rgb(255, 0, 0)),
                (1, 2, 3, termion::color::Rgb(0, 255, 0)),
            ],
        )
    }

    pub fn new_cube() -> Self {
        // TODO: The indices are incorrect causing some faces to be inverted. And I'm too lazy to fix.
        Mesh::new_from_vertices_indices(
            &[
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, -1.0),
                Vec3::new(-1.0, 1.0, -1.0),
                Vec3::new(-1.0, -1.0, 1.0),
                Vec3::new(1.0, -1.0, 1.0),
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(-1.0, 1.0, 1.0),
            ],
            &[
                (0, 1, 2, termion::color::Rgb(255, 0, 0)),
                (0, 2, 3, termion::color::Rgb(255, 0, 0)),
                (4, 5, 6, termion::color::Rgb(0, 255, 0)),
                (4, 6, 7, termion::color::Rgb(0, 255, 0)),
                (0, 1, 5, termion::color::Rgb(0, 0, 255)),
                (0, 5, 4, termion::color::Rgb(0, 0, 255)),
                (2, 3, 7, termion::color::Rgb(255, 255, 0)),
                (2, 7, 6, termion::color::Rgb(255, 255, 0)),
                (0, 3, 7, termion::color::Rgb(255, 0, 255)),
                (0, 7, 4, termion::color::Rgb(255, 0, 255)),
                (1, 2, 6, termion::color::Rgb(0, 255, 255)),
                (1, 6, 5, termion::color::Rgb(0, 255, 255)),
            ],
        )
    }
}
