#![allow(unused)]

use image::{Rgb, RgbaImage};

#[derive(Debug)]
pub struct MaterialGenericTexture {
    image: RgbaImage,
}

impl MaterialGenericTexture {
    pub fn new(image: RgbaImage) -> Self {
        Self { image }
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

#[derive(Debug)]
pub enum Material {
    GenericTexture(MaterialGenericTexture),
    GenericColor(MaterialGenericColor),
}

impl Material {
    pub fn sample(&self, _u: f32, _v: f32) -> Rgb<u8> {
        match self {
            Material::GenericTexture(_) => todo!(),
            Material::GenericColor(mat) => mat.color,
        }
    }
}
