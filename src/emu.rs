use std::collections::HashMap;
use std::path::Path;
use std::time::{Instant};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::PixelFormatEnum;
use sdl2::{EventPump};
use sdl2::render::{Texture, WindowCanvas};
use crate::nes::NES;
use crate::nes::io::frame::Frame;
use crate::nes::io::joycon::joycon_status::JoyconButton;
use crate::nes::ppu::registers::mask::MaskFlag::{ShowBackground, ShowSprites};
use crate::nes::rom::ROM;
use crate::util::bitvec::BitVector;
use crate::util::savestate::{SaveState};
use crate::util::sleep::PreciseSleeper;

pub struct Emulator {
    pub nes: NES,
    pub sleeper: PreciseSleeper,

    pub fps_timestamp: Instant,
    pub frame_timestamp: Instant,
    pub fps: f64,
    pub frames: u64,

    pub volume: f32,
    pub mute: bool,
    pub mute_pulse_one: bool,
    pub mute_pulse_two: bool,
    pub mute_triangle: bool,
    pub mute_noise: bool,
    pub mute_dmc: bool,
    pub fast_forward: bool,
    pub hide_background: bool,
    pub hide_sprites: bool,
}

impl Emulator {
    const TARGET_FPS: f64 = 60.0;

    pub fn new() -> Self {
        Emulator {
            nes: NES::new(),
            sleeper: PreciseSleeper::new(),

            fps_timestamp: Instant::now(),
            frame_timestamp: Instant::now(),
            fps: 0.0,
            frames: 0,

            volume: 1.00, // todo: implement
            mute: false,
            mute_pulse_one: false,
            mute_pulse_two: false,
            mute_triangle: false,
            mute_noise: false,
            mute_dmc: false,
            fast_forward: false,
            hide_background: false,
            hide_sprites: false,
        }
    }

    pub fn run_rom(&mut self, rom: &ROM) {
        self.load_rom(&rom);

        const SCALE: f32 = 3.0;
        const WINDOW_WIDTH: u32 = (SCALE * Frame::WIDTH as f32) as u32;
        const WINDOW_HEIGHT: u32 = (SCALE * Frame::HEIGHT as f32) as u32;
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("alpiNES", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered().build().unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let creator = canvas.texture_creator();
        let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, Frame::WIDTH as u32, Frame::HEIGHT as u32).unwrap();

        self.nes.cpu.memory.apu.init_audio_player(&sdl_context);

        loop {
            if self.nes.cpu.memory.ppu.poll_nmi() {
                self.nes.cpu.handle_nmi();
                self.nes.cpu.memory.ppu.clear_nmi();

                self.handle_input(&mut event_pump);
                self.render_frame(&mut canvas, &mut texture);
                self.sleep_frame();
            } else if rom.mapper_id == 4 && self.nes.cpu.memory.ppu.memory.rom.mapper4.poll_irq() {
               self.nes.cpu.handle_irq();
            }

            let Ok(_) = self.nes.step() else { return };
        }
    }

    fn render_frame(&mut self, canvas: &mut WindowCanvas, texture: &mut Texture) {
        let ppu = &mut self.nes.cpu.memory.ppu;
        let show_background = !self.hide_background && ppu.mask.is_set(ShowBackground);
        let show_sprites = !self.hide_sprites && ppu.mask.is_set(ShowSprites);
        match (show_background, show_sprites) {
            (true, true) => texture.update(None, ppu.frame.compose(), Frame::WIDTH * 3).unwrap(),
            (true, false) => texture.update(None, &ppu.frame.background, Frame::WIDTH * 3).unwrap(),
            (false, true) => texture.update(None, &ppu.frame.sprite, Frame::WIDTH * 3).unwrap(),
            (false, false) => texture.update(None, &[0; 3 * Frame::WIDTH * Frame::HEIGHT], Frame::WIDTH * 3).unwrap(),
        }
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }

