#![allow(unused)]

use image::{Rgb, RgbaImage};

pub trait Material: std::fmt::Debug {
    fn sample(&self, u: f32, v: f32) -> Rgb<u8>;
}

#[derive(Debug)]
pub struct MaterialGenericTexture {
    image: RgbaImage,
}

impl MaterialGenericTexture {
    pub fn new(image: RgbaImage) -> Self {
        Self { image }
    }
}

impl Material for MaterialGenericTexture {
    fn sample(&self, u: f32, v: f32) -> Rgb<u8> {
        todo!()
    }
}

#[derive(Debug)]
pub struct MaterialGenericColor {
    color: Rgb<u8>,
}

impl MaterialGenericColor {
    pub fn new(color: Rgb<u8>) -> Self {
        Self { color }
    }
}

impl Material for MaterialGenericColor {
    fn sample(&self, u: f32, v: f32) -> Rgb<u8> {
        self.color
    }
}
