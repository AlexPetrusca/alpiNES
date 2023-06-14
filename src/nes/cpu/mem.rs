use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use crate::nes::apu::APU;
use crate::nes::cpu::CPU;
use crate::nes::io::joycon::Joycon;
use crate::nes::io::joycon::joycon_status::JoyconButton::Select;
use crate::nes::ppu::PPU;
use crate::nes::rom::ROM;

// CPU memory map
#[macro_export] macro_rules! ram_range { () => {0x0000..=0x1FFF} }
#[macro_export] macro_rules! ppu_registers_range { () => {0x2000..=0x3FFF} }
#[macro_export] macro_rules! apu_io_registers_range { () => {0x4000..=0x401F} }
#[macro_export] macro_rules! custom_ram_range { () => {0x4020..=0x5FFF} }
#[macro_export] macro_rules! prg_ram_range { () => {0x6000..=0x7FFF} }
#[macro_export] macro_rules! prg_rom_range { () => {0x8000..=0xFFFF} }

pub struct Memory {
    pub memory: [u8; Memory::MEM_SIZE],
    pub ppu: PPU,
    pub apu: APU,
    pub rom: ROM, // todo: should this be Option<ROM>?
    pub save_ram: Option<File>,
    pub joycon1: Joycon,
    pub joycon2: Joycon,
}

impl Memory {
    pub const MEM_SIZE: usize = 0x10000 as usize; // 64kB
    pub const PRG_ROM_START: u16 = *prg_rom_range!().start();

    pub const PPU_CTRL_REGISTER: u16 = 0x2000;
    pub const PPU_MASK_REGISTER: u16 = 0x2001;
    pub const PPU_STAT_REGISTER: u16 = 0x2002;
    pub const PPU_OAM_ADDR_REGISTER: u16 = 0x2003;
    pub const PPU_OAM_DATA_REGISTER: u16 = 0x2004;
    pub const PPU_SCROLL_REGISTER: u16 = 0x2005;
    pub const PPU_ADDR_REGISTER: u16 = 0x2006;
    pub const PPU_DATA_REGISTER: u16 = 0x2007;
    pub const PPU_OAM_DMA_REGISTER: u16 = 0x4014;
    pub const JOYCON_ONE_REGISTER: u16 = 0x4016;
    pub const JOYCON_TWO_REGISTER: u16 = 0x4017;

    pub const APU_PULSE_ONE_REGISTER_A: u16 = 0x4000;
    pub const APU_PULSE_ONE_REGISTER_B: u16 = 0x4001;
    pub const APU_PULSE_ONE_REGISTER_C: u16 = 0x4002;
    pub const APU_PULSE_ONE_REGISTER_D: u16 = 0x4003;
    pub const APU_PULSE_TWO_REGISTER_A: u16 = 0x4004;
    pub const APU_PULSE_TWO_REGISTER_B: u16 = 0x4005;
    pub const APU_PULSE_TWO_REGISTER_C: u16 = 0x4006;
    pub const APU_PULSE_TWO_REGISTER_D: u16 = 0x4007;
    pub const APU_TRIANGLE_REGISTER_A: u16 = 0x4008;
    pub const APU_TRIANGLE_REGISTER_B: u16 = 0x4009;
    pub const APU_TRIANGLE_REGISTER_C: u16 = 0x400A;
    pub const APU_TRIANGLE_REGISTER_D: u16 = 0x400B;
    pub const APU_NOISE_REGISTER_A: u16 = 0x400C;
    pub const APU_NOISE_REGISTER_B: u16 = 0x400D;
    pub const APU_NOISE_REGISTER_C: u16 = 0x400E;
    pub const APU_NOISE_REGISTER_D: u16 = 0x400F;
    pub const APU_DMC_REGISTER_A: u16 = 0x4010;
    pub const APU_DMC_REGISTER_B: u16 = 0x4011;
    pub const APU_DMC_REGISTER_C: u16 = 0x4012;
    pub const APU_DMC_REGISTER_D: u16 = 0x4013;
    pub const APU_STATUS_REGISTER: u16 = 0x4015;
    pub const APU_FRAME_COUNTER_REGISTER: u16 = 0x4017;

    pub const IRQ_INT_VECTOR: u16 = 0xFFFE;
    pub const RESET_INT_VECTOR: u16 = 0xFFFC;
    pub const NMI_INT_VECTOR: u16 = 0xFFFA;

    pub fn new() -> Self {
        Memory {
            memory: [0; Memory::MEM_SIZE],
            ppu: PPU::new(),
            apu: APU::new(),
            rom: ROM::new(),
            save_ram: None,
            joycon1: Joycon::new(),
            joycon2: Joycon::new(),
        }
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.rom = rom.clone();
        self.ppu.memory.load_rom(rom);
        if rom.has_save_ram {
            self.init_save_ram();
        }
    }

