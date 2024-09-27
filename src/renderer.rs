#![allow(unused)]

use std::error::Error;

use image::Pixel;

use crate::{
    camera::{Camera, CameraOrbitController, PerspectiveCamera},
    display::{Cell, Display, Drawer},
    math::vector3::Vec3,
    mesh::Triangle,
    scene::Scene,
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
            projected_triangle: Triangle,
            view_normal: Vec3,
            material_index: usize,
        }

        let mut render_triangles: Vec<RenderTriangle> = self
            .scene
            .meshes
            .iter()
            .flat_map(|mesh| {
                mesh.triangles.iter().filter_map(|triangle| {
                    let camera_triangle = triangle.multiply_matrix(&camera_matrix);

                    // HACK: *Bad* frustum culling
                    if !camera_frustum.contains(&camera_triangle.v0)
                        || !camera_frustum.contains(&camera_triangle.v1)
                        || !camera_frustum.contains(&camera_triangle.v2)
                    {
                        return None;
                    }

                    let camera_triangle_normal = camera_triangle.normal();

                    // Backside culling
                    if camera_triangle_normal.z > 0.0 {
                        return None;
                    }

                    let projected_triangle = camera_triangle.multiply_matrix(&projection_matrix);

                    Some(RenderTriangle {
                        projected_triangle,
                        view_normal: camera_triangle_normal,
                        material_index: mesh.material_index,
                    })
                })
            })
            .collect::<Vec<_>>();

        // HACK: Temporary triangle depth 'check'
        render_triangles.sort_by(|a, b| {
            (a.projected_triangle.v0.z + a.projected_triangle.v1.z + a.projected_triangle.v2.z)
                .total_cmp(
                    &(b.projected_triangle.v0.z
                        + b.projected_triangle.v1.z
                        + b.projected_triangle.v2.z),
                )
        });

        render_triangles.into_iter().for_each(|rt| {
            let screen_triangle = rt
                .projected_triangle
                .translate(&Vec3::new(1.0, 1.0, 0.0))
                .scale(&Vec3::new(
                    drawer.width() as f32 / 2.0,
                    drawer.height() as f32 / 2.0,
                    1.0,
                ));

            let material = &self.scene.materials[rt.material_index];
            let mut color = material.sample(0.0, 0.0);

            color.channels_mut().iter_mut().for_each(|c| {
                *c = ((*c as f32) * -rt.view_normal.z) as u8;
            });

            let cell = Cell::new_bg(termion::color::Rgb(color.0[0], color.0[1], color.0[2]));

            drawer.triangle(
                &cell,
                screen_triangle.v0.x as isize,
                screen_triangle.v0.y as isize,
                screen_triangle.v1.x as isize,
                screen_triangle.v1.y as isize,
                screen_triangle.v2.x as isize,
                screen_triangle.v2.y as isize,
            );
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
