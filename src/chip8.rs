#![allow(dead_code)]
#![allow(unused_variables)]

#[path = "stack.rs"] mod stack;
use stack::Stack;

use log::{debug, error, trace, warn};
use std::fs;
use std::fs::File;
use std::io::Read;
use rand::Rng;

const PIXEL_COUNT: usize = 32 * 64 * 3;
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn op_implemented(pc: usize, opcode: u16, instruction: &str, description: &str) {
    debug!("I ({:#04x}) {:04X} | {} - {}", pc - 2, opcode, instruction, description);
}

fn op_unimplemented(pc: usize, opcode: u16, instruction: &str, description: &str) {
    warn!("U ({:#04x}) {:04X} | {} - {}", pc - 2, opcode, instruction, description);
}

pub struct Opcode {
    pub code: u16,
    pub nnn: u16,
    pub nn: u8,
    pub n: usize,
    pub x: usize,
    pub y: usize,
}

impl Opcode {
    pub fn new(code: u16) -> Self {
        Opcode {
            code,
            nnn: code & 0x0FFF,
            nn: (code & 0x00FF) as u8,
            n: (code & 0x000F) as usize,
            x: ((code & 0x0F00) >> 8) as usize,
            y: ((code & 0x00F0) >> 4) as usize,
        }
    }
}

pub struct Chip8 {
    pub pc: usize,
    pub opcode: Opcode,
    pub memory: [u8; 4096],
    pub display: [bool; 64 * 32],
    pub stack: Stack,
    pub reg: [u8; 16],
    pub reg_i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypress: Option<u8>,
    pub pixels: [u8; PIXEL_COUNT],
    pub redraw: bool,
    pub reg_read: Vec<usize>,
    pub reg_write: Vec<usize>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        trace!("Resetting the CPU");
        // Load the font
        let mut memory = [0; 4096];
        for i in 0..FONT.len() {
            memory[i] = FONT[i];
        }
        // Return the Chip8
        Chip8 {
            pc: 0x200,
            memory,
            opcode: Opcode::new(0x0000),
            display: [false; 64 * 32],
            stack: Stack::new(),
            reg: [0; 16],
            reg_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypress: None,
            pixels: [0; PIXEL_COUNT],
            redraw: false,
            reg_read: Vec::new(),
            reg_write: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        trace!("Resetting the CPU");
        self.memory = [0; 4096];
        for i in 0..FONT.len() {
            self.memory[i] = FONT[i];
        }
        self.pc = 0x200;
        self.opcode = Opcode::new(0x0000);
        self.display = [false; 64 * 32];
        self.reg = [0;16];
        self.reg_i = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.keypress = None;
        self.pixels = [0;PIXEL_COUNT];
        self.redraw = false;
        self.reg_read.clear();
        self.reg_write.clear();
    }

    pub fn load_rom(&mut self, filename: &str) {
        trace!("Loading ROM file '{}'", filename);
        let mut file = File::open(&filename).expect("File doesn't exist");
        let metadata = fs::metadata(&filename).expect("Unable to read metadata");
        let filesize = metadata.len() as usize;
        trace!("ROM file size is {} bytes", filesize);
        let start = 0x200;
        let end = start + filesize;
        file.read_exact(&mut self.memory[start..end])
            .expect("Buffer overflow");
    }

    pub fn load_vec(&mut self, vector: Vec<u16>) {
        for i in 0..vector.len() {
            self.memory[(i * 2) + 0x200] = ((vector[i] & 0xFF00) >> 8) as u8;
            self.memory[(i * 2) + 0x200 + 1] = (vector[i] & 0x00FF) as u8;
        }
    }

    pub fn fetch(&mut self) -> Opcode {
        trace!("Fetching the next opcode at {:#04x}", self.pc);
        let left = self.memory[self.pc] as u16;
        let right = self.memory[self.pc + 1] as u16;
        Opcode::new(left << 8 | right)
    }

    pub fn tick(&mut self, keypress: Option<u8>) {
        if !self.reg_read.is_empty() { self.reg_read.clear() };
        if !self.reg_write.is_empty() { self.reg_write.clear() };
        self.opcode = self.fetch();
        self.pc += 2;
        if self.delay_timer > 0 { self.delay_timer -= 1 };
        if self.sound_timer > 0 { self.sound_timer -= 1 };
        self.keypress = keypress;
        self.execute();
    }

