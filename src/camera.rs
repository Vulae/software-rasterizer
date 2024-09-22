#![allow(unused)]

use crate::{matrix::Matrix4x4, vector::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    direction: Vec3,
    distance: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlaneSide {
    Frontside,
    Onside,
    Backside,
}

impl Plane {
    pub fn new(direction: Vec3, distance: f32) -> Self {
        Self {
            direction: direction.normalized(),
            distance,
        }
    }

    pub fn signed_distance(&self, position: &Vec3) -> f32 {
        Vec3::dot(&self.direction, position) - self.distance
    }

    pub fn side(&self, position: &Vec3) -> PlaneSide {
        const EPS: f32 = 0.000060915946;
        let distance = self.signed_distance(position);
        if distance < -EPS {
            PlaneSide::Backside
        } else if distance > EPS {
            PlaneSide::Frontside
        } else {
            PlaneSide::Onside
        }
    }
}

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
}

#[test]
fn frustum_tests() {
    {
        let frustum = Frustum::new(Box::new([
            Plane::new(Vec3::new(0.0, 0.0, 1.0), -1.0),  // Near plane
            Plane::new(Vec3::new(0.0, 0.0, -1.0), -1.0), // Far plane
            Plane::new(Vec3::new(1.0, 0.0, 0.0), -1.0),  // Left plane
            Plane::new(Vec3::new(-1.0, 0.0, 0.0), -1.0), // Right plane
            Plane::new(Vec3::new(0.0, 1.0, 0.0), -1.0),  // Bottom plane
            Plane::new(Vec3::new(0.0, -1.0, 0.0), -1.0), // Top plane
        ]));

        let point = Vec3::new(0.0, 0.0, 0.0);

        assert!(frustum.contains(&point));
    }
    {
        let mut camera = PerspectiveCamera::new(1.0, 0.1, 100.0);
        camera.r#move(-10.0, 0.0, 0.0);
        let frustum = camera.frustum();

        let point = Vec3::new(0.0, 0.0, 0.0);
        frustum.planes().enumerate().for_each(|(i, plane)| {
            println!(
                "{}: {:?} {:?}",
                i,
                plane.signed_distance(&point),
                plane.side(&point),
            );
        });

        assert!(frustum.contains(&point));
    }
}

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

    #[rustfmt::skip]
    fn frustum(&self) -> Frustum {
        // https://gamedev.stackexchange.com/questions/159918/how-do-you-handle-large-triangles-with-frustum#answer-159920
        // TODO: Is this valid for every type of projection?
        // FIXME: I don't think this works properly lol
        let m = self.matrix_view() * self.matrix_projection();
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
