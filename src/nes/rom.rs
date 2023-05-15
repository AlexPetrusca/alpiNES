pub mod registers;

use std::fs;
use std::fs::File;
use std::io::Read;
use crate::nes::cpu::mem::Memory;
use crate::nes::rom::registers::shift::ShiftRegister;

macro_rules! prg_bank0_range { () => {0x8000..=0xBFFF} }
macro_rules! prg_bank1_range { () => {0xC000..=0xFFFF} }

macro_rules! chr_bank0_range { () => {0x0000..=0x0FFF} }
macro_rules! chr_bank1_range { () => {0x1000..=0x1FFF} }

macro_rules! mapper1_control_range { () => {0x8000..=0x9FFF} }
macro_rules! mapper1_chr0_range { () => {0xA000..=0xBFFF} }
macro_rules! mapper1_chr1_range { () => {0xC000..=0xDFFF} }
macro_rules! mapper1_prg_range { () => {0xE000..=0xFFFF} }

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
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
    pub is_prg_rom_mirror: bool,
    pub is_chr_ram: bool,

    pub prg_bank_select: u8, // mapper2, mapper66
    pub chr_bank_select: u8, // mapper3, mapper66

    pub shift_register: ShiftRegister, // mapper1
    pub prg_bank_select_mode: u8, // mapper1
    pub chr_bank_select_mode: u8, // mapper1
    pub chr_bank0_select: u8, // mapper1
    pub chr_bank1_select: u8, // mapper1
}

impl ROM {
    const NES_SIGNATURE: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
    pub const CHR_ROM_PAGE_SIZE: usize = 0x2000; // 8kB
    pub const PRG_ROM_PAGE_SIZE: usize = 0x4000; // 16kB

    pub fn new() -> Self {
        ROM {
            prg_rom: Vec::new(),
            chr_rom: Vec::new(),
            mapper: 0,
            screen_mirroring: Mirroring::Horizontal,
            is_prg_rom_mirror: false,
            is_chr_ram: false,

            chr_bank_select: 0,
            prg_bank_select: 0,

            shift_register: ShiftRegister::new(),
            prg_bank_select_mode: 3,
            chr_bank_select_mode: 0,
            chr_bank0_select: 0,
            chr_bank1_select: 0,
        }
    }

    pub fn from_filepath(filepath: &str) -> Result<ROM, String> {
        let mut file = File::open(filepath).expect("no file found");
        let metadata = fs::metadata(filepath).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        file.read(&mut buffer).expect("buffer overflow");
        ROM::from_buffer(&buffer)
    }

    pub fn from_buffer(raw: &Vec<u8>) -> Result<ROM, String> {
        if &raw[0..4] != ROM::NES_SIGNATURE {
            return Err("File is not in iNES file format".to_string());
        }

        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = raw[4] as usize * ROM::PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * ROM::CHR_ROM_PAGE_SIZE;

        let is_prg_rom_mirror = prg_rom_size == ROM::PRG_ROM_PAGE_SIZE;
        let is_chr_ram = chr_rom_size == 0;

        let has_trainer = raw[6] & 0b100 != 0;
        let prg_rom_start = 16 + if has_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let prg_rom = raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec();
        let chr_rom = if is_chr_ram {
            vec![0; ROM::CHR_ROM_PAGE_SIZE]
        } else {
            raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
        };

        println!("ROM: mapper: {}, trainer: {}, screen_mirroring: {:?}, is_prg_rom_mirroring: {}, is_chr_ram: {}, prg_rom_size: 0x{:x}, chr_rom_size: 0x{:x}",
            mapper, has_trainer, &screen_mirroring, is_prg_rom_mirror, is_chr_ram, prg_rom_size, chr_rom_size);

        Ok(ROM {
            prg_rom,
            chr_rom,
            mapper,
            screen_mirroring,
            is_prg_rom_mirror,
            is_chr_ram,
            prg_bank_select: 0,
            chr_bank_select: 0,
            shift_register: ShiftRegister::new(),
            prg_bank_select_mode: 3,
            chr_bank_select_mode: 0,
            chr_bank0_select: 0,
            chr_bank1_select: 0,
        })
    }

