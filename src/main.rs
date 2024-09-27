mod camera;
mod display;
mod loaders;
mod material;
mod math;
mod mesh;
mod raycast;
mod reader;
mod renderer;
mod scene;

use std::{error::Error, fmt::Write, io::Write as _, path::PathBuf};

use clap::Parser;
use display::Drawer;
use material::MaterialGenericColor;
use renderer::Renderer;
use scene::Scene;
use termion::{input::TermRead, raw::IntoRawMode};

static CELL_ASPECT_RATIO: f32 = 9.0 / 20.0;

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
                .materials
                .push(Box::new(MaterialGenericColor::new(image::Rgb([
                    255, 255, 255,
                ]))));
            scene
        }
        Some("glb") => loaders::gltf::load_scene(std::fs::File::open(cli.file)?)?,
        _ => panic!("Invalid file."),
    };

    //let stdin = std::io::stdin();
    // Initialize stdout for raw mode & mouse input.
    let mut stdout = termion::input::MouseTerminal::from(std::io::stdout().lock().into_raw_mode()?);
    let stdin = termion::async_stdin();

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

    let mut renderer = Renderer::new(scene);
    renderer.controller.set_distance(100.0);

    let mut mouse_left: bool = false;
    let mut mouse_right: bool = false;
    let mut mouse_pos: (usize, usize) = (0, 0);

    let mut events = stdin.events(); //.peekable();
    'outer: loop {
        let (width, height) = termion::terminal_size()?;
        let (width, height) = (width as usize, height as usize);
        renderer.controller.camera.aspect = (width as f32) * CELL_ASPECT_RATIO / (height as f32);

        let mut mouse_movement: (isize, isize) = (0, 0);
        for event in events.by_ref() {
            let event = event?;
            match event {
                termion::event::Event::Key(termion::event::Key::Esc)
                | termion::event::Event::Key(termion::event::Key::Char('q')) => break 'outer,
                termion::event::Event::Mouse(termion::event::MouseEvent::Press(
                    press_button,
                    x,
                    y,
                )) => match press_button {
                    termion::event::MouseButton::WheelUp => {
                        renderer.controller.zoom_in();
                    }
                    termion::event::MouseButton::WheelDown => {
                        renderer.controller.zoom_out();
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
                },
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
                termion::event::Event::Key(termion::event::Key::Left) => {
                    renderer.controller.roll(0.2);
                }
                termion::event::Event::Key(termion::event::Key::Right) => {
                    renderer.controller.roll(-0.2);
                }
                _ => {}
            }

            if mouse_left {
                if !mouse_right {
                    renderer.controller.grab_move(
                        (mouse_movement.0 as f32) / 10.0,
                        (mouse_movement.1 as f32) / CELL_ASPECT_RATIO / 10.0,
                    );
                } else {
                    renderer.controller.pan_move(
                        (mouse_movement.0 as f32) / 100.0,
                        (mouse_movement.1 as f32) / CELL_ASPECT_RATIO / 100.0,
                    );
                }
            }
        }

        let mut dbg_text = String::new();

        writeln!(
            dbg_text,
            "{} {} {:?} {:?}",
            mouse_left, mouse_right, mouse_pos, mouse_movement,
        )?;

        writeln!(dbg_text, "Controller: {:?}", renderer.controller)?;

        let (mut display, render_info) = renderer.render(width, height)?;
        writeln!(dbg_text, "Render info: {:?}", render_info)?;

        let mut drawer = Drawer::new(&mut display);
        drawer.text(0, 0, &dbg_text, None, None);

        display.display(&mut stdout)?;
    }

    println!("{}{}", termion::screen::ToMainScreen, termion::cursor::Show);

    Ok(())
}
