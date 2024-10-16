extern crate sdl2;

pub mod game;
pub mod graphics;
pub mod logic;

use game::Game;
use graphics::GameRenderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::cell::RefCell;
use std::time::Duration;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Sumdoku a.k.a Killer Sudoku", 730, 1000)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    let game = Game::new();
    let mut renderer = GameRenderer::new(RefCell::new(canvas), RefCell::new(game), 80);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(key),
                    ..
                } => renderer.handle_keyboard_input(key),
                Event::MouseButtonDown { x, y, .. } => renderer.handle_click(x, y),
                _ => {}
            }
        }

        renderer.render()?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