    #[inline]
    pub fn read_prg_byte(&mut self, address: u16) -> u8 {
        let mirror_address = self.mirror_prg_address(address);
        match self.mapper {
            0 | 3 => {
                self.prg_rom[(mirror_address - 0x8000) as usize]
            },
            1 => {
                match self.prg_bank_select_mode {
                    0 | 1 => {
                        // switch 32 KB at $8000, ignoring low bit of bank number
                        let prg_bank_select = self.prg_bank_select & 0b1111_1110;
                        let bank_start = 2 * ROM::PRG_ROM_PAGE_SIZE * prg_bank_select as usize;
                        self.prg_rom[(bank_start + (mirror_address - 0x8000) as usize) % self.prg_rom.len()]
                    },
                    2 => {
                        // fix first bank at $8000 and switch 16 KB bank at $C000
                        match mirror_address {
                            prg_bank0_range!() => {
                                self.prg_rom[(mirror_address as usize - 0x8000) % self.prg_rom.len()]
                            },
                            prg_bank1_range!() => {
                                let bank_start = ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                                self.prg_rom[(bank_start + (mirror_address - 0xC000) as usize) % self.prg_rom.len()]
                            },
                            _ => {
                                panic!("Address out of range on mapper {}: {}", self.mapper, mirror_address);
                            }
                        }
                    },
                    3 => {
                        // fix last bank at $C000 and switch 16 KB bank at $8000
                        match mirror_address {
                            prg_bank0_range!() => {
                                let bank_start = ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                                self.prg_rom[(bank_start + (mirror_address - 0x8000) as usize) % self.prg_rom.len()]
                            },
                            prg_bank1_range!() => {
                                let last_bank_start = self.prg_rom.len() - ROM::PRG_ROM_PAGE_SIZE;
                                self.prg_rom[last_bank_start + (mirror_address - 0xC000) as usize]
                            },
                            _ => {
                                panic!("Address out of range on mapper {}: {}", self.mapper, mirror_address);
                            }
                        }
                    },
                    _ => panic!("can't be")
                }
            }
            2 => {
                match mirror_address {
                    prg_bank0_range!() => {
                        let bank_start = ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                        self.prg_rom[(bank_start + (mirror_address - 0x8000) as usize) % self.prg_rom.len()]
                    },
                    prg_bank1_range!() => {
                        let last_bank_start = self.prg_rom.len() - ROM::PRG_ROM_PAGE_SIZE;
                        self.prg_rom[last_bank_start + (mirror_address - 0xC000) as usize]
                    },
                    _ => {
                        panic!("Address out of range on mapper {}: {}", self.mapper, mirror_address);
                    }
                }
            },
            66 => {
                let bank_start = 2 * ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                self.prg_rom[(bank_start + (mirror_address - 0x8000) as usize) % self.prg_rom.len()]
            },
            _ => {
                panic!("Unsupported mapper: {}", self.mapper);
            }
        }
    }

