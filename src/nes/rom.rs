pub mod registers;
pub mod mappers;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::nes::cpu::mem::Memory;
use crate::nes::rom::mappers::mapper::Mapper;
use crate::nes::rom::mappers::mapper0::Mapper0;
use crate::nes::rom::mappers::mapper1::Mapper1;
use crate::nes::rom::mappers::mapper2::Mapper2;
use crate::nes::rom::mappers::mapper3::Mapper3;
use crate::nes::rom::mappers::mapper4::Mapper4;
use crate::nes::rom::mappers::mapper66::Mapper66;

#[derive(Serialize, Deserialize,Debug, PartialEq, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    OneScreenLower,
    OneScreenUpper,
    FourScreen,
}

#[derive(Clone)]
pub struct ROM {
    pub game_title: String,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper_id: u8,
    pub is_prg_rom_mirror: bool,
    pub is_chr_ram: bool,
    pub has_save_ram: bool,
    pub screen_mirroring: Mirroring,

    pub mapper0: Mapper0,
    pub mapper1: Mapper1,
    pub mapper2: Mapper2,
    pub mapper3: Mapper3,
    pub mapper4: Mapper4,
    pub mapper66: Mapper66,
}

impl ROM {
    const NES_SIGNATURE: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
    pub const CHR_ROM_PAGE_SIZE: usize = 0x2000; // 8kB
    pub const PRG_ROM_PAGE_SIZE: usize = 0x4000; // 16kB

    pub fn new() -> Self {
        ROM {
            game_title: String::new(),
            prg_rom: Vec::new(),
            chr_rom: Vec::new(),
            mapper_id: 0,
            is_prg_rom_mirror: false,
            is_chr_ram: false,
            has_save_ram: false,
            screen_mirroring: Mirroring::Horizontal,

            mapper0: Mapper0::new(),
            mapper1: Mapper1::new(),
            mapper2: Mapper2::new(),
            mapper3: Mapper3::new(),
            mapper4: Mapper4::new(),
            mapper66: Mapper66::new(),
        }
    }

    pub fn from_path(path: &Path) -> Result<ROM, String> {
        let mut file = File::open(path).expect("no file found");
        let metadata = fs::metadata(path).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        file.read(&mut buffer).expect("buffer overflow");
        let mut rom_result = ROM::from_buffer(&buffer);

        let game_title = path.file_stem().expect("unable to parse file stem");
        rom_result.as_mut().unwrap().game_title = game_title.to_str().unwrap().to_string();

        return rom_result;
    }

    pub fn from_buffer(raw: &Vec<u8>) -> Result<ROM, String> {
        if &raw[0..4] != ROM::NES_SIGNATURE {
            return Err("File is not in iNES file format".to_string());
        }

        let ines_ver = (raw[7] >> 2) & 0b0011;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b0001 != 0;

        let prg_rom_size = raw[4] as usize * ROM::PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * ROM::CHR_ROM_PAGE_SIZE;

        let has_trainer = raw[6] & 0b0100 != 0;
        let has_save_ram = raw[6] & 0b0010 != 0;
        let prg_rom_start = 16 + if has_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let mut rom = ROM::new();
        rom.mapper_id = (raw[7] & 0b1111_0000) | (raw[6] >> 4);
        rom.is_prg_rom_mirror = prg_rom_size == ROM::PRG_ROM_PAGE_SIZE;
        rom.is_chr_ram = chr_rom_size == 0;
        rom.has_save_ram = has_save_ram;
        rom.prg_rom = raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec();
        rom.chr_rom = if rom.is_chr_ram {
            vec![0; ROM::CHR_ROM_PAGE_SIZE]
        } else {
            raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
        };
        rom.screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        println!("ROM: mapper: {}, trainer: {}, save_ram: {}, screen_mirroring: {:?}, \
            is_prg_rom_mirroring: {}, is_chr_ram: {}, prg_rom_size: 0x{:x}, chr_rom_size: 0x{:x}",
            rom.mapper_id, has_trainer, rom.has_save_ram, rom.screen_mirroring,
            rom.is_prg_rom_mirror, rom.is_chr_ram, prg_rom_size, chr_rom_size);

        return Ok(rom);
    }

    #[inline]
    pub fn read_prg_byte(&mut self, address: u16) -> u8 {
        let mirror_address = self.mirror_prg_address(address);
        match self.mapper_id {
            0 => self.mapper0.read_prg_byte(mirror_address, &self.prg_rom),
            1 => self.mapper1.read_prg_byte(mirror_address, &self.prg_rom),
            2 => self.mapper2.read_prg_byte(mirror_address, &self.prg_rom),
            3 => self.mapper3.read_prg_byte(mirror_address, &self.prg_rom),
            4 => self.mapper4.read_prg_byte(mirror_address, &self.prg_rom),
            66 => self.mapper66.read_prg_byte(mirror_address, &self.prg_rom),
            _ => panic!("Unsupported mapper: {}", self.mapper_id)
        }
    }

    #[inline]
    pub fn write_prg_byte(&mut self, address: u16, data: u8) {
        match self.mapper_id {
            0 => self.mapper0.write_mapper(address, data),
            1 => {
                self.mapper1.write_mapper(address, data);
                self.screen_mirroring = self.mapper1.screen_mirroring.clone();
            },
            2 => self.mapper2.write_mapper(address, data),
            3 => self.mapper3.write_mapper(address, data),
            4 => {
                self.mapper4.write_mapper(address, data);
                self.screen_mirroring = self.mapper4.screen_mirroring.clone();
            },
            66 => self.mapper66.write_mapper(address, data),
            _ => panic!("Attempt to write to Cartridge PRG ROM space: 0x{:0>4X}", address)
        }
    }

    #[inline]
    pub fn read_chr_byte(&self, address: u16) -> u8 {
        match self.mapper_id {
            0 => self.mapper0.read_chr_byte(address, &self.chr_rom),
            1 => self.mapper1.read_chr_byte(address, &self.chr_rom),
            2 => self.mapper2.read_chr_byte(address, &self.chr_rom),
            3 => self.mapper3.read_chr_byte(address, &self.chr_rom),
            4 => self.mapper4.read_chr_byte(address, &self.chr_rom),
            66 => self.mapper66.read_chr_byte(address, &self.chr_rom),
            _ => panic!("Unsupported mapper: {}", self.mapper_id),
        }
    }

    #[inline]
    pub fn write_chr_byte(&mut self, address: u16, data: u8) {
        if self.is_chr_ram {
            self.chr_rom[address as usize] = data;
        } else {
            println!("[WARNING] Attempt to write to Cartridge CHR ROM space: 0x{:0>4X}", address)
        }
    }

    #[inline]
    fn mirror_prg_address(&mut self, address: u16) -> u16 {
        let mut offset = address - Memory::PRG_ROM_START;
        if self.is_prg_rom_mirror && address >= ROM::PRG_ROM_PAGE_SIZE as u16 {
            offset = offset % ROM::PRG_ROM_PAGE_SIZE as u16;
        }
        Memory::PRG_ROM_START + offset
    }

    #[inline]
    pub fn get_prg_bank_count(&self) -> usize {
        self.prg_rom.len() / ROM::PRG_ROM_PAGE_SIZE
    }

    #[inline]
    pub fn get_chr_bank_count(&self) -> usize {
        self.chr_rom.len() / ROM::CHR_ROM_PAGE_SIZE
    }
}
