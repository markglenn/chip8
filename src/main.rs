use std::fs::File;
use std::io::prelude::*;

mod cpu;
mod ram;

use cpu::Cpu;

fn main() {
    let mut file = File::open("BC_test.ch8").expect("File not found");
    let mut contents = Vec::<u8>::new();
    file.read_to_end(&mut contents)
        .expect("Could not load file");

    let mut cpu = Cpu::new();
    cpu.load_cart(&contents);

    for _i in 0..300 {
        cpu.execute_cycle();
    }

    for (_i, &row) in cpu.vram.iter().enumerate() {
        for (_j, &char) in row.iter().enumerate() {
            if char > 0 {
                print!("#");
            } else {
                print!(" ");
            }
        }

        println!("");
    }
}
