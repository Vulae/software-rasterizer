mod camera;
mod display;
mod loaders;
mod math;
mod mesh;
mod raycast;
mod reader;
mod scene;

use std::{error::Error, fmt::Write, io::Write as _, path::PathBuf};

use camera::{Camera, CameraOrbitController, PerspectiveCamera};
use clap::Parser;
use display::{Cell, Display, Drawer};
use math::vector3::Vec3;
use scene::Scene;
use termion::{input::TermRead, raw::IntoRawMode};

static CELL_ASPECT_RATIO: f32 = 9.0 / 20.0;

static BG_COLOR: Cell = Cell::new_bg(termion::color::Rgb(0, 0, 0));
//static FG_COLOR: Cell = Cell::new_bg(termion::color::Rgb(255, 255, 255));

fn render(
    debug_text: &mut String,
    drawer: &mut Drawer<'_>,
    scene: &Scene,
    camera: &impl Camera,
) -> Result<(), Box<dyn Error>> {
    let camera_matrix = camera.matrix_view();
    let camera_frustum = camera.frustum();
    let projection_matrix = camera.matrix_projection();

    writeln!(
        debug_text,
        "Out buffer: {}x{}",
        drawer.width(),
        drawer.height(),
    )?;

    let mut render_triangles = scene
        .meshes
        .iter()
        .flat_map(|mesh| {
            mesh.triangles.iter().filter_map(|triangle| {
                let mut camera_triangle = triangle.multiply_matrix(&camera_matrix);

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

                // Camera lighting
                camera_triangle.color.0 =
                    ((camera_triangle.color.0 as f32) * -camera_triangle_normal.z) as u8;
                camera_triangle.color.1 =
                    ((camera_triangle.color.1 as f32) * -camera_triangle_normal.z) as u8;
                camera_triangle.color.2 =
                    ((camera_triangle.color.2 as f32) * -camera_triangle_normal.z) as u8;

                let projected_triangle = camera_triangle.multiply_matrix(&projection_matrix);

                Some(projected_triangle)
            })
        })
        .collect::<Vec<_>>();

    // HACK: Temporary triangle depth 'check'
    render_triangles
        .sort_by(|a, b| (a.v0.z + a.v1.z + a.v2.z).total_cmp(&(b.v0.z + b.v1.z + b.v2.z)));

    render_triangles.iter().for_each(|triangle| {
        let screen_triangle = triangle
            .translate(&Vec3::new(1.0, 1.0, 0.0))
            .scale(&Vec3::new(
                drawer.width() as f32 / 2.0,
                drawer.height() as f32 / 2.0,
                1.0,
            ));

        let cell = Cell::new_bg(screen_triangle.color);

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

#[derive(Parser)]
struct Cli {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let scene = match cli.file.extension().and_then(|s| s.to_str()) {
        Some("obj") => {
            let mut scene = Scene::new();
            let mesh = loaders::obj::load_mesh(std::fs::File::open(cli.file)?)?;
            scene.meshes.push(mesh);
            scene
        }
        Some("glb") => loaders::gltf::load_scene(std::fs::File::open(cli.file)?)?,
        _ => panic!("Invalid file."),
    };

    let stdin = std::io::stdin();
    // Initialize stdout for raw mode & mouse input.
    let mut stdout = termion::input::MouseTerminal::from(std::io::stdout().lock().into_raw_mode()?);

    write!(
        stdout,
        "{}{}",
        termion::screen::ToAlternateScreen,
        termion::cursor::Hide,
    )?;
    write!(
        stdout,
        "Press Q or ESC to quit. Leftclick & drag mouse to orbit. Leftclick + rightclick & drag mouse to pan. Scroll to zoom."
    )?;
    stdout.flush()?;

    let mut controller = CameraOrbitController::new(PerspectiveCamera::new(90.0, 0.1, 1000.0));
    controller.set_distance(100.0);

    let mut render_count = 0;
    let mut mouse_left: bool = false;
    let mut mouse_right: bool = false;
    let mut mouse_pos: (usize, usize) = (0, 0);

    for event in stdin.events() {
        let mut display = Display::init(&BG_COLOR)?;
        controller.camera.aspect =
            (display.width() as f32) * CELL_ASPECT_RATIO / (display.height() as f32);
        let mut debug_text = String::new();

        let event = event?;
        let mut mouse_movement: (isize, isize) = (0, 0);
        match event {
            termion::event::Event::Key(termion::event::Key::Esc)
            | termion::event::Event::Key(termion::event::Key::Char('q')) => break,
            termion::event::Event::Mouse(termion::event::MouseEvent::Press(press_button, x, y)) => {
                match press_button {
                    termion::event::MouseButton::WheelUp => {
                        controller.zoom_in();
                    }
                    termion::event::MouseButton::WheelDown => {
                        controller.zoom_out();
                    }
                    _ => {
                        match press_button {
                            termion::event::MouseButton::Left => mouse_left = true,
                            termion::event::MouseButton::Right => mouse_right = true,
                            _ => {}
                        }
                        mouse_movement = (0, 0);
                        mouse_pos = ((x as usize) - 1, (y as usize) - 1);
                    }
                }
            }
            termion::event::Event::Mouse(termion::event::MouseEvent::Release(x, y)) => {
                mouse_left = false;
                mouse_right = false;
                mouse_movement = (
                    (x as isize) - 1 - (mouse_pos.0 as isize),
                    (y as isize) - 1 - (mouse_pos.1 as isize),
                );
                mouse_pos = ((x as usize) - 1, (y as usize) - 1);
            }
            termion::event::Event::Mouse(termion::event::MouseEvent::Hold(x, y)) => {
                mouse_movement = (
                    (x as isize) - 1 - (mouse_pos.0 as isize),
                    (y as isize) - 1 - (mouse_pos.1 as isize),
                );
                mouse_pos = ((x as usize) - 1, (y as usize) - 1);
                if mouse_movement.0 == 0 && mouse_movement.1 == 0 {
                    continue;
                }
            }
            _ => {}
        }

        writeln!(
            debug_text,
            "{} {} {:?} {:?}",
            mouse_left, mouse_right, mouse_pos, mouse_movement,
        )?;

        writeln!(debug_text, "Controller: {:?}", controller)?;

        let mut drawer = Drawer::new(&mut display);

        if mouse_left {
            if !mouse_right {
                controller.grab_move(
                    (mouse_movement.0 as f32) / 10.0,
                    (mouse_movement.1 as f32) / CELL_ASPECT_RATIO / 10.0,
                );
            } else {
                controller.pan_move(
                    (mouse_movement.0 as f32) / 100.0,
                    (mouse_movement.1 as f32) / CELL_ASPECT_RATIO / 100.0,
                );
            }
        }

        render_count += 1;
        writeln!(debug_text, "Render count: {}", render_count)?;
        render(&mut debug_text, &mut drawer, &scene, &controller.camera)?;

        drawer.text(0, 0, &debug_text, None, None);
        display.display(&mut stdout)?;
    }

    println!("{}{}", termion::screen::ToMainScreen, termion::cursor::Show);

    Ok(())
}
