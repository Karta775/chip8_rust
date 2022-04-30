use std::time;
use std::time::Instant;
use macroquad::prelude::*;
use egui::{Context, Slider, Ui};
use egui::Color32;
use egui::RichText;
use crate::Chip8;
use rfd::FileDialog;
use crate::miniquad::date::now;

pub struct App {
    pub chip8: Chip8,
    pub pause_execution: bool,
    pub step: bool,
    pub fg_color: [f32;3],
    pub bg_color: [f32;3],
    bold_text_color: Color32,
    now: Instant,
    old_time: u64,
    pub ops_per_sec: u32,
    pub draw_per_sec: u32,
    ops_last_sec: u32,
    draw_last_sec: u32,
    pub speed: u32,
}

impl App {
    pub fn new() -> Self {
        let now = time::Instant::now();
        App {
            chip8: Chip8::new(),
            pause_execution: false,
            step: false,
            fg_color: [1.;3],
            bg_color: [0.;3],
            bold_text_color: Color32::from_rgb(110, 255, 110),
            now,
            old_time: now.elapsed().as_secs(),
            ops_per_sec: 0,
            draw_per_sec: 0,
            ops_last_sec: 0,
            draw_last_sec: 0,
            speed: 6,
        }
    }

    pub fn calculate_ops_and_draws(&mut self) {
        // Reset the per second counters
        if self.now.elapsed().as_secs() != self.old_time {
            self.old_time = self.now.elapsed().as_secs();
            self.ops_last_sec = self.ops_per_sec;
            self.draw_last_sec = self.draw_per_sec;
            self.ops_per_sec = 0;
            self.draw_per_sec = 0;
        }
    }

    pub fn label_bold(&mut self, text: &str, ui: &mut Ui) {
        ui.label(RichText::new(text).color(self.bold_text_color));
    }

    pub fn show_main_menubar(&mut self, egui_ctx: &Context) {
        //pub fn show_main_menubar(&mut self, egui_ctx: &Context, chip8: &mut Chip8) {
        egui::TopBottomPanel::top("menu_bar").show(egui_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load ROM").clicked() {
                        let files = FileDialog::new()
                            .add_filter("CHIP-8 ROM", &["ch8"])
                            .set_directory("/")
                            .pick_file();
                        match files {
                            Some(path) => {
                                let rom = path.into_os_string().into_string().unwrap();
                                self.chip8.reset();
                                self.chip8.load_rom(&rom);
                            },
                            None => ()
                        }
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory().reset_areas();
                        ui.close_menu();
                    }
                    if ui
                        .button("Reset egui memory")
                        .on_hover_text("Forget scroll, positions, sizes etc")
                        .clicked()
                    {
                        *ui.ctx().memory() = Default::default();
                        ui.close_menu();
                    }
                });
            });
        });
    }

    pub fn show_general_state(&mut self, egui_ctx: &Context) {
        // pub fn show_general_state(&mut self, egui_ctx: &Context, chip8: &Chip8, ops_last_sec: i32, draw_last_sec: i32) {
        egui::Window::new("General State").show(egui_ctx, |ui| {
            ui.set_max_width(190.);
            self.label_bold("CPU Info:", ui);
            ui.horizontal_wrapped(|ui| {
                self.label_bold("PC:", ui);
                ui.label(format!("{:02X} ", self.chip8.pc));
                self.label_bold("OP:", ui);
                ui.label(format!("{:02X} ", self.chip8.opcode.code));
                self.label_bold("IR:", ui);
                ui.label(format!("{:02X} ", self.chip8.reg_i));
            });
            ui.separator();
            if !self.chip8.stack.is_empty() {
                ui.label(format!("Stack: {:#04x}", self.chip8.stack.top()));
            } else {
                ui.label("Stack: empty");
            }
            match self.chip8.keypress {
                Some(x) => ui.label(format!("Keypress: {:#}", x)),
                None => ui.label("Keypress: none"),
            };
            ui.label(format!("Delay timer: {}", self.chip8.delay_timer));
            ui.label(format!("Sound timer: {}", self.chip8.sound_timer));
            ui.label(format!("Instruction/s: {}", self.ops_last_sec));
            ui.label(format!("Redraw/s: {}", self.draw_last_sec));
            ui.separator();
            ui.label(RichText::new("Registers:").color(self.bold_text_color));
            ui.horizontal_wrapped(|ui| { // TODO: Do this programmatically
                ui.label(RichText::new("0:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x0]));
                ui.label(RichText::new("1:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x1]));
                ui.label(RichText::new("2:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x2]));
                ui.label(RichText::new("3:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x3]));
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("4:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x4]));
                ui.label(RichText::new("5:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x5]));
                ui.label(RichText::new("6:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x6]));
                ui.label(RichText::new("7:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x7]));
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("8:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x8]));
                ui.label(RichText::new("9:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0x9]));
                ui.label(RichText::new("A:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0xA]));
                ui.label(RichText::new("B:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0xB]));
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("C:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0xC]));
                ui.label(RichText::new("D:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0xD]));
                ui.label(RichText::new("E:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0xE]));
                ui.label(RichText::new("F:").color(self.bold_text_color));
                ui.label(format!("{:02X} ", self.chip8.reg[0xF]));
            });
        });
    }

    pub fn show_controls(&mut self, egui_ctx: &Context) {
        //pub fn show_controls(&mut self, egui_ctx: &Context, chip8: &mut Chip8, speed: &mut i32, pause_execution: &mut bool, step: &mut bool, fg_color: &mut [f32;3], bg_color: &mut [f32;3]) {
        egui::Window::new("Control").show(egui_ctx, |ui| {
            ui.set_max_width(190.);
            ui.label(RichText::new("Execution:").color(self.bold_text_color));
            ui.add(
                Slider::new(&mut self.speed, 1..=30)
                    .logarithmic(false)
                    .text("Speed"),
            );
            ui.horizontal(|ui| {
                if ui.button("Toggle execution").clicked() {
                    self.pause_execution = !self.pause_execution;
                }
                if ui.button("Step").clicked() {
                    self.step = true;
                }
            });

            ui.separator();
            ui.label(RichText::new("Display Color:").color(self.bold_text_color));
            ui.horizontal(|ui| {
                ui.label("FG:");
                if ui.color_edit_button_rgb(&mut self.fg_color).changed() {
                    self.chip8.redraw = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("BG:");
                if ui.color_edit_button_rgb(&mut self.bg_color).changed() {
                    self.chip8.redraw = true;
                }
            });
        });
    }
}
