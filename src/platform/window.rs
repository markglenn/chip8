use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window as SDLWindow;

use crate::Cpu;

pub struct Window {
    canvas: Canvas<SDLWindow>,
    events: sdl2::EventPump,
}

const SCALE_FACTOR: u32 = 20;
const SCREEN_WIDTH: u32 = (64 as u32) * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = (32 as u32) * SCALE_FACTOR;

impl Window {
    pub fn new() -> Window {
        let context = sdl2::init().unwrap();
        let video = context.video().unwrap();

        let mut canvas = video
            .window("Chip8", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Window {
            canvas: canvas,
            events: context.event_pump().unwrap(),
        }
    }

    pub fn poll(&mut self, cpu: &mut Cpu) -> Result<[bool; 16], ()> {
        if cpu.display_updated {
            self.draw(&cpu.vram);
        }

        for event in self.events.poll_iter() {
            if let Event::Quit { .. } = event {
                return Err(());
            };
        }

        let keys: Vec<Keycode> = self
            .events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let mut keypad = [false; 16];

        for key in keys {
            let index = match key {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(0x4),
                Keycode::W => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
                _ => None,
            };

            if let Some(i) = index {
                keypad[i] = true;
            }
        }

        Ok(keypad)
    }

    pub fn draw(&mut self, pixels: &[[u8; 64]; 32]) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = (x as u32) * SCALE_FACTOR;
                let y = (y as u32) * SCALE_FACTOR;

                self.canvas.set_draw_color(color(col));
                let _ = self.canvas.fill_rect(Rect::new(
                    x as i32,
                    y as i32,
                    SCALE_FACTOR,
                    SCALE_FACTOR,
                ));
            }
        }
        self.canvas.present();
    }
}

fn color(value: u8) -> pixels::Color {
    if value == 0 {
        pixels::Color::RGB(0, 0, 0)
    } else {
        pixels::Color::RGB(0, 250, 0)
    }
}
