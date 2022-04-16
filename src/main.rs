use log::info;

mod chip8;
use chip8::Chip8;
use clap::Parser;
use std::time;

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

    // Set up timer
    let now = time::Instant::now();
    let mut old_time = now.elapsed().as_secs();
    let mut ops_per_sec = 0;

    // Emulation loop
    loop {
        // Compute and print the instructions per second TODO: Build into UI (ImGui?)
        if now.elapsed().as_secs() == old_time {
            ops_per_sec += 1;
        } else {
            info!("{} instructions per second", ops_per_sec);
            old_time = now.elapsed().as_secs();
            ops_per_sec = 0;
        }

        // Fetch, decode, execute
        chip8.tick();

        // Draw graphics on the screen
        if chip8.redraw {
            // draw_screen();
        }
    }
}
