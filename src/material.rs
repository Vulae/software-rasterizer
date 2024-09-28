#![allow(unused)]

use image::{Rgb, RgbaImage};

pub trait Material: std::fmt::Debug {
    fn sample(&self, u: f32, v: f32) -> Rgb<u8>;
}

#[derive(Debug)]
pub struct MaterialGenericTexture {
    image: RgbaImage,
    avg_color: Rgb<u8>,
}

impl MaterialGenericTexture {
    pub fn new(image: RgbaImage) -> Self {
        Self {
            avg_color: {
                let mut pixels = 0u64;
                let total = image.pixels().fold((0u64, 0u64, 0u64), |(tr, tg, tb), c| {
                    if (c.0[0] == 0 && c.0[1] == 0 && c.0[2] == 0) || c.0[3] == 0 {
                        (tr, tg, tb)
                    } else {
                        pixels += 1;
                        (tr + c.0[0] as u64, tg + c.0[1] as u64, tb + c.0[2] as u64)
                    }
                });
                Rgb([
                    (total.0 / pixels) as u8,
                    (total.1 / pixels) as u8,
                    (total.2 / pixels) as u8,
                ])
            },
            image,
        }
    }
}

impl Material for MaterialGenericTexture {
    fn sample(&self, u: f32, v: f32) -> Rgb<u8> {
        // For now we just return avg color for testing
        self.avg_color
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
