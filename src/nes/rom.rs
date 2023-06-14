pub mod registers;
pub mod mappers;

use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use crate::nes::cpu::mem::Memory;
use crate::nes::rom::mappers::mapper0::Mapper0;
use crate::nes::rom::mappers::mapper1::Mapper1;
use crate::nes::rom::mappers::mapper2::Mapper2;
use crate::nes::rom::mappers::mapper::Mapper;
use crate::nes::rom::registers::shift::ShiftRegister;

macro_rules! prg_bank0_range { () => {0x8000..=0xBFFF} }
macro_rules! prg_bank1_range { () => {0xC000..=0xFFFF} }

macro_rules! prg_subbank0_range { () => {0x8000..=0x9FFF} }
macro_rules! prg_subbank1_range { () => {0xA000..=0xBFFF} }
macro_rules! prg_subbank2_range { () => {0xC000..=0xDFFF} }
macro_rules! prg_subbank3_range { () => {0xE000..=0xFFFF} }

macro_rules! prg_subbank0_range { () => {0x8000..=0x9FFF} }
macro_rules! prg_subbank1_range { () => {0xA000..=0xBFFF} }
macro_rules! prg_subbank2_range { () => {0xC000..=0xDFFF} }
macro_rules! prg_subbank3_range { () => {0xE000..=0xFFFF} }

macro_rules! chr_bank0_range { () => {0x0000..=0x0FFF} }
macro_rules! chr_bank1_range { () => {0x1000..=0x1FFF} }

macro_rules! chr_subbank0_1kb_range { () => {0x0000..=0x03FF} }
macro_rules! chr_subbank1_1kb_range { () => {0x0400..=0x07FF} }
macro_rules! chr_subbank2_1kb_range { () => {0x0800..=0x0BFF} }
macro_rules! chr_subbank3_1kb_range { () => {0x0C00..=0x0FFF} }
macro_rules! chr_subbank4_1kb_range { () => {0x1000..=0x13FF} }
macro_rules! chr_subbank5_1kb_range { () => {0x1400..=0x17FF} }
macro_rules! chr_subbank6_1kb_range { () => {0x1800..=0x1BFF} }
macro_rules! chr_subbank7_1kb_range { () => {0x1C00..=0x1FFF} }

macro_rules! chr_subbank0_2kb_range { () => {0x0000..=0x07FF} }
macro_rules! chr_subbank1_2kb_range { () => {0x0800..=0x0FFF} }
macro_rules! chr_subbank2_2kb_range { () => {0x1000..=0x17FF} }
macro_rules! chr_subbank3_2kb_range { () => {0x1800..=0x1FFF} }

macro_rules! mapper1_control_range { () => {0x8000..=0x9FFF} }
macro_rules! mapper1_chr0_range { () => {0xA000..=0xBFFF} }
macro_rules! mapper1_chr1_range { () => {0xC000..=0xDFFF} }
macro_rules! mapper1_prg_range { () => {0xE000..=0xFFFF} }

macro_rules! mapper4_bank_select_data_range { () => {0x8000..=0x9FFF} }
macro_rules! mapper4_bank_mirror_protect_range { () => {0xA000..=0xBFFF} }
macro_rules! mapper4_irq_latch_reload_range { () => {0xC000..=0xDFFF} }
macro_rules! mapper4_irq_disable_enable_range { () => {0xE000..=0xFFFF} }

