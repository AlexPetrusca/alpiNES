use std::fs;
use std::fs::File;
use std::io::Read;
use alpiNES::emu::Emulator;
use alpiNES::nes::NES;
use alpiNES::rom::ROM;

use std::time::Duration;
use bitvec::macros::internal::funty::Fundamental;
use rand::Rng;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::Window;
use alpiNES::logln;
use alpiNES::logger::Logger;

const SCALE: f32 = 20.0;

fn load_game(emu: &mut Emulator) {
    let rom = ROM::from_filepath("rom/snake.nes").unwrap();
    emu.load_rom(&rom);
}

fn color(byte: u8) -> Color {
    match byte {
        0 => Color::BLACK,
        1 => Color::WHITE,
        2 | 9 => Color::GREY,
        3 | 10 => Color::RED,
        4 | 11 => Color::GREEN,
        5 | 12 => Color::BLUE,
        6 | 13 => Color::MAGENTA,
        7 | 14 => Color::YELLOW,
        _ => Color::CYAN,
    }
}

fn read_screen_state(nes: &NES, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = nes.mem.read_byte(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}

fn handle_user_input(nes: &mut NES, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                std::process::exit(0);
            },
            Event::KeyDown { keycode: Some(Keycode::W | Keycode::Up), .. } => {
                nes.mem.write_byte(0xff, 0x77);
            },
            Event::KeyDown { keycode: Some(Keycode::S | Keycode::Down), .. } => {
                nes.mem.write_byte(0xff, 0x73);
            },
            Event::KeyDown { keycode: Some(Keycode::A | Keycode::Left), .. } => {
                nes.mem.write_byte(0xff, 0x61);
            },
            Event::KeyDown { keycode: Some(Keycode::D | Keycode::Right), .. } => {
                nes.mem.write_byte(0xff, 0x64);
            }
            _ => {}
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake Game", (32.0 * SCALE) as u32, (32.0 * SCALE) as u32)
        .position_centered()
        .build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();

    let mut emulator = Emulator::new();
    load_game(&mut emulator);

    let mut screen_state = [0 as u8; 32 * 32 * 3];
    let mut rng = rand::thread_rng();

    let mut logger = Logger::new("rom/log/snake.log");
    emulator.run_with_callback(|nes| {
        logln!(logger, "opcode: {}", nes.mem.read_byte(nes.cpu.program_counter));

        handle_user_input(nes, &mut event_pump);
        nes.mem.write_byte(0xfe, rng.gen_range(1..16));

        if read_screen_state(nes, &mut screen_state) {
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        std::thread::sleep(Duration::new(0, 70_000));
    });
}