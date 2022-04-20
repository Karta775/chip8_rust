use log::{debug, error, info, warn};
use std::ptr::null;

mod chip8;
use chip8::Chip8;
use clap::Parser;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::TextureAccess;
use std::time::Duration;
use std::{thread, time};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 320;

/// CHIP-8 Emulator
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the ROM file to run
    #[clap(required = true)]
    romfile: String,
}

fn main() {
    // SDL Init
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("CHIP-8", 640, 320)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture_data: [u8; 64 * 32 * 4] = [0; 64 * 32 * 4];
    let mut texture = texture_creator
        .create_texture(PixelFormatEnum::ARGB8888, TextureAccess::Streaming, 64, 32)
        .unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // Parse command line arguments
    let args = Args::parse();

    // Initialise the logger
    env_logger::init();

    // Set up CHIP-8 and load the ROM
    let mut keypress: Option<u8> = None;
    let mut chip8 = Chip8::new();
    chip8.load_rom(&args.romfile);

    // Set up timer
    let now = time::Instant::now();
    let mut old_time = now.elapsed().as_secs();
    let mut ops_per_sec = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Compute and print the instructions per second TODO: Build into UI (ImGui?)
        if now.elapsed().as_secs() == old_time {
            ops_per_sec += 1;
        } else {
            warn!("{} instructions per second", ops_per_sec);
            old_time = now.elapsed().as_secs();
            ops_per_sec = 0;
        }

        // Fetch, decode, execute
        chip8.tick(keypress);

        // Blit the pixels from chip8.display to pixels if the draw flag is set
        if chip8.redraw {
            blit(&chip8.display, &mut texture_data);
            texture.update(None, &texture_data, 64 * 4).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            chip8.redraw = false;
        }

        thread::sleep(time::Duration::from_millis(1));
    }
}

fn blit(pixels: &[bool; 64 * 32], texture_data: &mut [u8; 64 * 32 * 4]) {
    for i in 0..pixels.len() {
        let offset = i * 4;
        texture_data[offset + 0] = if pixels[i] { 255 } else { 0 };
        texture_data[offset + 1] = if pixels[i] { 255 } else { 0 };
        texture_data[offset + 2] = if pixels[i] { 255 } else { 0 };
        texture_data[offset + 3] = 255;
    }
}
