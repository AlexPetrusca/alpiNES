use crate::nes::rom::mappers::mapper::Mapper;
use crate::nes::rom::registers::shift::ShiftRegister;
use crate::nes::rom::{Mirroring, ROM};

macro_rules! prg_bank0_range { () => {0x8000..=0xBFFF} }
macro_rules! prg_bank1_range { () => {0xC000..=0xFFFF} }

macro_rules! chr_bank0_range { () => {0x0000..=0x0FFF} }
macro_rules! chr_bank1_range { () => {0x1000..=0x1FFF} }

macro_rules! mapper1_control_range { () => {0x8000..=0x9FFF} }
macro_rules! mapper1_chr0_range { () => {0xA000..=0xBFFF} }
macro_rules! mapper1_chr1_range { () => {0xC000..=0xDFFF} }
macro_rules! mapper1_prg_range { () => {0xE000..=0xFFFF} }

#[derive(Clone)]
pub struct Mapper1 {
    pub shift_register: ShiftRegister,
    pub prg_bank_select_mode: u8,
    pub chr_bank_select_mode: u8,
    pub prg_bank_select: u8,
    pub chr_bank_select: u8,
    pub chr_bank0_select: u8,
    pub chr_bank1_select: u8,
    pub screen_mirroring: Mirroring,
}

impl Mapper1 {
    pub fn new() -> Self {
        Mapper1 {
            shift_register: ShiftRegister::new(),
            prg_bank_select_mode: 3,
            chr_bank_select_mode: 0,
            prg_bank_select: 0,
            chr_bank_select: 0,
            chr_bank0_select: 0,
            chr_bank1_select: 0,
            screen_mirroring: Mirroring::Horizontal,
        }
    }
}

impl Mapper for Mapper1 {
    fn read_prg_byte(&mut self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        match self.prg_bank_select_mode {
            0 | 1 => {
                // switch 32 KB at $8000, ignoring low bit of bank number
                let prg_bank_select = self.prg_bank_select & 0b1111_1110;
                let bank_start = 2 * ROM::PRG_ROM_PAGE_SIZE * prg_bank_select as usize;
                prg_rom[(bank_start + (address - 0x8000) as usize) % prg_rom.len()]
            },
            2 => {
                // fix first bank at $8000 and switch 16 KB bank at $C000
                match address {
                    prg_bank0_range!() => {
                        prg_rom[(address as usize - 0x8000) % prg_rom.len()]
                    },
                    prg_bank1_range!() => {
                        let bank_start = ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                        prg_rom[(bank_start + (address - 0xC000) as usize) % prg_rom.len()]
                    },
                    _ => {
                        panic!("Address out of range on mapper 1: {}", address);
                    }
                }
            },
            3 => {
                // fix last bank at $C000 and switch 16 KB bank at $8000
                match address {
                    prg_bank0_range!() => {
                        let bank_start = ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
                        prg_rom[(bank_start + (address - 0x8000) as usize) % prg_rom.len()]
                    },
                    prg_bank1_range!() => {
                        let last_bank_start = prg_rom.len() - ROM::PRG_ROM_PAGE_SIZE;
                        prg_rom[last_bank_start + (address - 0xC000) as usize]
                    },
                    _ => {
                        panic!("Address out of range on mapper 1: {}", address);
                    }
                }
            },
            _ => panic!("can't be")
        }
    }

    fn read_chr_byte(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        if self.chr_bank_select_mode == 0 {
            // switch 8 KB at a time
            let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 2) * self.chr_bank_select as usize;
            chr_rom[(bank_start + address as usize) % chr_rom.len()]
        } else {
            // switch two separate 4 KB banks
            match address {
                chr_bank0_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 2) * self.chr_bank0_select as usize;
                    chr_rom[(bank_start + address as usize) % chr_rom.len()]
                },
                chr_bank1_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 2) * self.chr_bank1_select as usize;
                    chr_rom[(bank_start + address as usize - 0x1000) % chr_rom.len()]
                },
                _ => {
                    panic!("Address out of range on mapper 1: {}", address);
                }
            }
        }
    }

    fn write_mapper(&mut self, address: u16, data: u8) {
        self.shift_register.write(data);
        if self.shift_register.is_fifth_write() {
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
                },
                mapper1_chr0_range!() => {
                    // 4bit0
                    // -----
                    // CCCCC
                    // |||||
                    // +++++- Select 4 KB or 8 KB CHR bank at PPU $0000 (low bit ignored in 8 KB mode)
                    if self.chr_bank_select_mode == 1 { // todo: use enum
                        self.chr_bank0_select = value;
                    } else {
                        self.chr_bank_select = value;
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
                },
                _ => {
                    panic!("Address out of range on mapper 1: {}", address);
                }
            }
        }
    }
}
