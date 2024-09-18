#![allow(unused)]

use crate::{matrix::Matrix4x4, vector::Vec3};

#[derive(Debug)]
pub struct Camera {
    position: Vec3,
    direction: Vec3,
    up: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn forward(&self) -> Vec3 {
        self.direction
    }

    pub fn up(&self) -> Vec3 {
        self.up
    }

    pub fn right(&self) -> Vec3 {
        Vec3::cross(&self.up(), &self.forward())
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.position += *translation;
    }

    /// movement = Vec3::(forward, up, right)
    pub fn move_relative_to_direction(&mut self, movement: &Vec3) {
        let mut translation = Vec3::new(0.0, 0.0, 0.0);

        translation += self.forward() * movement.x;
        translation += self.up() * movement.y;
        translation += self.right() * movement.z;

        self.translate(&translation);
    }

    pub fn yaw(&mut self, add_yaw: f32) {
        self.direction =
            (self.direction * Matrix4x4::rotate_axis(&self.up(), add_yaw)).normalized();
    }

    pub fn pitch(&mut self, add_pitch: f32) {
        self.direction =
            (self.direction * Matrix4x4::rotate_axis(&-self.right(), add_pitch)).normalized();
        self.up = Vec3::cross(&-self.right(), &self.direction).normalized();
    }

    pub fn view_matrix(&self) -> Matrix4x4 {
        Matrix4x4::view_matrix(self.position, self.direction, self.up)
    }
}

#[derive(Debug)]
pub struct CameraOrbitController {
    pub camera: Camera,
    pub distance: f32,
}

impl CameraOrbitController {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            distance: 0.0,
        }
    }

    pub fn grab_move(&mut self, mx: f32, my: f32) {
        // NOTE: This is stupid and dumb, probably should get changed.
        // We move to the pivot point, then rotate, then move back.
        self.camera.position += self.camera.direction * self.distance;
        self.camera.yaw(mx);
        self.camera.pitch(my);
        self.camera.position -= self.camera.direction * self.distance;
    }

    pub fn zoom(&mut self, amount: f32) {
        self.camera.position += -self.camera.direction * amount;
        self.distance += amount;
    }
}
