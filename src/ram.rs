pub struct Ram {
    block: [u8; 4096],
}

impl Ram {
    pub fn new() -> Ram {
        let mut ram = Ram { block: [0; 0x1000] };

        let sprites: [u8; 80] = [
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

        ram.load_block(0x0000, &sprites);

        ram
    }

    pub fn load_block(&mut self, offset: usize, block: &[u8]) {
        for (i, &byte) in block.iter().enumerate() {
            let addr = i + offset;

            if addr < 0x1000 {
                self.block[addr] = byte;
            } else {
                break;
            }
        }
    }

    pub fn read_word(&self, position: usize) -> u16 {
        (self.block[position] as u16) << 8 | (self.block[position + 1] as u16)
    }

    pub fn read_byte(&self, position: usize) -> u8 {
        self.block[position]
    }

    pub fn write_byte(&mut self, position: usize, byte: u8) {
        self.block[position] = byte;
    }
}
