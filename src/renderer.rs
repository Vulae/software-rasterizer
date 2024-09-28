#![allow(unused)]

use std::error::Error;

use image::Pixel;

use crate::{
    camera::{Camera, CameraOrbitController, PerspectiveCamera},
    display::{Cell, Display, Drawer},
    math::vector3::Vec3,
    mesh::triangle_normal,
    scene::Scene,
    uv::Uv,
};

static BG_COLOR: Cell = Cell::new_bg(termion::color::Rgb(0, 0, 0));

#[derive(Debug)]
pub struct Renderer {
    pub scene: Scene,
    pub controller: CameraOrbitController<PerspectiveCamera>,
    render_count: u64,
}

#[derive(Debug)]
pub struct RenderInfo {
    pub dbg_text: String,
    pub render_time: std::time::Duration,
    pub render_count: u64,
}

impl Renderer {
    pub fn new(scene: Scene) -> Self {
        Self {
            scene,
            controller: CameraOrbitController::new(PerspectiveCamera::new(90.0, 0.1, 1000.0)),
            render_count: 0,
        }
    }

    pub fn render_inner(
        &self,
        drawer: &mut Drawer,
        mut dbg_text: impl std::fmt::Write,
    ) -> Result<(), Box<dyn Error>> {
        let camera_matrix = self.controller.camera.matrix_view();
        let camera_frustum = self.controller.camera.frustum();
        let projection_matrix = self.controller.camera.matrix_projection();

        writeln!(
            dbg_text,
            "Out buffer: {}x{}",
            drawer.width(),
            drawer.height(),
        )?;

        #[derive(Debug)]
        struct RenderTriangle {
            v0: Vec3,
            v1: Vec3,
            v2: Vec3,
            t0: Uv,
            t1: Uv,
            t2: Uv,
            view_normal: Vec3,
            material_index: usize,
        }

        let mut render_triangles: Vec<RenderTriangle> = self
            .scene
            .meshes
            .iter()
            .flat_map(|mesh| {
                mesh.indices.iter().filter_map(|(i0, i1, i2)| {
                    let mut v0 = mesh.position[*i0];
                    let mut v1 = mesh.position[*i1];
                    let mut v2 = mesh.position[*i2];

                    let cam_v0 = v0 * camera_matrix;
                    let cam_v1 = v1 * camera_matrix;
                    let cam_v2 = v2 * camera_matrix;

                    // HACK: *Bad* frustum culling
                    if !camera_frustum.contains(&cam_v0)
                        || !camera_frustum.contains(&cam_v1)
                        || !camera_frustum.contains(&cam_v2)
                    {
                        return None;
                    }

                    let cam_normal = triangle_normal(&cam_v0, &cam_v1, &cam_v2);

                    // Backside culling
                    if cam_normal.z > 0.0 {
                        return None;
                    }

                    let proj_v0 = cam_v0 * projection_matrix;
                    let proj_v1 = cam_v1 * projection_matrix;
                    let proj_v2 = cam_v2 * projection_matrix;

                    Some(RenderTriangle {
                        v0: proj_v0,
                        v1: proj_v1,
                        v2: proj_v2,
                        t0: mesh.texcoord[*i0],
                        t1: mesh.texcoord[*i1],
                        t2: mesh.texcoord[*i2],
                        view_normal: cam_normal,
                        material_index: mesh.material_index,
                    })
                })
            })
            .collect::<Vec<_>>();

        // HACK: Temporary triangle depth 'check'
        render_triangles
            .sort_by(|a, b| (a.v0.z + a.v1.z + a.v2.z).total_cmp(&(b.v0.z + b.v1.z + b.v2.z)));

        let screenspace_mul_vec = Vec3::new(
            drawer.width() as f32 / 2.0,
            drawer.height() as f32 / 2.0,
            1.0,
        );

        render_triangles.into_iter().for_each(|rt| {
            #[rustfmt::skip]
            let screen_v0 = (rt.v0 + Vec3::new(1.0, 1.0, 0.0)) * screenspace_mul_vec;
            let screen_v1 = (rt.v1 + Vec3::new(1.0, 1.0, 0.0)) * screenspace_mul_vec;
            let screen_v2 = (rt.v2 + Vec3::new(1.0, 1.0, 0.0)) * screenspace_mul_vec;

            let material = &self.scene.materials[rt.material_index];
            //let mut color = material.sample(0.0, 0.0);

            //color.channels_mut().iter_mut().for_each(|c| {
            //    *c = ((*c as f32) * -rt.view_normal.z) as u8;
            //});

            //let cell = Cell::new_bg(termion::color::Rgb(color.0[0], color.0[1], color.0[2]));

            drawer
                .iter_triangle(
                    screen_v0.x as isize,
                    screen_v0.y as isize,
                    screen_v1.x as isize,
                    screen_v1.y as isize,
                    screen_v2.x as isize,
                    screen_v2.y as isize,
                )
                .for_each(|(px, py)| {
                    let (v0, v1, v2) = (screen_v0, screen_v1, screen_v2);
                    let denom = (v1.y - v2.y) * (v0.x - v2.x) + (v2.x - v1.x) * (v0.y - v2.y);
                    let a = ((v1.y - v2.y) * (px as f32 - v2.x)
                        + (v2.x - v1.x) * (py as f32 - v2.y))
                        / denom;
                    let b = ((v2.y - v0.y) * (px as f32 - v2.x)
                        + (v0.x - v2.x) * (py as f32 - v2.y))
                        / denom;
                    let c = 1.0 - a - b;

                    // FIXME: This condition stops weird textures with texture atlas.
                    // But it also makes holes in the triangles.
                    if a >= 0.0 && b >= 0.0 && c >= 0.0 {
                        let uv = Uv::new(
                            a * rt.t0.u + b * rt.t1.u + c * rt.t2.u,
                            a * rt.t0.v + b * rt.t1.v + c * rt.t2.v,
                        );

                        let color = material.sample(uv.u, uv.v);
                        let cell =
                            Cell::new_bg(termion::color::Rgb(color.0[0], color.0[1], color.0[2]));

                        drawer.pixel(&cell, px, py);
                    }
                });
        });

        Ok(())
    }

    pub fn render(
        &mut self,
        width: usize,
        height: usize,
    ) -> Result<(Display, RenderInfo), Box<dyn Error>> {
        let start = std::time::Instant::now();
        let mut dbg_text = String::new();

        let mut display = Display::init_with_size(&BG_COLOR, width, height);
        let mut drawer = Drawer::new(&mut display);

        self.render_inner(&mut drawer, &mut dbg_text)?;

        self.render_count += 1;

        Ok((
            display,
            RenderInfo {
                dbg_text,
                render_time: start.elapsed(),
                render_count: self.render_count,
            },
        ))
    }
}
