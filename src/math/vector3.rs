#![allow(unused)]

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::matrix4x4::Matrix4x4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[macro_export]
macro_rules! vec3 {
    ($x:expr, $y:expr, $z:expr $(,)? $(,)?) => {
        Vec3::new(($x) as f32, ($y) as f32, ($z) as f32)
    };
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl DivAssign for Vec3 {
    fn div_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Vec3 {
    pub fn length(&self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn distance(&self, rhs: &Self) -> f32 {
        (*self - *rhs).length()
    }

    pub fn normalized(&self) -> Self {
        *self / self.length()
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn sum(&self) -> f32 {
        self.x + self.y + self.z
    }
}

impl Mul<Matrix4x4> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Matrix4x4) -> Self::Output {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        let w = 1.0;
        Self {
            x: x * rhs[(0, 0)] + y * rhs[(0, 1)] + z * rhs[(0, 2)] + w * rhs[(0, 3)],
            y: x * rhs[(1, 0)] + y * rhs[(1, 1)] + z * rhs[(1, 2)] + w * rhs[(1, 3)],
            z: x * rhs[(2, 0)] + y * rhs[(2, 1)] + z * rhs[(2, 2)] + w * rhs[(2, 3)],
        } / (x * rhs[(3, 0)] + y * rhs[(3, 1)] + z * rhs[(3, 2)] + w * rhs[(3, 3)])
    }
}

impl MulAssign<Matrix4x4> for Vec3 {
    fn mul_assign(&mut self, rhs: Matrix4x4) {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        let w = 1.0;
        *self = Self {
            x: x * rhs[(0, 0)] + y * rhs[(0, 1)] + z * rhs[(0, 2)] + w * rhs[(0, 3)],
            y: x * rhs[(1, 0)] + y * rhs[(1, 1)] + z * rhs[(1, 2)] + w * rhs[(1, 3)],
            z: x * rhs[(2, 0)] + y * rhs[(2, 1)] + z * rhs[(2, 2)] + w * rhs[(2, 3)],
        } / (x * rhs[(3, 0)] + y * rhs[(3, 1)] + z * rhs[(3, 2)] + w * rhs[(3, 3)]);
    }
}
