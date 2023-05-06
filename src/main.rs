use std::thread::sleep;
use std::time::{Duration, Instant};
use rand::Rng;
use sdl2::audio::AudioSpecDesired;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;

use alpines::emu::Emulator;
use alpines::nes::NES;
use alpines::nes::cpu::CPU;
use alpines::nes::io::frame::Frame;
use alpines::nes::ppu::PPU;
use alpines::util::rom::ROM;
use alpines::util::logger::Logger;
use alpines::util::bitvec::BitVector;
use alpines::logln;

// snake - 6502 CPU game

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

fn read_screen_state(nes: &mut NES, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x200..0x600 {
        let color_idx = nes.cpu.memory.read_byte(i as u16);
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
                nes.cpu.memory.write_byte(0xff, 0x77);
            },
            Event::KeyDown { keycode: Some(Keycode::S | Keycode::Down), .. } => {
                nes.cpu.memory.write_byte(0xff, 0x73);
            },
            Event::KeyDown { keycode: Some(Keycode::A | Keycode::Left), .. } => {
                nes.cpu.memory.write_byte(0xff, 0x61);
            },
            Event::KeyDown { keycode: Some(Keycode::D | Keycode::Right), .. } => {
                nes.cpu.memory.write_byte(0xff, 0x64);
            }
            _ => {}
        }
    }
}

fn run_snake() {
    const SCALE: f32 = 20.0;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake", (32.0 * SCALE) as u32, (32.0 * SCALE) as u32)
        .position_centered()
        .build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();

    let cartridge_path = "rom/test/cpu/snake.nes";
    let mut emulator = Emulator::new();
    emulator.load_rom(&ROM::from_filepath(cartridge_path).unwrap());

    let mut screen_state = [0 as u8; 32 * 32 * 3];
    let mut rng = rand::thread_rng();

    emulator.run_with_callback(|nes| {
        handle_user_input(nes, &mut event_pump);
        nes.cpu.memory.write_byte(0xfe, rng.gen_range(1..16));

        if read_screen_state(nes, &mut screen_state) {
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        sleep(Duration::new(0, 70_000));
    });
}

// chrdump - chr rom dump of pacman for the nes

fn render_tile(chr_rom: &Vec<u8>, bank: usize, tile_n: usize, frame: &mut Frame) {
    let tile_addr = 0x1000 * bank + 16 * tile_n;
    let tile = &chr_rom[tile_addr..(tile_addr + 16)];
    for y in 0..8 {
        let mut high_byte = tile[y];
        let mut low_byte = tile[y + 8];
        for x in (0..8).rev() {
            let value = (1 & high_byte) << 1 | (1 & low_byte);
            let rgb = match value {
                0 => NES::SYSTEM_PALLETE[0x01],
                1 => NES::SYSTEM_PALLETE[0x23],
                2 => NES::SYSTEM_PALLETE[0x27],
                3 => NES::SYSTEM_PALLETE[0x30],
                _ => panic!("chr_rom value out of range: {}", value),
            };
            const TILE_SIZE: usize = 8;
            const PADDING: usize = 1;
            const BOX_SIZE: usize = (TILE_SIZE + 2 * PADDING);
            const TILES_PER_ROW: usize = Frame::WIDTH / BOX_SIZE;
            const TILES_PER_COL_BANK: usize = 256 / TILES_PER_ROW + (256 % TILES_PER_ROW > 0) as usize;
            const MARGIN: usize = (Frame::WIDTH - BOX_SIZE * TILES_PER_ROW) / 2;
            let tile_x = x + BOX_SIZE * (tile_n % TILES_PER_ROW) + PADDING + MARGIN;
            let tile_y = y + BOX_SIZE * (tile_n / TILES_PER_ROW) + PADDING + MARGIN
                + (TILES_PER_COL_BANK + 1) * BOX_SIZE * bank;
            frame.set_pixel(tile_x, tile_y, rgb);
            high_byte = high_byte >> 1;
            low_byte = low_byte >> 1;
        }
    }
}

fn run_chrdump(filepath: &str) {
    const SCALE: f32 = 3.0;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("alpiNES - CHR Dump", (SCALE * Frame::WIDTH as f32) as u32, (SCALE * Frame::HEIGHT as f32) as u32)
        .position_centered()
        .build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, Frame::WIDTH as u32, Frame::HEIGHT as u32).unwrap();

    let mut emulator = Emulator::new();
    let rom = ROM::from_filepath(filepath).unwrap();
    emulator.load_rom(&rom);

    let mut tile_frame = Frame::new();
    for i in 0..256 {
        render_tile(&rom.chr_rom, 0, i, &mut tile_frame);
        render_tile(&rom.chr_rom, 1, i, &mut tile_frame);
    }

    texture.update(None, &tile_frame.data, Frame::WIDTH * 3).unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    std::process::exit(0)
                },
                _ => {
                    // do nothing
                }
            }
        }
    }
}

// run nes game

fn run_game(filepath: &str) {
    let mut emu = Emulator::new();
    let rom = ROM::from_filepath(filepath).unwrap();
    emu.run_rom(&rom);
}

// todo: PPU should own Frame
//  - Reset frame on VBlank
//  - Draw background sprites on VBlank
//  - Draw background w/scroll + screen-split on visible scanlines
//  - Draw foreground sprites on NMI (right before render)

// todo: test audio with different games
//  - pacman: nothing sounds right
//  - duck hunt: is broken (also visually broken)
//  - pinball: shouldn't have audio playing during demo
//  - ice climber: breaking blocks (noise) doesn't sound right
//  - balloon fight: shouldn't have any audio during title screen and credits
//  - donkey kong: footsteps and jumps dont sound right

// todo: continue mapper2 debugging
//  - contra: only left half of the background shows up; only top half of sprites show up
//  - castlevania: some sprites don't show up properly
//  - top gun: background doesnt render

// todo: continue mapper3 debugging
//  - friday the 13th + tetris + qbert: background tiles look messed up
//  - friday the 13th: completely visually broken
//  - arkistras ring: background doesn't scroll when moving
//  - solomons key: doesn't play whatsoever

fn main() {
    // run_snake();
    // run_chrdump("rom/mapper0/duck_hunt.nes");
    // run_game("rom/test/cpu/nestest.nes");
    // run_game("rom/test/ppu/nes15.nes");
    // run_game("rom/test/apu/sndtest.nes");

    // run_game("rom/mapper0/duck_hunt.nes");
    // run_game("rom/mapper1/legend_of_zelda.nes"); // todo: impl
    // run_game("rom/mapper2/contra.nes");
    // run_game("rom/mapper3/arkistas_ring.nes");
    // run_game("rom/mapper4/super_mario_bros_3.nes"); // todo: impl
    // run_game("rom/mapper5/castlevania_3.nes"); // todo: impl
}
