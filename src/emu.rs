use std::thread::sleep;
use std::time::{Duration, Instant};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, WindowCanvas};
use sdl2::{EventPump, Sdl};
use sdl2::video::Window;
use crate::nes::NES;
use crate::util::rom::ROM;
use crate::nes::cpu::CPU;
use crate::nes::cpu::mem::Memory;
use crate::nes::ppu::PPU;
use crate::nes::ppu::mem::PpuMemory;
use crate::nes::io::frame::Frame;

pub struct Emulator {
    pub nes: NES,
    pub fps_timestamp: Instant,
    pub frame_timestamp: Instant,
    pub fps: f32,
    pub frames: u64,
}

impl Emulator {
    const TARGET_FPS: f32 = 60.0;

    pub fn new() -> Self {
        Emulator {
            nes: NES::new(),
            fps_timestamp: Instant::now(),
            frame_timestamp: Instant::now(), // todo use
            fps: 0.0,
            frames: 0,
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

        loop {
            if self.nes.cpu.memory.ppu.poll_nmi() {
                self.nes.cpu.handle_nmi();
                self.nes.cpu.memory.ppu.clear_nmi();

                let mut frame = Frame::new();
                Emulator::render(&self.nes.cpu.memory.ppu, &mut frame);

                texture.update(None, &frame.data, Frame::WIDTH * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();

                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } |
                        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                            std::process::exit(0)
                        },
                        _ => { }
                    }
                }

                self.sleep_frame();
            }

            let Ok(_) = self.nes.step() else { return };
        }
    }

    fn sleep_frame(&mut self) {
        self.tick_fps();
        const FRAME_SLEEP_OFFSET: f32 = 1.0 / 56.67 - 1.0 / 60.0; // todo: why do I need this? somethings wrong
        let mut sleep_time = 1.0 / Emulator::TARGET_FPS - self.frame_timestamp.elapsed().as_secs_f32() - FRAME_SLEEP_OFFSET;
        if sleep_time > 0.0 {
            sleep(Duration::from_secs_f32(sleep_time));
        }
        self.frame_timestamp = Instant::now();
    }

    fn tick_fps(&mut self) {
        self.frames += 1;
        if self.frames % 100 == 0 {
            self.fps = 100.0 / self.fps_timestamp.elapsed().as_secs_f32();
            self.fps_timestamp = Instant::now();
            self.frames = 0;
            println!("fps: {:.2}", self.fps);
        }
    }

    fn render(ppu: &PPU, frame: &mut Frame) {
        let bank = ppu.ctrl.get_background_chrtable_address();

        for i in 0..0x03c0 { // just for now, lets use the first nametable
            let tile = ppu.memory.read_byte(PpuMemory::VRAM_START + i) as u16;
            let tile_x = i % 32;
            let tile_y = i / 32;
            let tile = &ppu.memory.memory[(bank + 16 * tile) as usize..=(bank + 16 * tile + 15) as usize];
            let palette = Emulator::bg_palette(ppu, tile_x as usize, tile_y as usize);

            for y in 0..8 {
                let mut lower = tile[y as usize];
                let mut upper = tile[y as usize + 8];

                for x in (0..8).rev() {
                    let value = (1 & upper) << 1 | (1 & lower);
                    lower = lower >> 1;
                    upper = upper >> 1;
                    let rgb = match value {
                        0 => PPU::SYSTEM_PALLETE[palette[0] as usize],
                        1 => PPU::SYSTEM_PALLETE[palette[1] as usize],
                        2 => PPU::SYSTEM_PALLETE[palette[2] as usize],
                        3 => PPU::SYSTEM_PALLETE[palette[3] as usize],
                        _ => panic!("can't be"),
                    };
                    frame.set_pixel(8 * tile_x as usize + x, 8 * tile_y as usize + y, rgb)
                }
            }
        }
    }

    fn bg_palette(ppu: &PPU, tile_x: usize, tile_y: usize) -> [u8; 4] {
        let attr_table_idx = 8 * (tile_y / 4) + tile_x / 4;
        let attr_byte = ppu.memory.read_byte(PpuMemory::VRAM_START + 0x3c0 + attr_table_idx as u16);  // todo: note: still using hardcoded first nametable

        let pallete_idx = match ((tile_x % 4) / 2, (tile_y % 4) / 2) {
            (0, 0) => attr_byte & 0b0000_0011,
            (1, 0) => (attr_byte >> 2) & 0b0000_0011,
            (0, 1) => (attr_byte >> 4) & 0b0000_0011,
            (1, 1) => (attr_byte >> 6) & 0b0000_0011,
            (_, _) => panic!("can't be"),
        };

        let pallete_start = (4 * pallete_idx + 1) as u16;
        [
            ppu.memory.read_byte(PpuMemory::PALLETES_START),
            ppu.memory.read_byte(PpuMemory::PALLETES_START + pallete_start),
            ppu.memory.read_byte(PpuMemory::PALLETES_START + pallete_start + 1),
            ppu.memory.read_byte(PpuMemory::PALLETES_START + pallete_start + 2),
        ]
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
        assert_eq!(cpu.status & 0b0000_0010, 0);
        assert_eq!(cpu.status & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0, CPU::BRK]);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0xff, CPU::BRK]);

        let mut cpu = &mut emu.nes.cpu;
        assert_eq!(cpu.register_a, 0xff);
        assert_eq!(cpu.status & 0b1000_0000, 0b1000_0000);
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
        assert_eq!(cpu.status, 0b0010_0100);
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
        assert_eq!(cpu.status, 0b1010_0101);
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
        assert_eq!(cpu.status, 0b1110_0100);
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
        assert_eq!(cpu.status, 0b1110_0100);
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
        assert_eq!(cpu.status, 0b0010_0111);
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
        assert_eq!(cpu.status, 0b0010_0111);
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
        assert_eq!(cpu.status, 0b0010_0100);
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
        assert_eq!(cpu.status, 0b0010_0100);
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
        assert_eq!(cpu.status, 0b0010_0111);
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
        assert_eq!(cpu.status, 0b0010_0111);
        assert_eq!(cpu.program_counter, 0x0736);
    }
}