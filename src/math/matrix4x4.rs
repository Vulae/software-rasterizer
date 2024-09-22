#![allow(unused)]

use std::ops::{Index, IndexMut, Mul};

use super::vector3::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4 {
    pub elements: [[f32; 4]; 4],
}

impl Matrix4x4 {
    pub const fn new(elements: [[f32; 4]; 4]) -> Self {
        Self { elements }
    }

    pub fn empty() -> Self {
        Self::new([[0f32; 4]; 4])
    }

    #[rustfmt::skip]
    pub fn identity_matrix() -> Self {
        Self::new([
            [ 1.0, 0.0, 0.0, 0.0 ],
            [ 0.0, 1.0, 0.0, 0.0 ],
            [ 0.0, 0.0, 1.0, 0.0 ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ])
    }

    #[rustfmt::skip]
    pub fn view_matrix(camera_pos: &Vec3, camera_dir: &Vec3, camera_up: &Vec3) -> Self {
        let camera_dir = camera_dir.normalized();
        let camera_right = Vec3::cross(camera_up, &camera_dir);
        let camera_up = Vec3::cross(&camera_dir, &camera_right);
        Self::new([
            [ camera_right.x                       , camera_up.x                       , camera_dir.x                       , 0.0 ],
            [ camera_right.y                       , camera_up.y                       , camera_dir.y                       , 0.0 ],
            [ camera_right.z                       , camera_up.z                       , camera_dir.z                       , 0.0 ],
            [ -Vec3::dot(&camera_right, camera_pos), -Vec3::dot(&camera_up, camera_pos), -Vec3::dot(&camera_dir, camera_pos), 1.0 ],
        ])
    }

    #[rustfmt::skip]
    pub fn perspective_projection_matrix(aspect_ratio: f32, fov: f32, near: f32, far: f32) -> Self {
        assert!(near > 0.0);
        assert!(near < far);

        let tan_half_fov = f32::tan(fov.to_radians() / 2.0);

        Self::new([
            [ 1.0 / (aspect_ratio * tan_half_fov), 0.0               , 0.0                         , 0.0                              ],
            [ 0.0                                , 1.0 / tan_half_fov, 0.0                         , 0.0                              ],
            [ 0.0                                , 0.0               , -(far + near) / (far - near), -2.0 * far * near / (far - near) ],
            [ 0.0                                , 0.0               , -1.0                        , 0.0                              ],
        ])

        //let fov_rad = 1.0 / f32::tan(fov * 0.5 / 180.0 * std::f32::consts::PI);
        //
        //let mut matrix = Self::empty();
        //
        //matrix[(0, 0)] = 1.0 / aspect_ratio * fov_rad;
        //matrix[(1, 1)] = fov_rad;
        //matrix[(2, 2)] = far / (far - near);
        //matrix[(3, 2)] = (-far * near) / (far - near);
        //matrix[(2, 3)] = 1.0;
        //
        //matrix
    }

    #[rustfmt::skip]
    pub fn translation_matrix(translation: Vec3) -> Self {
        Self::new([
            [ 1.0, 0.0, 0.0, translation.x ],
            [ 0.0, 1.0, 0.0, translation.y ],
            [ 0.0, 0.0, 1.0, translation.z ],
            [ 0.0, 0.0, 0.0, 1.0           ],
        ])
    }

    #[rustfmt::skip]
    pub fn scale_matrix(scale: Vec3) -> Self {
        Self::new([
            [ scale.x, 0.0    , 0.0    , 0.0 ],
            [ 0.0    , scale.y, 0.0    , 0.0 ],
            [ 0.0    , 0.0    , scale.z, 0.0 ],
            [ 0.0    , 0.0    , 0.0    , 1.0 ],
        ])
    }

    #[rustfmt::skip]
    pub fn rotate_x(amount: f32) -> Self {
        Self::new([
            [ 1.0, 0.0             , 0.0              , 0.0 ],
            [ 0.0, f32::cos(amount), -f32::sin(amount), 0.0 ],
            [ 0.0, f32::sin(amount), f32::cos(amount) , 0.0 ],
            [ 0.0, 0.0             , 0.0              , 1.0 ],
        ])
    }

    #[rustfmt::skip]
    pub fn rotate_y(amount: f32) -> Self {
        Self::new([
            [ f32::cos(amount) , 0.0, f32::sin(amount), 0.0 ],
            [ 0.0              , 1.0, 0.0             , 0.0 ],
            [ -f32::sin(amount), 0.0, f32::cos(amount), 0.0 ],
            [ 0.0              , 0.0, 0.0             , 1.0 ],
        ])
    }

    #[rustfmt::skip]
    pub fn rotate_z(amount: f32) -> Self {
        Self::new([
            [ f32::cos(amount), -f32::sin(amount), 0.0, 0.0 ],
            [ f32::sin(amount), f32::cos(amount) , 0.0, 0.0 ],
            [ 0.0             , 0.0              , 1.0, 0.0 ],
            [ 0.0             , 0.0              , 0.0, 1.0 ],
        ])
    }

    #[rustfmt::skip]
    pub fn rotation_matrix(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self::new([
            [ f32::cos(yaw)*f32::cos(pitch), f32::cos(yaw)*f32::sin(pitch)*f32::sin(roll) - f32::sin(yaw)*f32::cos(roll), f32::cos(yaw)*f32::sin(pitch)*f32::cos(roll) + f32::sin(yaw)*f32::sin(roll), 0.0 ],
            [ f32::sin(yaw)*f32::cos(pitch), f32::sin(yaw)*f32::sin(pitch)*f32::sin(roll) + f32::cos(yaw)*f32::cos(roll), f32::sin(yaw)*f32::sin(pitch)*f32::cos(roll) - f32::cos(yaw)*f32::sin(roll), 0.0 ],
            [ -f32::sin(pitch)             , f32::cos(pitch)*f32::sin(roll)                                             , f32::cos(pitch)*f32::cos(roll)                                             , 0.0 ],
            [ 0.0                          , 0.0                                                                        , 0.0                                                                        , 1.0 ],
        ])
    }

    #[rustfmt::skip]
    pub fn rotate_axis(axis: &Vec3, angle: f32) -> Self {
        let c = f32::cos(angle);
        let s = f32::sin(angle);
        let t = 1.0 - c;
        let x = axis.x;
        let y = axis.y;
        let z = axis.z;
        Self::new([
            [ t * x * x + c    , t * x * y - s * z, t * x * z + s * y, 0.0 ],
            [ t * x * y + s * z, t * y * y + c    , t * y * z - s * x, 0.0 ],
            [ t * x * z - s * y, t * y * z + s * x, t * z * z + c    , 0.0 ],
            [ 0.0              , 0.0              , 0.0              , 1.0 ],
        ])
    }
}

#[macro_export]
macro_rules! matrix4x4 {
    ($a:expr, $b:expr, $c:expr, $d:expr; $e:expr, $f:expr, $g:expr, $h:expr; $i:expr, $j:expr, $k:expr, $l:expr; $m:expr, $n:expr, $o:expr, $p:expr $(;)? $(;)?) => {
        Matrix4x4::new([
            [($a) as f32, ($b) as f32, ($c) as f32, ($d) as f32],
            [($e) as f32, ($f) as f32, ($g) as f32, ($h) as f32],
            [($i) as f32, ($j) as f32, ($k) as f32, ($l) as f32],
            [($m) as f32, ($n) as f32, ($o) as f32, ($p) as f32],
        ])
    };
}

impl Index<(usize, usize)> for Matrix4x4 {
    type Output = f32;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.elements[index.1][index.0]
    }
}

impl IndexMut<(usize, usize)> for Matrix4x4 {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.elements[index.1][index.0]
    }
}

impl Mul for Matrix4x4 {
    type Output = Matrix4x4;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Matrix4x4::empty();

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[(j, i)] += self[(k, i)] * rhs[(j, k)];
                }
            }
        }

        result
    }
}

#[test]
fn matrix_tests() {
    let m1 = matrix4x4!(1, 2, 3, 4; 5, 6, 7, 8; 9, 10, 11, 12; 13, 14, 15, 16);
    assert_eq!(Matrix4x4::identity_matrix() * m1, m1);
    let m2 = matrix4x4!(17, 18, 19, 20; 21, 22, 23, 24; 25, 26, 27, 28; 29, 30, 31, 32);
    assert_eq!(
        m1 * m2,
        matrix4x4!(
            250, 260, 270, 280;
            618, 644, 670, 696;
            986, 1028, 1070, 1112;
            1354, 1412, 1470, 1528
        ),
    );
    let v1 = vec3!(2, 4, 8);
    assert_eq!(v1 * m2, vec3!(347, 362, 377));
}
