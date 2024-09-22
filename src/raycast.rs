#![allow(unused)]

use crate::math::vector3::Vec3;

#[derive(Debug)]
pub struct Ray {
    pub position: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(position: Vec3, direction: Vec3) -> Self {
        Self {
            position,
            direction: direction.normalized(),
        }
    }
}

#[derive(Debug)]
pub struct RayIntersection {
    pub distance: f32,
    pub position: Vec3,
    pub normal: Vec3,
}