    fn handle_input(&mut self, event_pump: &mut EventPump) {
        let mut keymap_one = HashMap::new();
        keymap_one.insert(Keycode::Down, JoyconButton::Down);
        keymap_one.insert(Keycode::Up, JoyconButton::Up);
        keymap_one.insert(Keycode::Right, JoyconButton::Right);
        keymap_one.insert(Keycode::Left, JoyconButton::Left);
        keymap_one.insert(Keycode::RShift, JoyconButton::Select);
        keymap_one.insert(Keycode::Return, JoyconButton::Start);
        keymap_one.insert(Keycode::Z, JoyconButton::A);
        keymap_one.insert(Keycode::X, JoyconButton::B);

        let mut keymap_two = HashMap::new();
        keymap_two.insert(Keycode::Semicolon, JoyconButton::Down);
        keymap_two.insert(Keycode::P, JoyconButton::Up);
        keymap_two.insert(Keycode::Quote, JoyconButton::Right);
        keymap_two.insert(Keycode::L, JoyconButton::Left);
        keymap_two.insert(Keycode::Minus, JoyconButton::Select);
        keymap_two.insert(Keycode::Plus, JoyconButton::Start);
        keymap_two.insert(Keycode::A, JoyconButton::A);
        keymap_two.insert(Keycode::S, JoyconButton::B);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    std::process::exit(0)
                },
                Event::KeyDown { keycode: Some(Keycode::Num1), keymod, .. } => {
                    self.handle_savestate_input(keymod, 1);
                },
                Event::KeyDown { keycode: Some(Keycode::Num2), keymod, .. } => {
                    self.handle_savestate_input(keymod, 2);
                },
                Event::KeyDown { keycode: Some(Keycode::Num3), keymod, .. } => {
                    self.handle_savestate_input(keymod, 3);
                },
                Event::KeyDown { keycode: Some(Keycode::Num4), keymod, .. } => {
                    self.handle_savestate_input(keymod, 4);
                },
                Event::KeyDown { keycode: Some(Keycode::Num5), keymod, .. } => {
                    self.handle_savestate_input(keymod, 5);
                },
                Event::KeyDown { keycode: Some(Keycode::Num6), keymod, .. } => {
                    self.handle_savestate_input(keymod, 6);
                },
                Event::KeyDown { keycode: Some(Keycode::Num7), keymod, .. } => {
                    self.handle_savestate_input(keymod, 7);
                },
                Event::KeyDown { keycode: Some(Keycode::Num8), keymod, .. } => {
                    self.handle_savestate_input(keymod, 8);
                },
                Event::KeyDown { keycode: Some(Keycode::Num9), keymod, .. } => {
                    self.handle_savestate_input(keymod, 9);
                },
                Event::KeyDown { keycode: Some(Keycode::Num0), keymod, .. } => {
                    self.handle_savestate_input(keymod, 0);
                },
                Event::KeyDown { keycode: Some(Keycode::F1), .. } => {
                    self.mute_pulse_one = !self.mute_pulse_one;
                    self.nes.cpu.memory.apu.audio_player.as_mut().unwrap().device.lock().mute_pulse_one = self.mute_pulse_one;
                },
                Event::KeyDown { keycode: Some(Keycode::F2), .. } => {
                    self.mute_pulse_two = !self.mute_pulse_two;
                    self.nes.cpu.memory.apu.audio_player.as_mut().unwrap().device.lock().mute_pulse_two = self.mute_pulse_two;
                },
                Event::KeyDown { keycode: Some(Keycode::F3), .. } => {
                    self.mute_triangle = !self.mute_triangle;
                    self.nes.cpu.memory.apu.audio_player.as_mut().unwrap().device.lock().mute_triangle = self.mute_triangle;
                },
                Event::KeyDown { keycode: Some(Keycode::F4), .. } => {
                    self.mute_noise = !self.mute_noise;
                    self.nes.cpu.memory.apu.audio_player.as_mut().unwrap().device.lock().mute_noise = self.mute_noise;
                },
                Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                    self.mute_dmc = !self.mute_dmc;
                    self.nes.cpu.memory.apu.audio_player.as_mut().unwrap().device.lock().mute_dmc = self.mute_dmc;
                },
                Event::KeyDown { keycode: Some(Keycode::F6), .. } => {
                    self.mute = !self.mute;
                    self.nes.cpu.memory.apu.audio_player.as_mut().unwrap().device.lock().mute = self.mute;
                },
                Event::KeyDown { keycode: Some(Keycode::F11), .. } => {
                    self.hide_background = !self.hide_background;
                },
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    self.hide_sprites = !self.hide_sprites;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    self.fast_forward = true;
                },
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => {
                    self.fast_forward = false;
                },
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = keymap_one.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        self.nes.cpu.memory.joycon1.set_button((*key).clone());
                    }
                    if let Some(key) = keymap_two.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        self.nes.cpu.memory.joycon2.set_button((*key).clone());
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = keymap_one.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        self.nes.cpu.memory.joycon1.clear_button((*key).clone());
                    }
                    if let Some(key) = keymap_two.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        self.nes.cpu.memory.joycon2.clear_button((*key).clone());
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_savestate_input(&mut self, keymod: Mod, save_idx: u8) {
        if keymod == Mod::LGUIMOD.union(Mod::LALTMOD) {
            self.load_state(save_idx);
        } else if keymod == Mod::LGUIMOD {
            self.save_state(save_idx);
        }
    }

    fn sleep_frame(&mut self) {
        self.tick_fps();
        if !self.fast_forward {
            let mut sleep_time = 1.0 / Emulator::TARGET_FPS - self.frame_timestamp.elapsed().as_secs_f64();
            if sleep_time > 0.0 {
                PreciseSleeper::new().precise_sleep(sleep_time);
            }
        }
        self.frame_timestamp = Instant::now();
    }

    fn tick_fps(&mut self) {
        self.frames += 1;
        if self.frames % 100 == 0 {
            self.fps = 100.0 / self.fps_timestamp.elapsed().as_secs_f64();
            self.fps_timestamp = Instant::now();
            self.frames = 0;
            println!("fps: {:.2}", self.fps);
        }
    }

    pub fn load_state(&mut self, save_idx: u8) {
        println!("loading state {}...", save_idx);

        let save_path_str = format!("Saves/{}/{}.savestate", self.nes.cpu.memory.rom.game_title, save_idx);
        let save_path = Path::new(save_path_str.as_str());
        if let Some(save_state) = SaveState::deserialize(save_path) {
            SaveState::load_nes_state(&mut self.nes, &save_state);
        }
    }

    pub fn save_state(&mut self, save_idx: u8) {
        println!("saving state {}...", save_idx);

        let game_title = &self.nes.cpu.memory.rom.game_title;
        let save_path_str = format!("Saves/{}/{}.savestate", game_title, save_idx);
        let save_path = Path::new(save_path_str.as_str());
        SaveState::serialize(save_path, &SaveState::new(&self.nes));
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.nes.load_rom(rom);
    }

    pub fn load(&mut self, program: &Vec<u8>) {
        self.nes.load(program)
    }

    pub fn load_at_addr(&mut self, addr: u16, program: &Vec<u8>) {
        self.nes.load_at_addr(addr, program);
    }

    pub fn load_and_run(&mut self, program: &Vec<u8>) {
        self.nes.load(program);
        self.run()
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F) where F: FnMut(&mut NES) {
        loop {
            if self.nes.cpu.memory.ppu.poll_nmi() {
                self.tick_fps();
                self.nes.cpu.handle_nmi();
            }
            callback(&mut self.nes);
            let Ok(_) = self.nes.step() else { return };
        }
    }

    pub fn reset(&mut self) {
        self.nes.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nes::cpu::CPU;
    use crate::nes::cpu::mem::Memory;

    #[test]
    fn test_load_and_reset() {
        let mut emu = Emulator::new();
        emu.load(&vec![0xff]);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START);
        assert_eq!(cpu.memory.read_byte(Memory::PRG_ROM_START), 0xff);
    }

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut emu = Emulator::new();
        emu.load(&vec![CPU::LDA_IM, 5, CPU::BRK]);
        emu.run();

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0x05);
        assert_eq!(cpu.status.value & 0b0000_0010, 0);
        assert_eq!(cpu.status.value & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0, CPU::BRK]);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.status.value & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0xff, CPU::BRK]);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0xff);
        assert_eq!(cpu.status.value & 0b1000_0000, 0b1000_0000);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0x10, CPU::TAX, CPU::BRK]);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0x10);
        assert_eq!(cpu.register_x, 0x10);
    }

    #[test]
    fn test_inx_overflow() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDX_IM, 0xff, CPU::INX, CPU::INX, CPU::BRK]);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_x, 1);
    }

    #[test]
    fn test_5_ops() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0xc0, CPU::TAX, CPU::INX, CPU::BRK]);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0xc0);
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_program_simple() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa9, 0x01, 0x8d, 0x00, 0x02, 0xa9, 0x05, 0x8d, 0x01, 0x02,
            0xa9, 0x08, 0x8d, 0x02, 0x02, 0x00
        ];
        emu.load_and_run(&program);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0x08);
        assert_eq!(cpu.memory.read_byte(0x0202), 0x08);
        assert_eq!(cpu.status.value, 0b0010_0100);
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_adc() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa9, 0xc0, 0xaa, 0xe8, 0x69, 0xc4, 0x00
        ];
        emu.load_and_run(&program);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0x84);
        assert_eq!(cpu.register_x, 0xc1);
        assert_eq!(cpu.status.value, 0b1010_0101);
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_signed_division_by_four() {
        let mut emu = Emulator::new();
        let program = vec![
            CPU::LDA_IM, 0x88, CPU::CMP_IM, 0x80, CPU::ARR, 0xff, CPU::ROR, CPU::BRK
        ];
        emu.load_and_run(&program);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0xe2);
        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.register_y, 0x00);
        assert_eq!(cpu.status.value, 0b1110_0100);
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_adc_carry_overflow() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa9, 0xff, 0x69, 0xff, 0xa9, 0x0f, 0x69, 0x70, 0x00
        ];
        emu.load_and_run(&program);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.status.value, 0b1110_0100);
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_branch() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa2, 0x08, 0xca, 0x8e, 0x00, 0x02, 0xe0, 0x03, 0xd0, 0xf8,
            0x8e, 0x01, 0x02, 0x00
        ];
        emu.load_and_run(&program);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_x, 0x03);
        assert_eq!(cpu.status.value, 0b0010_0111);
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_subroutines() {
        let mut emu = Emulator::new();
        let program = vec![
            0x20, 0x09, 0x06, 0x20, 0x0c, 0x06, 0x20, 0x12, 0x06, 0xa2,
            0x00, 0x60, 0xe8, 0xe0, 0x05, 0xd0, 0xfb, 0x60, 0x00
        ];
        emu.load_at_addr(0x600, &program);
        emu.run();

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_x, 0x05);
        assert_eq!(cpu.stack, 0xfb);
        assert_eq!(cpu.status.value, 0b0010_0111);
        assert_eq!(cpu.program_counter, 0x600 + program.len() as u16);
    }

    #[test]
    fn test_program_indexed_indirect_x() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa2, 0x01, 0xa9, 0x05, 0x85, 0x01, 0xa9, 0x07, 0x85, 0x02,
            0xa0, 0x0a, 0x8c, 0x05, 0x07, 0xa1, 0x00, 0x00
        ];
        emu.load_and_run(&program);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0x0a);
        assert_eq!(cpu.register_x, 0x01);
        assert_eq!(cpu.register_y, 0x0a);
        assert_eq!(cpu.status.value, 0b0010_0100);
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_indirect_indexed_y() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa0, 0x01, 0xa9, 0x03, 0x85, 0x01, 0xa9, 0x07, 0x85, 0x02,
            0xa2, 0x0a, 0x8e, 0x04, 0x07, 0xb1, 0x01, 0x00
        ];
        emu.load_and_run(&program);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0x0a);
        assert_eq!(cpu.register_x, 0x0a);
        assert_eq!(cpu.register_y, 0x01);
        assert_eq!(cpu.status.value, 0b0010_0100);
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_stack_operations() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa2, 0x00, 0xa0, 0x00, 0x8a, 0x99, 0x00, 0x02, 0x48, 0xe8,
            0xc8, 0xc0, 0x10, 0xd0, 0xf5, 0x68, 0x99, 0x00, 0x02, 0xc8,
            0xc0, 0x20, 0xd0, 0xf7, 0x00
        ];
        emu.load_and_run(&program);

        let mut cpu = &mut emu.nes.cpu;
        for i in 0..16 {
            assert_eq!(cpu.memory.read_byte(0x200 + i), i as u8);
            assert_eq!(cpu.memory.read_byte(0x200 + (31 - i)), i as u8);
        }
        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.register_x, 0x10);
        assert_eq!(cpu.register_y, 0x20);
        assert_eq!(cpu.stack, 0xfd);
        assert_eq!(cpu.status.value, 0b0010_0111);
        assert_eq!(cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_snake_game() {
        let mut emu = Emulator::new();
        let program = vec![
            0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20,
            0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85, 0x02, 0xa9, 0x04, 0x85,
            0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9,
            0x0f, 0x85, 0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85,
            0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe, 0x29, 0x03,
            0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20,
            0x8d, 0x06, 0x20, 0xc3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20,
            0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
            0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0,
            0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60, 0xa9, 0x04, 0x24, 0x02,
            0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24,
            0x02, 0xd0, 0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01,
            0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02, 0x60, 0xa9,
            0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60,
            0x60, 0x20, 0x94, 0x06, 0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00,
            0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
            0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02,
            0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06, 0xb5, 0x11, 0xc5, 0x11,
            0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa,
            0x06, 0x4c, 0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5,
            0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02, 0x4a, 0xb0,
            0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f,
            0xa5, 0x10, 0x38, 0xe9, 0x20, 0x85, 0x10, 0x90, 0x01, 0x60,
            0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
            0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10,
            0x18, 0x69, 0x20, 0x85, 0x10, 0xb0, 0x01, 0x60, 0xe6, 0x11,
            0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5,
            0x10, 0x29, 0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35,
            0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60, 0xa6, 0x03,
            0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10,
            0x60, 0xa2, 0x00, 0xea, 0xea, 0xca, 0xd0, 0xfb, 0x60, 0x00
        ];
        emu.load_at_addr(0x600, &program);
        emu.run();

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0x1f);
        assert_eq!(cpu.register_x, 0xff);
        assert_eq!(cpu.register_y, 0x00);
        assert_eq!(cpu.stack, 0xf9);
        assert_eq!(cpu.status.value, 0b0010_0111);
        assert_eq!(cpu.program_counter, 0x0736);
    }
}