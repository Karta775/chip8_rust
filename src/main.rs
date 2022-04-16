// use log::{trace, debug, info, warn, error};

mod chip8;
use chip8::Chip8;

fn main() {
    env_logger::init();

    let mut chip8 = Chip8::new();
    chip8.load_rom("pong.ch8");

    loop {
        chip8.tick();

        if chip8.redraw {
            // draw_screen();
        }
    }
}
