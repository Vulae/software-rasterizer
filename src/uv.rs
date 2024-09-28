#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uv {
    u: f32,
    v: f32,
}

impl Uv {
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
}
