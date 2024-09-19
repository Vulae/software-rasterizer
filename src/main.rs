mod camera;
mod display;
mod loaders;
mod matrix;
mod mesh;
mod raycast;
mod reader;
mod scene;
mod vector;

use std::{error::Error, fmt::Write, io::Write as _, path::PathBuf};

use camera::{Camera, CameraOrbitController};
use clap::Parser;
use display::{Cell, Display, Drawer};
use matrix::Matrix4x4;
use scene::Scene;
use termion::{input::TermRead, raw::IntoRawMode};
use vector::Vec3;

static CELL_ASPECT_RATIO: f32 = 9.0 / 20.0;

static BG_COLOR: Cell = Cell::new_bg(termion::color::Rgb(0, 0, 0));
static FG_COLOR: Cell = Cell::new_bg(termion::color::Rgb(255, 255, 255));

fn render(
    debug_text: &mut String,
    drawer: &mut Drawer<'_>,
    scene: &Scene,
    camera: &Camera,
) -> Result<(), Box<dyn Error>> {
    let camera_matrix = camera.view_matrix();

    let aspect_ratio = (drawer.width() as f32) * CELL_ASPECT_RATIO / (drawer.height() as f32);
    let projection_matrix =
        Matrix4x4::perspective_projection_matrix(aspect_ratio, 90.0, 0.1, 1000.0);

    writeln!(
        debug_text,
        "out buffer: {}x{}",
        drawer.width(),
        drawer.height(),
    )?;

    writeln!(debug_text, "camera: {:?}", camera)?;

    let mut render_triangles = scene
        .meshes
        .iter()
        .flat_map(|mesh| {
            mesh.triangles.iter().filter_map(|triangle| {
                let mut camera_triangle = triangle.multiply_matrix(&camera_matrix);

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
        .sort_by(|a, b| (b.v0.z + b.v1.z + b.v2.z).total_cmp(&(a.v0.z + a.v1.z + a.v2.z)));

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
        "Press Q or ESC to quit. Click & drag mouse to orbit. Scroll to zoom."
    )?;
    stdout.flush()?;

    let mut controller = CameraOrbitController::new(Camera::new());
    controller.zoom(-50.0);

    let mut render_count = 0;
    let mut mouse_button: Option<termion::event::MouseButton> = None;
    let mut mouse_pos: (usize, usize) = (0, 0);

    for event in stdin.events() {
        let mut display = Display::init(&BG_COLOR)?;
        let mut debug_text = String::new();

        let event = event?;
        let mut mouse_movement: (isize, isize) = (0, 0);
        match event {
            termion::event::Event::Key(termion::event::Key::Esc)
            | termion::event::Event::Key(termion::event::Key::Char('q')) => break,
            termion::event::Event::Mouse(termion::event::MouseEvent::Press(press_button, x, y)) => {
                match press_button {
                    termion::event::MouseButton::WheelUp => {
                        controller.zoom(5.0);
                    }
                    termion::event::MouseButton::WheelDown => {
                        controller.zoom(-5.0);
                    }
                    _ => {
                        mouse_button = Some(press_button);
                        mouse_movement = (0, 0);
                        mouse_pos = ((x as usize) - 1, (y as usize) - 1);
                    }
                }
            }
            termion::event::Event::Mouse(termion::event::MouseEvent::Release(x, y)) => {
                mouse_button = None;
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
            "{:?} {:?} {:?}",
            mouse_button, mouse_pos, mouse_movement,
        )?;

        let mut drawer = Drawer::new(&mut display);

        if mouse_button == Some(termion::event::MouseButton::Left) {
            controller.grab_move(
                (mouse_movement.0 as f32) / 10.0,
                (mouse_movement.1 as f32) / CELL_ASPECT_RATIO / 10.0,
            );
        }

        render_count += 1;
        writeln!(debug_text, "Render Count: {}", render_count)?;
        render(&mut debug_text, &mut drawer, &scene, &controller.camera)?;

        drawer.text(0, 0, &debug_text, None, None);
        display.display(&mut stdout)?;
    }

    println!("{}{}", termion::screen::ToMainScreen, termion::cursor::Show);

    Ok(())
}