    #[inline]
    pub fn write_prg_byte(&mut self, address: u16, data: u8) {
        match self.mapper {
            1 => {
                self.shift_register.write(data);
                if self.shift_register.is_fifth_write() {
                    // print!("mapper1: [0x{:x} => 0b{:0>8b}] ", address, self.shift_register.value);
                    let value = self.shift_register.value;
                    match address {
                        mapper1_control_range!() => {
                            // 4bit0
                            // -----
                            // CPPMM
                            // |||||
                            // |||++- Mirroring (0: one-screen, lower bank; 1: one-screen, upper bank;
                            // |||               2: vertical; 3: horizontal)
                            // |++--- PRG ROM bank mode (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
                            // |                         2: fix first bank at $8000 and switch 16 KB bank at $C000;
                            // |                         3: fix last bank at $C000 and switch 16 KB bank at $8000)
                            // +----- CHR ROM bank mode (0: switch 8 KB at a time; 1: switch two separate 4 KB banks)
                            self.screen_mirroring = match value & 0b0000_0011 {
                                0 => Mirroring::OneScreenLower,
                                1 => Mirroring::OneScreenUpper,
                                2 => Mirroring::Vertical,
                                3 => Mirroring::Horizontal,
                                _ => panic!("can't be")
                            };
                            self.prg_bank_select_mode = (value & 0b0000_1100) >> 2;
                            self.chr_bank_select_mode = (value & 0b0001_0000) >> 4;
                            // println!("control => screen_mirroring: {:?}, chr_bank_select_mode: {}, prg_bank_select_mode: {}",
                            //     self.screen_mirroring, self.chr_bank_select_mode, self.prg_bank_select_mode);
                        },
                        mapper1_chr0_range!() => {
                            // 4bit0
                            // -----
                            // CCCCC
                            // |||||
                            // +++++- Select 4 KB or 8 KB CHR bank at PPU $0000 (low bit ignored in 8 KB mode)
                            if self.chr_bank_select_mode == 1 { // todo: use enum
                                self.chr_bank0_select = value;
                                // println!("chr0 => chr_bank0_select: {}", self.chr_bank0_select);
                            } else {
                                self.chr_bank_select = value;
                                // println!("chr0 => chr_bank_select: {}", self.chr_bank_select);
                            }
                        },
                        mapper1_chr1_range!() => {
                            // 4bit0
                            // -----
                            // CCCCC
                            // |||||
                            // +++++- Select 4 KB CHR bank at PPU $1000 (ignored in 8 KB mode)
                            if self.chr_bank_select_mode == 1 { // todo: use enum
                                self.chr_bank1_select = value;
                                // println!("chr1 => chr_bank1_select: {}", self.chr_bank1_select);
                            }
                        },
                        mapper1_prg_range!() => {
                            // 4bit0
                            // -----
                            // RPPPP
                            // |||||
                            // |++++- Select 16 KB PRG ROM bank (low bit ignored in 32 KB mode)
                            // +----- MMC1B and later: PRG RAM chip enable (0: enabled; 1: disabled; ignored on MMC1A)
                            //        MMC1A: Bit 3 bypasses fixed bank logic in 16K mode (0: affected; 1: bypassed)
                            self.prg_bank_select = value & 0b0000_1111;
                            // println!("prg => prg_bank_select: {}", self.prg_bank_select);
                        },
                        _ => {
                            panic!("Address out of range on mapper {}: {}", self.mapper, address);
                        }
                    }
                }
            },
            2 => {
                self.prg_bank_select = data & 0b0000_1111;
            },
            3 => {
                self.chr_bank_select = data;
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
        match self.mapper {
            0 | 2 => {
                self.chr_rom[address as usize]
            },
            1 => {
                if self.chr_bank_select_mode == 0 {
                    // switch 8 KB at a time
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 2) * self.chr_bank_select as usize;
                    self.chr_rom[(bank_start + address as usize) % self.chr_rom.len()]
                } else {
                    // switch two separate 4 KB banks
                    match address {
                        chr_bank0_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 2) * self.chr_bank0_select as usize;
                            self.chr_rom[(bank_start + address as usize) % self.chr_rom.len()]
                        },
                        chr_bank1_range!() => {
                            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 2) * self.chr_bank1_select as usize;
                            self.chr_rom[(bank_start + address as usize - 0x1000) % self.chr_rom.len()]
                        },
                        _ => {
                            panic!("Address out of range on mapper {}: {}", self.mapper, address);
                        }
                    }
                }
            },
            3 | 66 => {
                let bank_start = ROM::CHR_ROM_PAGE_SIZE * self.chr_bank_select as usize;
                self.chr_rom[(bank_start + address as usize) % self.chr_rom.len()]
            },
            _ => {
                panic!("Unsupported mapper: {}", self.mapper);
            }
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
