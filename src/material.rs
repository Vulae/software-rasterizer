#![allow(unused)]

use image::{Pixel, Rgb, RgbaImage};

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
        let width = self.image.width();
        let height = self.image.height();

        let x = ((u * width as f32).floor() as u32) % width;
        let y = ((v * height as f32).floor() as u32) % height;

        self.image.get_pixel(x, y).to_rgb()
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