#[derive(Debug, PartialEq, Clone)]
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
    pub screen_mirroring: Mirroring,
    pub is_prg_rom_mirror: bool,
    pub is_chr_ram: bool,
    pub has_save_ram: bool,

    pub mapper0: Mapper0,
    pub mapper1: Mapper1,
    pub mapper2: Mapper2,

    pub prg_bank_select: u8, // mapper2, mapper66
    pub chr_bank_select: u8, // mapper3, mapper66

    pub shift_register: ShiftRegister, // mapper1
    pub prg_bank_select_mode: u8, // mapper1, mapper4
    pub chr_bank_select_mode: u8, // mapper1, mapper4
    pub chr_bank0_select: u8, // mapper1, mapper4
    pub chr_bank1_select: u8, // mapper1, mapper4

    pub bank_select: u8, // mapper4
    pub chr_bank0_2kb_select: u8, // mapper4
    pub chr_bank1_2kb_select: u8, // mapper4
    pub chr_bank0_1kb_select: u8, // mapper4
    pub chr_bank1_1kb_select: u8, // mapper4
    pub chr_bank2_1kb_select: u8, // mapper4
    pub chr_bank3_1kb_select: u8, // mapper4
    pub prg_bank0_select:u8, // mapper4
    pub prg_bank1_select:u8, // mapper4
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
            screen_mirroring: Mirroring::Horizontal,
            is_prg_rom_mirror: false,
            is_chr_ram: false,
            has_save_ram: false,

            mapper0: Mapper0::new(),
            mapper1: Mapper1::new(),
            mapper2: Mapper2::new(),

            chr_bank_select: 0,
            prg_bank_select: 0,

            shift_register: ShiftRegister::new(),
            prg_bank_select_mode: 0,
            chr_bank_select_mode: 0,
            chr_bank0_select: 0,
            chr_bank1_select: 0,

            bank_select: 0,
            chr_bank0_2kb_select: 0,
            chr_bank1_2kb_select: 0,
            chr_bank0_1kb_select: 0,
            chr_bank1_1kb_select: 0,
            chr_bank2_1kb_select: 0,
            chr_bank3_1kb_select: 0,
            prg_bank0_select: 0,
            prg_bank1_select: 0,
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
            3 => {
                self.prg_rom[(mirror_address - 0x8000) as usize]
            },
            4 => {
                match mirror_address {
                    prg_subbank0_range!() => {
                        if self.prg_bank_select_mode == 0 {
                            // $8000-$9FFF swappable
                            // 110: R6: Select 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF)
                            let bank_start = (ROM::PRG_ROM_PAGE_SIZE / 2) * self.prg_bank0_select as usize;
                            self.prg_rom[(bank_start + (mirror_address - 0x8000) as usize) % self.prg_rom.len()]
                        } else {
                            // $8000-$9FFF fixed to second-last bank
                            let last_bank_start = self.prg_rom.len() - ROM::PRG_ROM_PAGE_SIZE;
                            self.prg_rom[last_bank_start + (mirror_address - 0x8000) as usize]
                        }
                    },
                    prg_subbank1_range!() => {
                        // 111: R7: Select 8 KB PRG ROM bank at $A000-$BFFF
                        let bank_start = (ROM::PRG_ROM_PAGE_SIZE / 2) * self.prg_bank1_select as usize;
                        self.prg_rom[(bank_start + (mirror_address - 0xA000) as usize) % self.prg_rom.len()]
                    },
                    prg_subbank2_range!() => {
                        if self.prg_bank_select_mode == 0 {
                            // $C000-$DFFF fixed to second-last bank;
                            let last_bank_start = self.prg_rom.len() - ROM::PRG_ROM_PAGE_SIZE;
                            self.prg_rom[last_bank_start + (mirror_address - 0xC000) as usize]
                        } else {
                            // $C000-$DFFF swappable
                            // 110: R6: Select 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF)
                            let bank_start = (ROM::PRG_ROM_PAGE_SIZE / 2) * self.prg_bank0_select as usize;
                            self.prg_rom[(bank_start + (mirror_address - 0xC000) as usize) % self.prg_rom.len()]
                        }
                    },
                    prg_subbank3_range!() => {
                        // $E000-$FFFF: 8 KB PRG ROM bank, fixed to the last bank
                        let last_bank_start = self.prg_rom.len() - (ROM::PRG_ROM_PAGE_SIZE / 2);
                        self.prg_rom[last_bank_start + (mirror_address - 0xE000) as usize]
                    },
                    _ => panic!("can't be")
                }
            },
            66 => {
                let bank_start = 2 * ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                self.prg_rom[(bank_start + (mirror_address - 0x8000) as usize) % self.prg_rom.len()]
            },
            _ => {
                panic!("Unsupported mapper: {}", self.mapper_id);
            }
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
            3 => {
                self.chr_bank_select = data;
            },
            4 => {
                match address {
                    mapper4_bank_select_data_range!() => {
                        if address % 2 == 0 {
                            // bank select
                            println!("mapper4: bank select => 0b{:0>8b}", data);
                            self.bank_select = data & 0b0000_0111;
                            self.prg_bank_select_mode = (data & 0b0100_0000) >> 6;
                            self.chr_bank_select_mode = (data & 0b1000_0000) >> 7;
                        } else {
                            // bank data
                            println!("mapper4: bank data => 0x{:0>4x}", data);
                            match self.bank_select {
                                0 => self.chr_bank0_2kb_select = data,
                                1 => self.chr_bank1_2kb_select = data,
                                2 => self.chr_bank0_1kb_select = data,
                                3 => self.chr_bank1_1kb_select = data,
                                4 => self.chr_bank2_1kb_select = data,
                                5 => self.chr_bank3_1kb_select = data,
                                6 => self.prg_bank0_select = data,
                                7 => self.prg_bank1_select = data,
                                _ => panic!("can't be")
                            }
                        }
                    },
                    mapper4_bank_mirror_protect_range!() => {
                        if address % 2 == 0 {
                            // mirroring
                            let mirroring = if data & 1 == 0 { Mirroring::Vertical } else { Mirroring::Horizontal };
                            println!("mapper4: mirroring => {:?}", mirroring);
                        } else {
                            // prg ram protect
                            println!("mapper4: prg ram protect => 0b{:0>8b}", data);
                        }
                    },
                    mapper4_irq_latch_reload_range!() => {
                        if address % 2 == 0 {
                            // irq latch
                            println!("mapper4: irq latch => {}", data);
                        } else {
                            // irq reload
                            println!("mapper4: irq reload");
                        }
                    },
                    mapper4_irq_disable_enable_range!() => {
                        if address % 2 == 0 {
                            // irq disable
                            println!("mapper4: irq disable");
                        } else {
                            // irq enable
                            println!("mapper4: irq enable");
                        }
                    },
                    _ => {
                        panic!("Address out of range on mapper {}: {}", self.mapper_id, address);
                    }
                }
            },
            66 => {
                self.chr_bank_select = data & 0b0000_0011;
                self.prg_bank_select = (data >> 4) & 0b0000_0011;
            },
            _ => {
                panic!("Attempt to write to Cartridge PRG ROM space: 0x{:0>4X}", address)
            }
        }
    }

    #[inline]
    pub fn read_chr_byte(&self, address: u16) -> u8 {
        match self.mapper_id {
            0 => self.mapper0.read_chr_byte(address, &self.chr_rom),
            1 => self.mapper1.read_chr_byte(address, &self.chr_rom),
            2 => self.mapper2.read_chr_byte(address, &self.chr_rom),
            3 => {
                let bank_start = ROM::CHR_ROM_PAGE_SIZE * self.chr_bank_select as usize;
                self.chr_rom[(bank_start + address as usize) % self.chr_rom.len()]
            },
            4 => {
                if self.chr_bank_select_mode == 0 {
                    match address {
                        chr_subbank0_2kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 4) * (self.chr_bank0_2kb_select / 2) as usize;
                            self.chr_rom[(bank_start + address as usize) % self.chr_rom.len()]
                        },
                        chr_subbank1_2kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 4) * (self.chr_bank1_2kb_select / 2) as usize;
                            self.chr_rom[(bank_start + address as usize - 0x0800) % self.chr_rom.len()]
                        },
                        chr_subbank4_1kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank0_1kb_select as usize;
                            self.chr_rom[(bank_start + address as usize - 0x1000) % self.chr_rom.len()]
                        },
                        chr_subbank5_1kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank1_1kb_select as usize;
                            self.chr_rom[(bank_start + address as usize - 0x1400) % self.chr_rom.len()]
                        },
                        chr_subbank6_1kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank2_1kb_select as usize;
                            self.chr_rom[(bank_start + address as usize - 0x1800) % self.chr_rom.len()]
                        },
                        chr_subbank7_1kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank3_1kb_select as usize;
                            self.chr_rom[(bank_start + address as usize - 0x1C00) % self.chr_rom.len()]
                        },
                        _ => {
                            panic!("Address out of range on mapper {}: {}", self.mapper_id, address);
                        }
                    }
                } else {
                    match address {
                        chr_subbank0_1kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank0_1kb_select as usize;
                            self.chr_rom[(bank_start + address as usize) % self.chr_rom.len()]
                        },
                        chr_subbank1_1kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank1_1kb_select as usize;
                            self.chr_rom[(bank_start + address as usize - 0x0400) % self.chr_rom.len()]
                        },
                        chr_subbank2_1kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank2_1kb_select as usize;
                            self.chr_rom[(bank_start + address as usize - 0x0800) % self.chr_rom.len()]
                        },
                        chr_subbank3_1kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank3_1kb_select as usize;
                            self.chr_rom[(bank_start + address as usize - 0x0C00) % self.chr_rom.len()]
                        },
                        chr_subbank2_2kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 4) * (self.chr_bank0_2kb_select / 2) as usize;
                            self.chr_rom[(bank_start + address as usize - 0x1000) % self.chr_rom.len()]
                        },
                        chr_subbank3_2kb_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 4) * (self.chr_bank1_2kb_select / 2) as usize;
                            self.chr_rom[(bank_start + address as usize - 0x1800) % self.chr_rom.len()]
                        },
                        _ => {
                            panic!("Address out of range on mapper {}: {}", self.mapper_id, address);
                        }
                    }
                }
            },
            66 => {
                let bank_start = ROM::CHR_ROM_PAGE_SIZE * self.chr_bank_select as usize;
                self.chr_rom[(bank_start + address as usize) % self.chr_rom.len()]
            },
            _ => {
                panic!("Unsupported mapper: {}", self.mapper_id);
            },
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
