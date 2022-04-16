// use log::{trace, debug, info, warn, error};

mod chip8;
use chip8::Chip8;
use clap::Parser;

/// CHIP-8 Emulator
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the ROM file to run
    #[clap(required = true)]
    romfile: String,
}

fn main() {
    // Initialise the logger
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    // Set up CHIP-8 and load the ROM
    let mut chip8 = Chip8::new();
    chip8.load_rom(&args.romfile);

    // Emulation loop
    loop {
        chip8.tick();

        if chip8.redraw {
            // draw_screen();
        }
    }
}
