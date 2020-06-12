use crate::ram::Ram;

use rand;
use rand::Rng;

const V_WIDTH: usize = 64;
const V_HEIGHT: usize = 32;
const OPSIZE: usize = 2;

pub struct Cpu {
    // Address register
    i: usize,

    // Program counter
    pc: usize,

    // RAM
    ram: Ram,

    // V registers
    v: [u8; 16],

    // Video RAM
    pub vram: [[u8; V_WIDTH]; V_HEIGHT],

    // Stack
    stack: [usize; 16],

    // Stack pointer
    sp: u8,

    // Delay timer
    dt: u8,

    // Sound timer
    st: u8,

    // Key state
    keys: [bool; 16],

    // Sleep while awaiting key press?
    awaiting_key: bool,

    display_updated: bool,
}

enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            i: 0,
            pc: 0x200,
            ram: Ram::new(),
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0,
            vram: [[0; V_WIDTH]; V_HEIGHT],
            keys: [false; 16],
            awaiting_key: false,
            display_updated: false,
        }
    }

    pub fn execute_cycle(&mut self) {
        let opcode = self.ram.read_word(self.pc);

        // Update the delay timer
        if self.dt > 0 {
            self.dt -= 1;
        }

        self.execute_opcode(opcode);

        if self.display_updated {
            print!("\x1B[0;0H");
            for (_i, &row) in self.vram.iter().enumerate() {
                for (_j, &char) in row.iter().enumerate() {
                    if char > 0 {
                        print!("#");
                    } else {
                        print!(" ");
                    }
                }
                println!("");
            }
            self.display_updated = false;
        }
    }

    pub fn load_cart(&mut self, contents: &Vec<u8>) {
        self.ram.load_block(0x200, contents);
    }
    fn execute_opcode(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F),
        );

        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = (opcode & 0x0FFF) as usize;

        let jump = match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),
            (0x0, _, _, _) => ProgramCounter::Next,
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x2, _, _, _) => self.op_2nnn(nnn),
            (0x3, _, _, _) => self.op_3xnn(x, nn),
            (0x4, _, _, _) => self.op_4xnn(x, nn),
            (0x5, _, _, _) => self.op_5xy0(x, y),
            (0x6, _, _, _) => self.op_6xnn(x, nn),
            (0x7, _, _, _) => self.op_7xnn(x, nn),
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),
            (0x8, _, _, 0x6) => self.op_8x06(x),
            (0x8, _, _, 0x7) => self.op_8x07(x, y),
            (0x8, _, _, 0xE) => self.op_8x0e(x),
            (0x9, _, _, _) => self.op_9xy0(x, y),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),
            (0xC, _, _, _) => self.op_cxnn(x, nn),
            (0xD, _, _, _) => self.op_dxyn(x, y, n),
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),
            (0xF, _, 0x1, 0xe) => self.op_fx1e(x),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),
            _ => panic!("Unknown opcode: {:?}", nibbles),
        };

        match jump {
            ProgramCounter::Jump(address) => self.pc = address,
            ProgramCounter::Next => self.pc += OPSIZE,
            ProgramCounter::Skip => self.pc += OPSIZE * 2,
        };

        // println!("PC: {:?}", nibbles);
    }

    // Clears the screen
    fn op_00e0(&mut self) -> ProgramCounter {
        self.display_updated = true;

        for y in 0..V_HEIGHT {
            for x in 0..V_WIDTH {
                self.vram[y][x] = 0;
            }
        }

        ProgramCounter::Next
    }

    fn op_00ee(&mut self) -> ProgramCounter {
        self.sp -= 1;
        ProgramCounter::Jump(self.stack[self.sp as usize] as usize)
    }

    // Jumps to address NNN.
    fn op_1nnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }

    // Calls subroutine at NNN.
    fn op_2nnn(&mut self, nnn: usize) -> ProgramCounter {
        self.stack[self.sp as usize] = self.pc as usize + OPSIZE;
        self.sp += 1;

        ProgramCounter::Jump(nnn)
    }

    // Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
    fn op_3xnn(&mut self, x: usize, nn: u8) -> ProgramCounter {
        if self.v[x] == nn {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
    fn op_4xnn(&mut self, x: usize, nn: u8) -> ProgramCounter {
        if self.v[x] != nn {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }
    // Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
    fn op_5xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        if self.v[x] == self.v[y] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // Sets VX to NN.
    fn op_6xnn(&mut self, x: usize, nn: u8) -> ProgramCounter {
        self.v[x] = nn;

        ProgramCounter::Next
    }

    // Adds NN to VX. (Carry flag is not changed)
    fn op_7xnn(&mut self, x: usize, nn: u8) -> ProgramCounter {
        let result = self.v[x] as usize + nn as usize;
        self.v[x] = (result & 0xFF) as u8;

        ProgramCounter::Next
    }

    // Sets VX to the value of VY.
    fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] = self.v[y];

        ProgramCounter::Next
    }

    // Sets VX to VX or VY. (Bitwise OR operation)
    fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] |= self.v[y];

        ProgramCounter::Next
    }

    // Sets VX to VX and VY. (Bitwise AND operation)
    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] &= self.v[y];

        ProgramCounter::Next
    }

    // Sets VX to VX xor VY.
    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] ^= self.v[y];

        ProgramCounter::Next
    }

    // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter {
        let result = self.v[x] as u16 + self.v[y] as u16;

        self.v[x] = result as u8;
        self.v[0xF] = if result > 0xFF { 1 } else { 0 };

        ProgramCounter::Next
    }

    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = if self.v[y] > self.v[x] { 0 } else { 1 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);

        ProgramCounter::Next
    }

    // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
    fn op_8x06(&mut self, x: usize) -> ProgramCounter {
        self.v[0xF] = self.v[x] & 0x1;
        self.v[x] >>= 1;

        ProgramCounter::Next
    }

    // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when
    // there isn't.
    fn op_8x07(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = if self.v[x] > self.v[y] { 0 } else { 1 };
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);

        ProgramCounter::Next
    }

    // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
    fn op_8x0e(&mut self, x: usize) -> ProgramCounter {
        self.v[0xF] = self.v[x] >> 7;
        self.v[x] <<= 1;

        ProgramCounter::Next
    }

    // Skips the next instruction if VX doesn't equal VY. (Usually the next
    // instruction is a jump to skip a code block)
    fn op_9xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        if self.v[x] != self.v[y] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // Sets I to the address NNN
    fn op_annn(&mut self, nnn: usize) -> ProgramCounter {
        self.i = nnn;
        ProgramCounter::Next
    }

    // Jumps to the address NNN plus V0.
    fn op_bnnn(&mut self, nnn: usize) -> ProgramCounter {
        self.i = nnn + self.v[0] as usize;
        ProgramCounter::Next
    }

    // Jumps to the address NNN plus V0.
    fn op_cxnn(&mut self, x: usize, nn: u8) -> ProgramCounter {
        let mut rng = rand::thread_rng();
        self.v[x] = rng.gen::<u8>() & nn;

        ProgramCounter::Next
    }

    // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
    // Each row of 8 pixels is read as bit-coded starting from memory location I; I value doesn’t
    // change after the execution of this instruction. As described above, VF is set to 1 if any
    // screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that
    // doesn’t happen
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> ProgramCounter {
        self.v[0xF] = 0;
        self.display_updated = true;

        for byte in 0..n {
            let y = (self.v[y] as usize + byte) % V_HEIGHT;

            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % V_WIDTH;
                let color = (self.ram.read_byte(self.i as usize + byte) >> (7 - bit)) & 1;
                self.v[0xF] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }

        ProgramCounter::Next
    }

    fn op_exa1(&mut self, x: usize) -> ProgramCounter {
        if self.keys[x] {
            ProgramCounter::Next
        } else {
            ProgramCounter::Skip
        }
    }

    // Sets VX to the value of the delay timer
    fn op_fx07(&mut self, x: usize) -> ProgramCounter {
        self.v[x] = self.dt;
        ProgramCounter::Next
    }

    // Sets the delay timer to VX
    fn op_fx15(&mut self, x: usize) -> ProgramCounter {
        self.dt = self.v[x];
        ProgramCounter::Next
    }

    // Sets the sound timer to VX.
    fn op_fx18(&mut self, x: usize) -> ProgramCounter {
        self.st = self.v[x];
        ProgramCounter::Next
    }

    // Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF),
    // and to 0 when there isn't.
    fn op_fx1e(&mut self, x: usize) -> ProgramCounter {
        let result = self.i + self.v[x] as usize;
        self.i = result & 0xFFF as usize;
        self.v[0xF] = if result > 0x0FFF { 1 } else { 0 };

        ProgramCounter::Next
    }

    // Sets I to the location of the sprite for the character in VX. Characters
    // 0-F (in hexadecimal) are represented by a 4x5 font.
    fn op_fx29(&mut self, x: usize) -> ProgramCounter {
        self.i = x * 5;

        ProgramCounter::Next
    }

    // Stores the binary-coded decimal representation of VX, with the most
    // significant of three digits at the address in I, the middle digit at I
    // plus 1, and the least significant digit at I plus 2. (In other words,
    // take the decimal representation of VX, place the hundreds digit in memory
    // at location in I, the tens digit at location I+1, and the ones digit at
    // location I+2.)
    fn op_fx33(&mut self, x: usize) -> ProgramCounter {
        self.ram.write_byte(self.i + 0, self.v[x] / 100);
        self.ram.write_byte(self.i + 1, (self.v[x] % 100) / 10);
        self.ram.write_byte(self.i + 2, self.v[x] % 10);

        ProgramCounter::Next
    }

    // Stores V0 to VX (including VX) in memory starting at address I. The
    // offset from I is increased by 1 for each value written, but I itself is
    // left unmodified.[d]
    fn op_fx55(&mut self, x: usize) -> ProgramCounter {
        for i in 0..(x + 1) {
            self.ram.write_byte(self.i + i, self.v[i]);
        }

        ProgramCounter::Next
    }

    // Fills V0 to VX (including VX) with values from memory starting at address
    // I. The offset from I is increased by 1 for each value written, but I
    // itself is left unmodified.[d]
    fn op_fx65(&mut self, x: usize) -> ProgramCounter {
        for i in 0..(x + 1) {
            self.v[i] = self.ram.read_byte(self.i + i);
        }

        ProgramCounter::Next
    }
}
