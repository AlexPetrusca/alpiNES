use std::fs;
use std::fs::File;
use std::io::Read;
use crate::nes::cpu::mem::Memory;

#[derive(Debug, PartialEq, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
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

    pub prg_bank_select: u8, // todo: move? mapper2
    pub chr_bank_select: u8, // todo: move? mapper3
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
            chr_bank_select: 0
        })
    }

    #[inline]
    pub fn read_prg_byte(&mut self, address: u16) -> u8 {
        let mirror_address = self.mirror_prg_address(address);
        match self.mapper {
            0 => {
                self.prg_rom[(mirror_address - 0x8000) as usize]
            },
            2 => {
                match mirror_address {
                    0x8000..=0xBFFF => {
                        let bank_start = ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                        self.prg_rom[(bank_start + (mirror_address - 0x8000) as usize) % self.prg_rom.len()]
                    },
                    0xC000..=0xFFFF => {
                        let last_bank_start = self.prg_rom.len() - ROM::PRG_ROM_PAGE_SIZE;
                        self.prg_rom[last_bank_start + (mirror_address - 0xC000) as usize]
                    },
                    _ => {
                        panic!("Address out of range on mapper {}: {}", self.mapper, mirror_address);
                    }
                }
            },
            3 => {
                self.prg_rom[(mirror_address - 0x8000) as usize]
            },
            66 => {
                let bank_start = 2 * ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                self.prg_rom[bank_start + (mirror_address - 0x8000) as usize]
            },
            _ => {
                panic!("Unsupported mapper: {}", self.mapper);
            }
        }
    }

    #[inline]
    pub fn write_prg_byte(&mut self, address: u16, data: u8) {
        match self.mapper {
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
            3 => {
                let bank_start = ROM::CHR_ROM_PAGE_SIZE * self.chr_bank_select as usize;
                // todo: is below even correct?
                self.chr_rom[(bank_start + address as usize) % self.chr_rom.len()]
            },
            66 => {
                // todo: merge logic with mapper 3?
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
