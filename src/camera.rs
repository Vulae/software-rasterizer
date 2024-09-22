#![allow(unused)]

use crate::math::{frustum::Frustum, matrix4x4::Matrix4x4, plane::Plane, vector3::Vec3};

pub trait Camera {
    fn position(&self) -> Vec3;
    fn forward(&self) -> Vec3;
    fn up(&self) -> Vec3;
    fn right(&self) -> Vec3 {
        Vec3::cross(&self.up(), &self.forward())
    }

    /// Move relative to current camera look direction.
    fn r#move(&mut self, forward: f32, up: f32, right: f32);
    /// Rotate relative to current camera look direction.
    fn rotate(&mut self, pitch: f32, yaw: f32, roll: f32);

    fn matrix_view(&self) -> Matrix4x4 {
        Matrix4x4::view_matrix(&self.position(), &self.forward(), &self.up())
    }
    fn matrix_projection(&self) -> Matrix4x4;

    fn frustum(&self) -> Frustum {
        Frustum::from_projection_matrix(&(self.matrix_view() * self.matrix_projection()))
    }
}

#[derive(Debug)]
pub struct PerspectiveCamera {
    position: Vec3,
    direction: Vec3,
    up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

impl PerspectiveCamera {
    /// Aspect is expected to be set often, set PerspectiveCamera::aspect
    pub fn new(fov: f32, near: f32, far: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov,
            near,
            far,
            aspect: 1.0,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn forward(&self) -> Vec3 {
        self.direction
    }

    fn up(&self) -> Vec3 {
        self.up
    }

    fn r#move(&mut self, forward: f32, up: f32, right: f32) {
        self.position += self.forward() * forward + self.up() * up + self.right() * right;
    }

    fn rotate(&mut self, pitch: f32, yaw: f32, roll: f32) {
        // TODO: Roll???
        self.direction = (self.direction * Matrix4x4::rotate_axis(&self.up(), yaw)).normalized();
        self.direction =
            (self.direction * Matrix4x4::rotate_axis(&-self.right(), pitch)).normalized();
        self.up = Vec3::cross(&-self.right(), &self.direction).normalized();
    }

    fn matrix_projection(&self) -> Matrix4x4 {
        Matrix4x4::perspective_projection_matrix(self.aspect, self.fov, self.near, self.far)
    }
}

#[derive(Debug)]
pub struct CameraOrbitController<C: Camera> {
    pub camera: C,
    pub distance: f32,
}

impl<C: Camera> CameraOrbitController<C> {
    pub fn new(camera: C) -> Self {
        Self {
            camera,
            distance: 0.0,
        }
    }

    pub fn grab_move(&mut self, mx: f32, my: f32) {
        self.camera.r#move(self.distance, 0.0, 0.0);
        self.camera.rotate(my, mx, 0.0);
        self.camera.r#move(-self.distance, 0.0, 0.0);
    }

    pub fn pan_move(&mut self, mx: f32, my: f32) {
        self.camera
            .r#move(0.0, my * self.distance, mx * self.distance);
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.camera.r#move(self.distance, 0.0, 0.0);
        self.distance = distance;
        self.camera.r#move(-self.distance, 0.0, 0.0);
    }

    pub fn zoom_in(&mut self) {
        self.camera.r#move(self.distance, 0.0, 0.0);
        self.distance *= 0.95;
        if self.distance < 0.1 {
            self.distance = 0.1;
        }
        self.camera.r#move(-self.distance, 0.0, 0.0);
    }

    pub fn zoom_out(&mut self) {
        self.camera.r#move(self.distance, 0.0, 0.0);
        self.distance /= 0.95;
        self.camera.r#move(-self.distance, 0.0, 0.0);
    }
}