    pub fn execute(&mut self) {
        match self.opcode.code & 0xF000 {
            0x0000 => match self.opcode.code & 0x0FFF {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => self.op_0nnn(),
            },
            0x1000 => self.op_1nnn(),
            0x2000 => self.op_2nnn(),
            0x3000 => self.op_3xnn(),
            0x4000 => self.op_4xnn(),
            0x5000 => self.op_5xy0(),
            0x6000 => self.op_6xnn(),
            0x7000 => self.op_7xnn(),
            0x8000 => match self.opcode.code & 0x000F {
                0x0000 => self.op_8xy0(),
                0x0001 => self.op_8xy1(),
                0x0002 => self.op_8xy2(),
                0x0003 => self.op_8xy3(),
                0x0004 => self.op_8xy4(),
                0x0005 => self.op_8xy5(),
                0x0006 => self.op_8xy6(),
                0x0007 => self.op_8xy7(),
                0x000E => self.op_8xye(),
                _ => error!("Unknown opcode {:04X}", self.opcode.code),
            },
            0x9000 => self.op_9xy0(),
            0xA000 => self.op_annn(),
            0xB000 => self.op_bnnn(),
            0xC000 => self.op_cxnn(),
            0xD000 => self.op_dxyn(),
            0xE000 => match self.opcode.code & 0x00FF {
                0x009E => self.op_ex9e(),
                0x00A1 => self.op_exa1(),
                _ => error!("Unknown opcode {:04X}", self.opcode.code),
            },
            0xF000 => match self.opcode.code & 0x00FF {
                0x0007 => self.op_fx07(),
                0x000A => self.op_fx0a(),
                0x0015 => self.op_fx15(),
                0x0018 => self.op_fx18(),
                0x001E => self.op_fx1e(),
                0x0029 => self.op_fx29(),
                0x0033 => self.op_fx33(),
                0x0055 => self.op_fx55(),
                0x0065 => self.op_fx65(),
                _ => error!("Unknown opcode {:04X}", self.opcode.code),
            },
            _ => error!("Unknown opcode {:04X}", self.opcode.code),
        }
    }

