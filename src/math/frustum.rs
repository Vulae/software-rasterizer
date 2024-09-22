#![allow(unused)]

use super::{
    matrix4x4::Matrix4x4,
    plane::{Plane, PlaneSide},
    vector3::Vec3,
};

#[derive(Debug, Clone)]
pub struct Frustum {
    planes: Box<[Plane]>,
}

impl Frustum {
    // TODO: Check if frustum is closed.
    pub fn new(planes: Box<[Plane]>) -> Self {
        Self { planes }
    }

    pub fn planes(&self) -> impl Iterator<Item = &Plane> {
        self.planes.iter()
    }

    pub fn contains(&self, position: &Vec3) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.side(position) != PlaneSide::Frontside)
    }

    #[rustfmt::skip]
    pub fn from_projection_matrix(m: &Matrix4x4) -> Self {
        // https://gamedev.stackexchange.com/questions/159918/how-do-you-handle-large-triangles-with-frustum#answer-159920
        // TODO: Is this valid for every type of projection, not just perspective projection?
        // FIXME: I don't think this works properly lol
        Frustum::new(Box::new([
            Plane::new(Vec3::new(m[(0, 3)] + m[(0, 2)], m[(1, 3)] + m[(1, 2)], m[(2, 3)] + m[(2, 2)]), m[(3, 3)] + m[(3, 2)]), // Near
            Plane::new(Vec3::new(m[(0, 3)] - m[(0, 2)], m[(1, 3)] - m[(1, 2)], m[(2, 3)] - m[(2, 2)]), m[(3, 3)] - m[(3, 2)]), // Far
            Plane::new(Vec3::new(m[(0, 3)] + m[(0, 0)], m[(1, 3)] + m[(1, 0)], m[(2, 3)] + m[(2, 0)]), m[(3, 3)] + m[(3, 0)]), // Left
            Plane::new(Vec3::new(m[(0, 3)] - m[(0, 0)], m[(1, 3)] - m[(1, 0)], m[(2, 3)] - m[(2, 0)]), m[(3, 3)] - m[(3, 0)]), // Right
            Plane::new(Vec3::new(m[(0, 3)] - m[(0, 1)], m[(1, 3)] - m[(1, 1)], m[(2, 3)] - m[(2, 1)]), m[(3, 3)] - m[(3, 1)]), // Top
            Plane::new(Vec3::new(m[(0, 3)] + m[(0, 1)], m[(1, 3)] + m[(1, 1)], m[(2, 3)] + m[(2, 1)]), m[(3, 3)] + m[(3, 1)]), // Bottom
        ]))
    }
}
