# chip8_rust

This is a CHIP-8 emulator written in Rust as a way of learning Rust & emulation concepts.

### TODO
* [ ] Move CHIP-8 unit testing to a separate module (?)
* [ ] Implement all 36 CPU instructions
* [ ] Add graphics
* [ ] Add sound

### Build instructions
```shell
git clone https://github.com/Karta775/chip8_rust
cd chip8_rust
cargo build
```

### Usage
```shell
cargo test # For unit testing
cargo run romfile.ch8 # To run normally
RUST_LOG=debug cargo run romfile.ch8 # To debug
```