mod chip8;
mod app;

use std::fmt::format;
use macroquad::prelude::*;
use clap::Parser;
use egui::{Slider, Ui};
use egui::Color32;
use egui::RichText;
use chip8::Chip8;
use app::App;
use std::time::Duration;
use std::{thread, time};

/// CHIP-8 Emulator
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the ROM file to run
    #[clap(required = true)]
    romfile: String,
}

pub struct Environment {

}

fn window_conf() -> Conf {
    Conf {
        window_title: "CHIP-8".to_owned(),
        high_dpi: true,
        window_resizable: true,
        window_width: 960,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initialise the logger
    env_logger::init();

    // Set up CHIP-8 and load the ROM
    let mut keypress: Option<u8> = None;
    let mut app = App::new();
    app.chip8.load_rom(&args.romfile);

    // Set up texture for macroquad
    let mut texture = pixels_to_texture2d(&app.chip8.display, &app.fg_color, &app.bg_color);
    texture.set_filter(FilterMode::Nearest);

    'running: loop {
        egui_macroquad::ui(|egui_ctx| {
            setup_custom_fonts(&egui_ctx);
            app.show_main_menubar(&egui_ctx);
            app.show_general_state(&egui_ctx);
            app.show_controls(&egui_ctx);
        });

        // If not paused or paused but step requested
        if !app.pause_execution || (app.pause_execution && app.step) {
            if !app.pause_execution { // Execute normally
                for i in 0..app.speed {
                    app.chip8.tick(keypress);
                    app.ops_per_sec += 1;
                }
            } else { // Step requested
                app.chip8.tick(keypress);
            }
            if app.chip8.redraw {
                texture = pixels_to_texture2d(&app.chip8.display, &app.fg_color, &app.bg_color);
                app.chip8.redraw = false;
                app.draw_per_sec += 1;
            }
            app.step = false;
        }

        // Render everything
        clear_background(BLACK);
        set_camera(&Camera2D {
            zoom: vec2(26.0 / screen_width(), 26.0 / screen_height()),
            target: vec2(32., 16.),
            ..Default::default()
        });
        draw_rectangle(-1., -1., 66., 34., GRAY);
        draw_texture_ex(texture,
                        0.0,
                        0.0,
                        WHITE,
                        DrawTextureParams{
                            dest_size: None,
                            source: None,
                            rotation: 0.0,
                            flip_x: false,
                            flip_y: true,
                            pivot: None
                        }
        );
        egui_macroquad::draw();
        next_frame().await
    }
}

fn debug_label(ui: &mut Ui, title: &str, body: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).color(color));
        ui.label(body);
    });
}

fn pixels_to_texture2d(pixels: &[bool; 64 * 32], fg_color: &[f32;3], bg_color: &[f32;3]) -> Texture2D {
    let mut bytes: Vec<u8> = Vec::from([0;8192]);
    for i in 0..pixels.len() {
        let offset = i * 4;
        bytes[offset + 0] = if pixels[i] { (fg_color[0] * 255.) as u8 } else { (bg_color[0] * 255.) as u8 };
        bytes[offset + 1] = if pixels[i] { (fg_color[1] * 255.) as u8 } else { (bg_color[1] * 255.) as u8 };
        bytes[offset + 2] = if pixels[i] { (fg_color[2] * 255.) as u8 } else { (bg_color[2] * 255.) as u8 };
        bytes[offset + 3] = 255;
    }
    let texture = Texture2D::from_rgba8(64, 32, &bytes);
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "JetBrains Mono".to_owned(),
        egui::FontData::from_static(include_bytes!("../JetBrainsMono-Regular.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "JetBrains Mono".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("JetBrains Mono".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}