    fn op_0nnn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "0NNN", "Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN. Not necessary for most ROMs.");
        self.pc = self.opcode.nnn as usize;
    }
    fn op_00e0(&mut self) {
        op_implemented(self.pc, 0x00E0, "00EE", "Clears the screen.");
        self.display.fill(false);
    }
    fn op_00ee(&mut self) {
        op_implemented(self.pc, 0x00EE, "00EE", "Returns from a subroutine.");
        self.pc = self.stack.pop() as usize;
    }
    fn op_1nnn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "1NNN", "Jumps to address NNN.");
        self.pc = self.opcode.nnn as usize;
    }
    fn op_2nnn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "2NNN", "Calls subroutine at NNN.");
        self.stack.push(self.pc as u16);
        self.pc = self.opcode.nnn as usize;
    }
    fn op_3xnn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "3XNN", "Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)");
        self.reg_read.push(self.opcode.x);
        if self.reg[self.opcode.x] == self.opcode.nn {
            self.pc += 2;
        }
    }
    fn op_4xnn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "4NNN", "Skips the next instruction if VX does not equal NN. (Usually the next instruction is a jump to skip a code block);");
        self.reg_read.push(self.opcode.x);
        if self.reg[self.opcode.x] != self.opcode.nn {
            self.pc += 2;
        }
    }
    fn op_5xy0(&mut self) {
        op_implemented(self.pc, self.opcode.code, "5XY0", "Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block);");
        self.reg_read.push(self.opcode.x);
        self.reg_read.push(self.opcode.y);
        if self.reg[self.opcode.x] == self.reg[self.opcode.y] {
            self.pc += 2;
        }
    }
    fn op_6xnn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "6XNN", "Sets VX to NN.");
        self.reg_write.push(self.opcode.x);
        self.reg[self.opcode.x] = self.opcode.nn;
    }
    fn op_7xnn(&mut self) {
        op_implemented(
            self.pc,
            self.opcode.code,
            "7XNN",
            "Adds NN to VX. (Carry flag is not changed);",
        );
        self.reg_write.push(self.opcode.x);
        self.reg[self.opcode.x] = self.reg[self.opcode.x].wrapping_add(self.opcode.nn);
    }
    fn op_8xy0(&mut self) {
        op_implemented(self.pc, self.opcode.code, "8XY0", "Sets VX to the value of VY.");
        self.reg_write.push(self.opcode.x);
        self.reg_read.push(self.opcode.y);
        self.reg[self.opcode.x] = self.reg[self.opcode.y];
    }
    fn op_8xy1(&mut self) {
        op_unimplemented(
            self.pc,
            self.opcode.code,
            "8XY1",
            "Sets VX to VX or VY. (Bitwise OR operation);",
        );
    }
    fn op_8xy2(&mut self) {
        op_implemented(
            self.pc,
            self.opcode.code,
            "8XY2",
            "Sets VX to VX and VY. (Bitwise AND operation);",
        );
        self.reg_read.push(self.opcode.y);
        self.reg_write.push(self.opcode.x);
        self.reg[self.opcode.x] &= self.reg[self.opcode.y];
    }
    fn op_8xy3(&mut self) {
        op_unimplemented(self.pc, self.opcode.code, "8XY3", "Sets VX to VX xor VY.");
    }
    fn op_8xy4(&mut self) {
        op_implemented(
            self.pc,
            self.opcode.code,
            "8XY4",
            "Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.",
        );
        self.reg_read.push(self.opcode.y);
        self.reg_write.push(self.opcode.x);
        let vx = self.reg[self.opcode.x];
        let vy = self.reg[self.opcode.y];
        let (result, carry) = vx.overflowing_add(vy);
        self.reg[self.opcode.x] = result;
        self.reg[0xF] = carry as u8;
        if carry { self.reg_write.push(0xF) };
    }
    fn op_8xy5(&mut self) {
        op_implemented(self.pc, self.opcode.code, "8XY5", "VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.");
        self.reg_read.push(self.opcode.y);
        self.reg_write.push(self.opcode.x);
        let vx = self.reg[self.opcode.x];
        let vy = self.reg[self.opcode.y];
        let (result, carry) = vx.overflowing_sub(vy);
        self.reg[self.opcode.x] = result;
        self.reg[0xF] = carry as u8;
        if carry { self.reg_write.push(0xF) };
    }
    fn op_8xy6(&mut self) {
        op_unimplemented(
            self.pc,
            self.opcode.code,
            "8XY6",
            "Stores the least significant bit of VX in VF and then shifts VX to the right by 1.",
        );
    }
    fn op_8xy7(&mut self) {
        op_unimplemented(self.pc, self.opcode.code, "8XY7", "Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.");
    }
    fn op_8xye(&mut self) {
        op_unimplemented(
            self.pc,
            self.opcode.code,
            "8XYE",
            "Stores the most significant bit of VX in VF and then shifts VX to the left by 1.",
        );
    }
    fn op_9xy0(&mut self) {
        op_unimplemented(self.pc, self.opcode.code, "9XY0", "Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block);");
    }
    fn op_annn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "ANNN", "Sets I to the address NNN.");
        self.reg_i = self.opcode.nnn;
    }
    fn op_bnnn(&mut self) {
        op_unimplemented(
            self.pc,
            self.opcode.code,
            "BNNN",
            "Jumps to the address NNN plus V0.",
        );
    }
    fn op_cxnn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "CXNN", "Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.");
        self.reg_write.push(self.opcode.x);
        let mut rng = rand::thread_rng();
        self.reg[self.opcode.x] = rng.gen_range(0..=255) & self.opcode.nn;
    }
    fn op_dxyn(&mut self) {
        op_implemented(self.pc, self.opcode.code, "DXYN","Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen");
        self.reg_read.push(self.opcode.y);
        self.reg_read.push(self.opcode.x);
        let vx = self.reg[self.opcode.x] as usize;
        let vy = self.reg[self.opcode.y] as usize;

        for sprite_y in 0..self.opcode.n {
            for sprite_x in 0..8 {
                if self.memory[self.reg_i as usize + sprite_y] << sprite_x & 0b10000000 == 128 {
                    let offset = ((vy + sprite_y) * 64) + (vx + sprite_x);
                    if offset < 64 * 32 {
                        self.reg[0xF] = self.display[offset] as u8; // Set VF
                        self.display[offset] = !self.display[offset]; // Flip pixel
                    }
                }
            }
        }
        self.redraw = true;
    }
    fn op_ex9e(&mut self) {
        op_unimplemented(self.pc, self.opcode.code, "EX9E", "Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block);");
    }
    fn op_exa1(&mut self) {
        op_implemented(self.pc, self.opcode.code, "EXA1", "Skips the next instruction if the key stored in VX is not pressed. (Usually the next instruction is a jump to skip a code block);");
        self.reg_read.push(self.opcode.x);
        match self.keypress {
            Some(key) => {
                if key != self.reg[self.opcode.x] {
                    self.pc += 2
                }
            }
            None => (),
        }
    }
    fn op_fx07(&mut self) {
        op_implemented(
            self.pc,
            self.opcode.code,
            "FX07",
            "Sets VX to the value of the delay timer.",
        );
        self.reg_write.push(self.opcode.x);
        self.reg[self.opcode.x] = self.delay_timer;
    }
    fn op_fx0a(&mut self) {
        op_unimplemented(self.pc, self.opcode.code, "FX0A", "A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event);");
    }
    fn op_fx15(&mut self) {
        op_implemented(self.pc, self.opcode.code, "FX15", "Sets the delay timer to VX.");
        self.reg_read.push(self.opcode.x);
        self.delay_timer = self.reg[self.opcode.x];
    }
    fn op_fx18(&mut self) {
        op_implemented(self.pc, self.opcode.code, "FX18", "Sets the sound timer to VX.");
        self.reg_read.push(self.opcode.x);
        self.sound_timer = self.reg[self.opcode.x];
    }
    fn op_fx1e(&mut self) {
        op_unimplemented(
            self.pc,
            self.opcode.code,
            "FX1E",
            "Adds VX to I. VF is not affected.",
        );
    }
    fn op_fx29(&mut self) {
        op_implemented(self.pc, self.opcode.code, "FX29", "Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.");
        self.reg_read.push(self.opcode.x);
        self.reg_i = 5 * self.reg[self.opcode.x] as u16;
    }
    fn op_fx33(&mut self) {
        op_implemented(self.pc, self.opcode.code, "FX33", "Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.);");
        self.reg_read.push(self.opcode.x);
        let hundreds = self.reg[self.opcode.x] / 100 % 10;
        let tens = self.reg[self.opcode.x] / 10 % 10;
        let ones = self.reg[self.opcode.x] % 10;
        self.memory[self.reg_i as usize] = hundreds;
        self.memory[self.reg_i as usize + 1] = tens;
        self.memory[self.reg_i as usize + 2] = ones;
    }
    fn op_fx55(&mut self) {
        op_unimplemented(self.pc, self.opcode.code, "FX55", "Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.");
    }
    fn op_fx65(&mut self) {
        op_implemented(self.pc, self.opcode.code, "FX65", "Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.");
        self.reg_write = vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15]; // TODO: Find a way to do with programmatically
        for i in 0..=self.opcode.x {
            self.reg[i] = self.memory[self.reg_i as usize + i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset() {
        let chip8 = Chip8::new();
        assert_eq!(chip8.pc, 0x200);
        assert_eq!(chip8.reg_i, 0);
        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);
        assert_eq!(chip8.keypress, None);
    }

    #[test]
    fn test_fetch() {
        let mut chip8 = Chip8::new();
        chip8.pc = 0x004;
        chip8.memory[chip8.pc] = 4;
        chip8.memory[chip8.pc + 1] = 5;
        assert_eq!(chip8.fetch().code, (4 << 8) | 5);
    }

    #[test]
    fn test_op_0nnn() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x0208]);
        chip8.tick(None);
        assert_eq!(chip8.pc, 0x208);
    }

    #[test]
    fn test_op_00ee() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x00ee]);
        chip8.stack.push(0x206);
        assert_eq!(chip8.pc, 0x200);
        chip8.tick(None);
        assert_eq!(chip8.pc, 0x206);
    }

    #[test]
    fn test_op_1nnn() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x1208]);
        assert_eq!(chip8.pc, 0x200);
        chip8.tick(None);
        assert_eq!(chip8.pc, 0x208);
    }

    #[test]
    fn test_op_2nnn() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x2208]);
        assert_eq!(chip8.pc, 0x200);
        chip8.tick(None);
        assert_eq!(chip8.pc, 0x208);
        assert_eq!(chip8.stack.pop(), 0x202)
    }

    #[test]
    fn test_op_3xnn_skip() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x3AFF]);
        chip8.reg[0xA] = 0xFF;
        chip8.tick(None);
        assert_eq!(chip8.pc, 0x204);
    }

    #[test]
    fn test_op_3xnn_no_skip() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x3AFF]);
        chip8.reg[0xA] = 0xF0;
        chip8.tick(None);
        assert_ne!(chip8.pc, 0x204);
    }

    #[test]
    fn test_op_4xnn_skip() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x4AFF]);
        chip8.reg[0xA] = 0xF0;
        chip8.tick(None);
        assert_eq!(chip8.pc, 0x204);
    }

    #[test]
    fn test_op_4xnn_no_skip() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x4AFF]);
        chip8.reg[0xA] = 0xFF;
        chip8.tick(None);
        assert_ne!(chip8.pc, 0x204);
    }

    #[test]
    fn test_op_5xy0_skip() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x5AB0]);
        chip8.reg[0xA] = 0xF0;
        chip8.reg[0xB] = 0xF0;
        chip8.tick(None);
        assert_eq!(chip8.pc, 0x204);
    }

    #[test]
    fn test_op_5xy0_no_skip() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x5AB0]);
        chip8.reg[0xA] = 0x0F;
        chip8.reg[0xB] = 0xF0;
        chip8.tick(None);
        assert_ne!(chip8.pc, 0x204);
    }

    #[test]
    fn test_op_6xnn() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x6A45]);
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 0x45);
    }

    #[test]
    fn test_op_7xnn_wrap() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x7A02]);
        chip8.reg[0xA] = 0xFF;
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 0x01);
        assert_ne!(chip8.reg[0xF], 1);
    }

    #[test]
    fn test_op_7xnn_no_wrap() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x7A10]);
        chip8.reg[0xA] = 0x0F;
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 0x0F + 0x10);
    }

    #[test]
    fn test_op_8xy0() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x8AB0]);
        chip8.reg[0xA] = 7;
        chip8.reg[0xB] = 10;
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 10);
    }

    #[test]
    fn test_op_8xy2() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x8AB2]);
        chip8.reg[0xA] = 0b11111100;
        chip8.reg[0xB] = 0b00111111;
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 0b00111100);
        assert_eq!(chip8.reg[0xB], 0b00111111);
    }

    #[test]
    fn test_op_8xy4_carry() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x8AB4]);
        chip8.reg[0xA] = 255;
        chip8.reg[0xB] = 7;
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 6);
        assert_eq!(chip8.reg[0xB], 7);
        assert_eq!(chip8.reg[0xF], 1)
    }

    #[test]
    fn test_op_8xy4_no_carry() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x8AB4]);
        chip8.reg[0xA] = 2;
        chip8.reg[0xB] = 5;
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 7);
        assert_eq!(chip8.reg[0xB], 5);
        assert_eq!(chip8.reg[0xF], 0)
    }

    #[test]
    fn test_op_8xy5_borrow() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x8AB5]);
        chip8.reg[0xA] = 0;
        chip8.reg[0xB] = 7;
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 249);
        assert_eq!(chip8.reg[0xB], 7);
        assert_eq!(chip8.reg[0xF], 1)
    }

    #[test]
    fn test_op_8xy5_no_borrow() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x8AB5]);
        chip8.reg[0xA] = 7;
        chip8.reg[0xB] = 5;
        chip8.tick(None);
        assert_eq!(chip8.reg[0xA], 2);
        assert_eq!(chip8.reg[0xB], 5);
        assert_eq!(chip8.reg[0xF], 0)
    }

    #[test]
    fn test_op_annn() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0xa123]);
        chip8.tick(None);
        assert_eq!(chip8.reg_i, 0x123);
    }

    #[test]
    fn test_op_dxyn() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0xd003, 0xd003]);
        chip8.reg_i = 0;
        chip8.tick(None);
        assert!(chip8.display[0]); // Drew white at 0x0
        assert_eq!(chip8.reg[0xF], 0); // Bit flipped, VF set
        chip8.tick(None);
        assert!(!chip8.display[0]); // Drew black at 0x0
        assert_eq!(chip8.reg[0xF], 1); // Bit flipped, VF set
    }

    #[test]
    fn test_op_fx07() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0xF207]);
        chip8.tick(None);
        assert_eq!(chip8.reg[2], chip8.delay_timer);
    }

    #[test]
    fn test_op_fx15() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0xFA15]);
        chip8.reg[0xA] = 57;
        chip8.tick(None);
        assert_eq!(chip8.delay_timer, 57);
    }

    #[test]
    fn test_op_fx18() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0xFB18]);
        chip8.reg[0xB] = 53;
        chip8.tick(None);
        assert_eq!(chip8.sound_timer, 53);
    }

    #[test]
    fn test_op_fx29() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0xFA29]);
        chip8.reg[0xA] = 0xE;
        chip8.tick(None);
        assert_eq!(chip8.reg_i, 70); // 0xE * 5
    }

    #[test]
    fn test_op_fx65() {
        let mut chip8 = Chip8::new();
        chip8.memory.fill(0xAA);
        chip8.load_vec(vec![0xF265]);
        chip8.tick(None);
        assert_eq!(chip8.reg[0], 0xAA);
        assert_eq!(chip8.reg[1], 0xAA);
        assert_eq!(chip8.reg[2], 0xAA);
    }
}