    fn init_save_ram(&mut self) {
        let save_path = format!("Saves/{}", self.rom.game_title);
        fs::create_dir_all(&save_path).unwrap();
        let save_path = format!("{}/battery.sav", save_path);
        if Path::new(save_path.as_str()).exists() {
            let mut save_file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(save_path)
                .unwrap();
            save_file.read(&mut self.memory[prg_ram_range!()]).expect("unable to load save file");
            self.save_ram = Some(save_file);
        } else {
            let mut save_file = File::create(save_path).expect("unable to create save file");
            save_file.write(vec![0; 0x2000].as_slice()).expect("unable to init save file");
            self.save_ram = Some(save_file);
        }
    }

    pub fn load_at_addr(&mut self, address: u16, program: &Vec<u8>) {
        for i in 0..program.len() {
            self.memory[address.wrapping_add(i as u16) as usize] = program[i];
        }
        let addr_bytes = &u16::to_le_bytes(address);
        self.memory[Memory::RESET_INT_VECTOR as usize] = addr_bytes[0];
        self.memory[Memory::RESET_INT_VECTOR.wrapping_add(1) as usize] = addr_bytes[1];
    }

    #[inline]
    pub fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            ram_range!() => {
                let mirror_addr = address & 0b0000_0111_1111_1111;
                self.memory[mirror_addr as usize]
            },
            ppu_registers_range!() => {
                let mirror_addr = address & 0b0010_0000_0000_0111;
                match mirror_addr {
                    Memory::PPU_CTRL_REGISTER | Memory::PPU_MASK_REGISTER |
                    Memory::PPU_OAM_ADDR_REGISTER | Memory::PPU_SCROLL_REGISTER |
                    Memory::PPU_ADDR_REGISTER => {
                        return 0 // todo: simulate ppu open bus here
                    },
                    Memory::PPU_STAT_REGISTER => {
                        self.ppu.read_status_register()
                    },
                    Memory::PPU_DATA_REGISTER => {
                        self.ppu.read_data_register()
                    },
                    Memory::PPU_OAM_DATA_REGISTER => {
                        self.ppu.read_oam_data_register()
                    },
                    _ => {
                        panic!("Attempt to read from write-only PPU address memory: 0x{:0>4X}", mirror_addr);
                    }
                }
            },
            apu_io_registers_range!() => {
                match address {
                    Memory::JOYCON_ONE_REGISTER => {
                        self.joycon1.read()
                    },
                    Memory::JOYCON_TWO_REGISTER => {
                        self.joycon2.read()
                    },
                    Memory::APU_PULSE_ONE_REGISTER_A..=Memory::APU_PULSE_ONE_REGISTER_D => {
                        self.apu.pulse_one.read(address as u8 % 4)
                    },
                    Memory::APU_PULSE_TWO_REGISTER_A..=Memory::APU_PULSE_TWO_REGISTER_D => {
                        self.apu.pulse_two.read(address as u8 % 4)
                    },
                    Memory::APU_TRIANGLE_REGISTER_A..=Memory::APU_TRIANGLE_REGISTER_D => {
                        self.apu.triangle.read(address as u8 % 4)
                    },
                    Memory::APU_NOISE_REGISTER_A..=Memory::APU_NOISE_REGISTER_D => {
                        self.apu.noise.read(address as u8 % 4)
                    },
                    Memory::APU_DMC_REGISTER_A..=Memory::APU_DMC_REGISTER_D => {
                        self.apu.dmc.read(address as u8 % 4)
                    },
                    Memory::APU_STATUS_REGISTER => {
                        self.apu.read_status_register()
                    },
                    _ => {
                        panic!("Attempt to read from unmapped APU/IO address memory: 0x{:0>4X}", address);
                    }
                }
            },
            custom_ram_range!() => {
                println!("[WARNING] Read from custom ram range: 0x{:0>4X}", address);
                self.memory[address as usize]
            },
            prg_ram_range!() => {
                self.memory[address as usize]
            },
            prg_rom_range!() => {
                self.rom.read_prg_byte(address)
            },
            _ => {
                panic!("Attempt to read from unmapped memory: 0x{:0>4X}", address);
            }
        }
    }

    #[inline]
    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            ram_range!() => {
                let mirror_addr = address & 0b0000_0111_1111_1111;
                self.memory[mirror_addr as usize] = data;
            }
            ppu_registers_range!() => {
                let mirror_addr = address & 0b0010_0000_0000_0111;
                match mirror_addr {
                    Memory::PPU_CTRL_REGISTER => {
                        self.ppu.write_ctrl_register(data);
                    },
                    Memory::PPU_MASK_REGISTER => {
                        self.ppu.write_mask_register(data);
                    },
                    Memory::PPU_ADDR_REGISTER => {
                        self.ppu.write_addr_register(data);
                    },
                    Memory::PPU_DATA_REGISTER => {
                        self.ppu.write_data_register(data);
                    },
                    Memory::PPU_OAM_ADDR_REGISTER => {
                        self.ppu.write_oam_addr_register(data);
                    },
                    Memory::PPU_OAM_DATA_REGISTER => {
                        self.ppu.write_oam_data_register(data);
                    },
                    Memory::PPU_SCROLL_REGISTER => {
                        self.ppu.write_scroll_register(data);
                    },
                    _ => {
                        println!("[WARNING] Attempt to write to read-only PPU register: 0x{:0>4X}", mirror_addr);
                    }
                }
            }
            apu_io_registers_range!() => {
                match address {
                    Memory::PPU_OAM_DMA_REGISTER => {
                        let read_addr = (data as u16) << 8;
                        let write_addr = self.ppu.oam_addr;
                        for i in 0..256 {
                            let value = self.read_byte(read_addr.wrapping_add(i));
                            self.ppu.oam.write_byte(write_addr.wrapping_add(i as u8), value);
                        }
                        // todo: this op takes between 513 - 514 CPU cycles to execute
                    },
                    Memory::JOYCON_ONE_REGISTER => {
                        self.joycon1.write(data);
                        self.joycon2.write(data);
                    },
                    Memory::APU_PULSE_ONE_REGISTER_A..=Memory::APU_PULSE_ONE_REGISTER_D => {
                        self.apu.write_pulse_one_registers(address as u8 % 4, data);
                    },
                    Memory::APU_PULSE_TWO_REGISTER_A..=Memory::APU_PULSE_TWO_REGISTER_D => {
                        self.apu.write_pulse_two_registers(address as u8 % 4, data);
                    },
                    Memory::APU_TRIANGLE_REGISTER_A..=Memory::APU_TRIANGLE_REGISTER_D => {
                        self.apu.write_triangle_registers(address as u8 % 4, data);
                    },
                    Memory::APU_NOISE_REGISTER_A..=Memory::APU_NOISE_REGISTER_D => {
                        self.apu.write_noise_registers(address as u8 % 4, data);
                    },
                    Memory::APU_DMC_REGISTER_A..=Memory::APU_DMC_REGISTER_D => {
                        self.apu.write_dmc_registers(address as u8 % 4, data);
                        if address == Memory::APU_DMC_REGISTER_C || address == Memory::APU_DMC_REGISTER_D {
                            // println!("DMC HIT");
                            // let sample_addr = self.apu.dmc.get_sample_address();
                            // let sample_length = self.apu.dmc.get_sample_length();
                            // for addr in sample_addr..(sample_addr + sample_length) {
                            //     let sample = self.read_byte(addr);
                            //     // todo: don't lock in a loop...
                            //     let mut guard = self.apu.audio_player.as_mut().unwrap().device.lock(); // todo: pulling the guard out like this sucks. Write a helper method
                            //     guard.dmc.add_dpcm_sample(sample);
                            // }
                        }
                    },
                    Memory::APU_STATUS_REGISTER => {
                        self.apu.write_status_register(data);
                    },
                    Memory::APU_FRAME_COUNTER_REGISTER => {
                        // todo: implement or replace
                    },
                    _ => {
                        panic!("Attempt to write to unmapped APU/IO address memory: 0x{:0>4X}", address);
                    }
                }
            }
            custom_ram_range!() => {
                println!("[WARNING] Write to custom ram range: 0x{:0>4X}", address);
                self.memory[address as usize] = data;
            },
            prg_ram_range!() => {
                self.memory[address as usize] = data;
                if self.rom.has_save_ram {
                    let pos = (address - 0x6000) as u64;
                    let mut save_file = self.save_ram.as_mut().unwrap();
                    save_file.seek(SeekFrom::Start(pos)).expect("unable to seek in save file");
                    save_file.write(&[data]).expect("unable to write to save file");
                }
            },
            prg_rom_range!() => {
                self.rom.write_prg_byte(address, data);
                self.ppu.memory.rom.write_prg_byte(address, data);
            },
            _ => {
                panic!("Attempt to write to unmapped memory: 0x{:0>4X}", address);
            }
        }
    }

    #[inline]
    pub fn write_bulk(&mut self, address: u16, data: &[u8]) {
        for i in 0..data.len() {
            self.write_byte(address.wrapping_add(i as u16), data[i]);
        }
    }

    #[inline]
    pub fn read_addr(&mut self, address: u16) -> u16 {
        u16::from_le_bytes([
            self.read_byte(address),
            self.read_byte(address.wrapping_add(1))
        ])
    }

    #[inline]
    pub fn read_addr_zp(&mut self, address: u8) -> u16 {
        u16::from_le_bytes([
            self.read_byte(address as u16),
            self.read_byte(address.wrapping_add(1) as u16)
        ])
    }

    #[inline]
    pub fn read_addr_in(&mut self, address: u16) -> u16 {
        let upper_addr = address & 0xff00;
        let lower_addr = (address & 0x00ff) as u8;
        u16::from_le_bytes([
            self.read_byte(address),
            self.read_byte(upper_addr + lower_addr.wrapping_add(1) as u16)
        ])
    }

    #[inline]
    pub fn write_addr(&mut self, address: u16, waddr: u16) {
        let bytes = u16::to_le_bytes(waddr);
        self.write_byte(address, bytes[0]);
        self.write_byte(address.wrapping_add(1), bytes[1]);
    }

    #[inline]
    pub fn zp_read(&mut self, address: u8) -> u8 {
        self.read_byte(address as u16)
    }

    #[inline]
    pub fn zp_x_read(&mut self, address: u8, register_x: u8) -> u8 {
        self.read_byte(address.wrapping_add(register_x) as u16)
    }

    #[inline]
    pub fn zp_y_read(&mut self, address: u8, register_y: u8) -> u8 {
        self.read_byte(address.wrapping_add(register_y) as u16)
    }

    #[inline]
    pub fn ab_read(&mut self, address: u16) -> u8 {
        self.read_byte(address)
    }

    #[inline]
    pub fn ab_x_read(&mut self, address: u16, register_x: u8) -> u8 {
        self.read_byte(address.wrapping_add(register_x as u16))
    }

    #[inline]
    pub fn ab_y_read(&mut self, address: u16, register_y: u8) -> u8 {
        self.read_byte(address.wrapping_add(register_y as u16))
    }

    #[inline]
    pub fn in_x_read(&mut self, address: u8, register_x: u8) -> u8 {
        let pointer = self.read_addr_zp(address.wrapping_add(register_x));
        self.read_byte(pointer)
    }

    #[inline]
    pub fn in_y_read(&mut self, address: u8, register_y: u8) -> u8 {
        let pointer = self.read_addr_zp(address);
        self.read_byte(pointer.wrapping_add(register_y as u16))
    }

    #[inline]
    pub fn zp_write(&mut self, address: u8, value: u8) {
        self.write_byte(address as u16, value);
    }

    #[inline]
    pub fn zp_x_write(&mut self, address: u8, register_x: u8, value: u8) {
        self.write_byte(address.wrapping_add(register_x) as u16, value);
    }

    #[inline]
    pub fn zp_y_write(&mut self, address: u8, register_y: u8, value: u8) {
        self.write_byte(address.wrapping_add(register_y) as u16, value);
    }

    #[inline]
    pub fn ab_write(&mut self, address: u16, value: u8) {
        self.write_byte(address, value);
    }

    #[inline]
    pub fn ab_x_write(&mut self, address: u16, register_x: u8, value: u8) {
        self.write_byte(address.wrapping_add(register_x as u16), value);
    }

    #[inline]
    pub fn ab_y_write(&mut self, address: u16, register_y: u8, value: u8) {
        self.write_byte(address.wrapping_add(register_y as u16), value)
    }

    #[inline]
    pub fn in_x_write(&mut self, address: u8, register_x: u8, value: u8) {
        let pointer = self.read_addr_zp(address.wrapping_add(register_x));
        self.write_byte(pointer, value);
    }

    #[inline]
    pub fn in_y_write(&mut self, address: u8, register_y: u8, value: u8) {
        let pointer = self.read_addr_zp(address);
        self.write_byte(pointer.wrapping_add(register_y as u16), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTE_A: u8 = 0x0a;
    const BYTE_B: u8 = 0x0b;

    // todo: add more tests for memory

    #[test]
    fn test_read_write() {
        let mut mem = Memory::new();
        mem.write_byte(0x0001, BYTE_A);
        mem.write_byte(0x0002, BYTE_B);
        assert_eq!(mem.read_byte(0x0001), BYTE_A);
        assert_eq!(mem.read_byte(0x0002), BYTE_B);
    }

    #[test]
    fn test_write_bulk() {
        let mut mem = Memory::new();
        mem.write_bulk(0x0001, &[BYTE_A, BYTE_B]);
        assert_eq!(mem.read_byte(0x0001), BYTE_A);
        assert_eq!(mem.read_byte(0x0002), BYTE_B);
    }

    #[test]
    fn test_read_write_addr() {
        let mut mem = Memory::new();
        mem.write_addr(0x0100, 0x0a0b);
        assert_eq!(mem.read_byte(0x0100), 0x0b);
        assert_eq!(mem.read_addr(0x0101), 0x0a);
        assert_eq!(mem.read_addr(0x0100), 0x0a0b);
    }
}