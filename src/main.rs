use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};

mod cpu;
mod platform;
mod ram;

use cpu::Cpu;
use platform::window::Window;

fn main() {
    let mut file = File::open("roms/KALEID").expect("File not found");
    let mut contents = Vec::<u8>::new();
    file.read_to_end(&mut contents)
        .expect("Could not load file");

    let mut cpu = Cpu::new();
    let mut window = Window::new();
    cpu.load_cart(&contents);

    let two_millis = time::Duration::from_millis(2);

    while let Ok(keypad) = window.poll(&mut cpu) {
        thread::sleep(two_millis);
        cpu.execute_cycle(&keypad);
    }
}
