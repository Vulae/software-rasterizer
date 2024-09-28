#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uv {
    pub u: f32,
    pub v: f32,
}

impl Uv {
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
}
