use std::path::Path;
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
use alpines::util::logger::Logger;
use alpines::util::bitvec::BitVector;
use alpines::logln;
use alpines::nes::rom::ROM;

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

    let mut emulator = Emulator::new();
    emulator.load_rom(&ROM::from_path(Path::new("rom/test/cpu/snake.nes")).unwrap());

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
                0 => (0, 0, 0),
                1 => (170, 170, 170),
                2 => (255, 255, 255),
                3 => (85, 85, 85),
                _ => panic!("chr_rom value out of range: {}", value),
            };
            const TILE_SIZE: usize = 8;
            const PADDING: usize = 1;
            const BOX_SIZE: usize = (TILE_SIZE + 2 * PADDING);
            const TILES_PER_ROW: usize = Frame::WIDTH / BOX_SIZE;
            const TILES_PER_COL_BANK: usize = 256 / TILES_PER_ROW + (256 % TILES_PER_ROW > 0) as usize;
            const MARGIN: usize = (Frame::WIDTH - BOX_SIZE * TILES_PER_ROW) / 2;
            let bank_offset: usize = (TILES_PER_COL_BANK + 1) * BOX_SIZE * (bank % 2);
            let tile_x = x + BOX_SIZE * (tile_n % TILES_PER_ROW) + PADDING + MARGIN;
            let tile_y = y + BOX_SIZE * (tile_n / TILES_PER_ROW) + PADDING + MARGIN + bank_offset;
            frame.set_background_color(tile_x, tile_y, rgb);
            high_byte = high_byte >> 1;
            low_byte = low_byte >> 1;
        }
    }
}

fn run_chrdump(path: &str) {
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
    let rom = ROM::from_path(Path::new(path)).unwrap();
    let mut tile_frame = Frame::new();
    emulator.load_rom(&rom);

    let max_page = rom.get_chr_bank_count();
    let mut page  = 0;

    loop {
        tile_frame.clear();
        for i in 0..256 {
            render_tile(&rom.chr_rom, page * 2, i, &mut tile_frame);
            render_tile(&rom.chr_rom, page * 2 + 1, i, &mut tile_frame);
        }

        texture.update(None, &tile_frame.background, Frame::WIDTH * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    std::process::exit(0)
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    if page + 1 < max_page {
                        page += 1;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    if page > 0 {
                        page -= 1;
                    }
                },
                _ => {
                    // do nothing
                }
            }
        }
    }
}

// run nes game

fn run_game(path: &str) {
    let mut emu = Emulator::new();
    let rom = ROM::from_path(Path::new(path)).unwrap();
    emu.run_rom(&rom);
}

// todo: test audio with different games
//  - pacman: nothing sounds right
//  - duck hunt: is broken (also visually broken)
//  - pinball: shouldn't have audio playing during demo
//  - ice climber: breaking blocks (noise) doesn't sound right
//  - balloon fight: shouldn't have any audio during title screen and credits
//  - donkey kong: footsteps and jumps dont sound right

// todo: continue mapper1 debugging
//  - chessmaster: freezes on the menu screen (same as winter games - related?)
//  - simpsons - bart vs the space mutants: unplayable
//  - smb_dh_wctm: super mario bros can't be selected, duck hunt unplayable
//  - teenage mutant ninja turtles: same issue as contra
//  - yoshi: unplayable

// todo: continue sprite zero debugging
//  - 240pee: horizontal hill test is broken
//      - fix is to clear SpriteZeroHit on scanline 260 instead of 261
//  - castlevania: SpriteZeroHit clear on start of vblank messes things up
//  - friday the 13th: always broken split screen

fn main() {
    // run_snake();
    // run_chrdump("rom/mapper66/super_mario_bros_duck_hunt.nes");
    // run_game("rom/test/cpu/nestest.nes");
    // run_game("rom/test/ppu/240pee.nes");
    // run_game("rom/test/apu/sndtest.nes");

    // run_game("rom/mapper0/super_mario_bros.nes");
    run_game("rom/mapper1/teenage_mutant_ninja_turtles.nes");
    // run_game("rom/mapper2/castlevania.nes");
    // run_game("rom/mapper3/friday_the_13th.nes");
    // run_game("rom/mapper4/super_mario_bros_3.nes");
    // run_game("rom/mapper5/castlevania_3.nes"); // todo: impl
    // run_game("rom/mapper66/super_mario_bros_duck_hunt.nes");

    /* TODO | regression test plan - run each game after changes | TODO */
    // run_game("rom/mapper0/ice_climber.nes");
    // run_game("rom/mapper66/super_mario_bros_duck_hunt.nes");
    // run_game("rom/mapper1/super_mario_bros_duck_hunt_world_world_class_track_meet.nes");
    // run_game("rom/mapper3/arkistas_ring.nes");
}
