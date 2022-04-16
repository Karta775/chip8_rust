#![allow(dead_code)]
#![allow(unused_variables)]

use log::{debug, error, info, warn};
use std::fs::File;
use std::io::Read;
use std::fs;

const PIXEL_COUNT: usize = 32 * 64 * 3;

fn op_info(pc: usize, opcode: u16, instruction: &str, description: &str) {
    info!("({:#04x}) {:04X} | {} - {}", pc - 2, opcode, instruction, description);
}

fn op_unimplemented(pc: usize, opcode: u16, instruction: &str, description: &str) {
    warn!("UNIMPLEMENTED: ({:#04x}) {:04X} | {} - {}", pc - 2, opcode, instruction, description);
}

fn unwrap_nnn(opcode: u16) -> u16 {
    opcode & 0x0FFF
}

pub struct Chip8 {
    pub pc: usize,
    pub memory: [u8; 4096],
    reg: [u8; 16],
    reg_i: u16,
    delay_timer: u8,
    sound_timer: u8,
    key_press: u8,
    pixels: [u8; PIXEL_COUNT],
    pub redraw: bool,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        debug!("Resetting the CPU");
        Chip8 {
            pc: 0x200,
            memory: [0; 4096],
            reg: [0; 16],
            reg_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            key_press: 0x0,
            pixels: [0; PIXEL_COUNT],
            redraw: false
        }
    }

    pub fn load_rom(&mut self, filename: &str) {
        debug!("Loading ROM file '{}'", filename);
        let mut file = File::open(&filename).expect("File doesn't exist");
        let metadata = fs::metadata(&filename).expect("Unable to read metadata");
        let filesize = metadata.len() as usize;
        debug!("ROM file size is {} bytes", filesize);
        let start = 0x200;
        let end = start + filesize;
        file.read_exact(&mut self.memory[start..end]).expect("Buffer overflow");
    }

    pub fn load_vec(&mut self, vector: Vec<u16>) {
        for i in 0..vector.len() {
            self.memory[(i * 2) + 0x200] = ((vector[i] & 0xFF00) >> 8) as u8;
            self.memory[(i * 2) + 0x200 + 1] = (vector[i] & 0x00FF) as u8;
        }
    }

    pub fn fetch(&mut self) -> u16 {
        debug!("Fetching the next opcode at {:#04x}", self.pc);
        let left = self.memory[self.pc] as u16;
        let right = self.memory[self.pc + 1] as u16;
        left << 8 | right
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();
        self.pc += 2;
        self.execute(opcode);
    }

    pub fn execute(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x0FFF {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => self.op_0nnn(opcode)
            }
            0x1000 => self.op_1nnn(opcode),
            0x2000 => self.op_2nnn(opcode),
            0x3000 => self.op_3xnn(opcode),
            0x4000 => self.op_4xnn(opcode),
            0x5000 => self.op_5xy0(opcode),
            0x6000 => self.op_6xnn(opcode),
            0x7000 => self.op_7xnn(opcode),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.op_8xy0(opcode),
                0x0001 => self.op_8xy1(opcode),
                0x0002 => self.op_8xy2(opcode),
                0x0003 => self.op_8xy3(opcode),
                0x0004 => self.op_8xy4(opcode),
                0x0005 => self.op_8xy5(opcode),
                0x0006 => self.op_8xy6(opcode),
                0x0007 => self.op_8xy7(opcode),
                0x000E => self.op_8xye(opcode),
                _ => error!("Unknown opcode {:04X}", opcode)
            }
            0x9000 => self.op_9xy0(opcode),
            0xA000 => self.op_annn(opcode),
            0xB000 => self.op_bnnn(opcode),
            0xC000 => self.op_cxnn(opcode),
            0xD000 => self.op_dxyn(opcode),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(opcode),
                0x00A1 => self.op_exa1(opcode),
                _ => error!("Unknown opcode {:04X}", opcode)
            }
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(opcode),
                0x000A => self.op_fx0a(opcode),
                0x0015 => self.op_fx15(opcode),
                0x0018 => self.op_fx18(opcode),
                0x001E => self.op_fx1e(opcode),
                0x0029 => self.op_fx29(opcode),
                0x0033 => self.op_fx33(opcode),
                0x0055 => self.op_fx55(opcode),
                0x0065 => self.op_fx65(opcode),
                _ => error!("Unknown opcode {:04X}", opcode)
            }
            _ => error!("Unknown opcode {:04X}", opcode)
        }
    }

    fn op_0nnn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "0NNN", "Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN. Not necessary for most ROMs.");
    }
    fn op_00e0(&mut self) {
        op_unimplemented(self.pc, 0x00E0, "00EE", "Clears the screen.");
    }
    fn op_00ee(&mut self) {
        op_unimplemented(self.pc, 0x00EE, "00EE", "Returns from a subroutine.");
    }
    fn op_1nnn(&mut self, opcode: u16) {
        op_info(self.pc, opcode, "1NNN", "Jumps to address NNN.");
        self.pc = unwrap_nnn(opcode) as usize;
    }
    fn op_2nnn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "2NNN", "Calls subroutine at NNN.");
    }
    fn op_3xnn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "3XNN", "Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)");
    }
    fn op_4xnn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "4NNN", "Skips the next instruction if VX does not equal NN. (Usually the next instruction is a jump to skip a code block);");
    }
    fn op_5xy0(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "5XY0", "Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block);");
    }
    fn op_6xnn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "6XNN", "Sets VX to NN.");
    }
    fn op_7xnn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "7XNN", "Adds NN to VX. (Carry flag is not changed);");
    }
    fn op_8xy0(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XY0", "Sets VX to the value of VY.");
    }
    fn op_8xy1(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XY1", "Sets VX to VX or VY. (Bitwise OR operation);");
    }
    fn op_8xy2(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XY2", "Sets VX to VX and VY. (Bitwise AND operation);");
    }
    fn op_8xy3(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XY3", "Sets VX to VX xor VY.");
    }
    fn op_8xy4(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XY4", "Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.");
    }
    fn op_8xy5(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XY5", "VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.");
    }
    fn op_8xy6(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XY6", "Stores the least significant bit of VX in VF and then shifts VX to the right by 1.");
    }
    fn op_8xy7(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XY7", "Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.");
    }
    fn op_8xye(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "8XYE", "Stores the most significant bit of VX in VF and then shifts VX to the left by 1.");
    }
    fn op_9xy0(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "9XY0", "Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block);");
    }
    fn op_annn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "ANNN", "Sets I to the address NNN.");
    }
    fn op_bnnn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "BNNN", "Jumps to the address NNN plus V0.");
    }
    fn op_cxnn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "CXNN", "Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.");
    }
    fn op_dxyn(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "DXYN","Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen");
    }
    fn op_ex9e(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "EX9E", "Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block);");
    }
    fn op_exa1(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "EXA1", "Skips the next instruction if the key stored in VX is not pressed. (Usually the next instruction is a jump to skip a code block);");
    }
    fn op_fx07(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX07", "Sets VX to the value of the delay timer.");
    }
    fn op_fx0a(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX0A", "A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event);");
    }
    fn op_fx15(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX15", "Sets the delay timer to VX.");
    }
    fn op_fx18(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX18", "Sets the sound timer to VX.");
    }
    fn op_fx1e(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX1E", "Adds VX to I. VF is not affected.");
    }
    fn op_fx29(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX29", "Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.");
    }
    fn op_fx33(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX33", "Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.);");
    }
    fn op_fx55(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX55", "Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.");
    }
    fn op_fx65(&mut self, opcode: u16) {
        op_unimplemented(self.pc, opcode, "FX65", "Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.");
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
        assert_eq!(chip8.key_press, 0x0);
    }

    #[test]
    fn test_fetch() {
        let mut chip8 = Chip8::new();
        chip8.pc = 0x004;
        chip8.memory[chip8.pc] = 4;
        chip8.memory[chip8.pc+1] = 5;
        assert_eq!(chip8.fetch(), (4 << 8) | 5);
    }

    #[test]
    fn test_op_1nnn() {
        let mut chip8 = Chip8::new();
        chip8.load_vec(vec![0x0000, 0x1200, 0x0000]);
        assert_eq!(chip8.fetch(), 0x0000);
        chip8.tick();
        assert_eq!(chip8.fetch(), 0x1200);
        chip8.tick();
        assert_eq!(chip8.fetch(), 0x0000);
        chip8.tick();
        assert_eq!(chip8.fetch(), 0x1200);
    }
}