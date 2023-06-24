use crate::nes::rom::mappers::mapper::Mapper;
use crate::nes::rom::{Mirroring, ROM};

macro_rules! bank_select_data_range { () => {0x8000..=0x9FFF} }
macro_rules! bank_mirror_protect_range { () => {0xA000..=0xBFFF} }
macro_rules! irq_latch_reload_range { () => {0xC000..=0xDFFF} }
macro_rules! irq_disable_enable_range { () => {0xE000..=0xFFFF} }

macro_rules! prg_subbank0_range { () => {0x8000..=0x9FFF} }
macro_rules! prg_subbank1_range { () => {0xA000..=0xBFFF} }
macro_rules! prg_subbank2_range { () => {0xC000..=0xDFFF} }
macro_rules! prg_subbank3_range { () => {0xE000..=0xFFFF} }

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

#[derive(Clone)]
pub struct Mapper4 {
    pub bank_select: u8,
    pub prg_bank_select_mode: u8,
    pub chr_bank_select_mode: u8,
    pub prg_bank0_select:u8,
    pub prg_bank1_select:u8,
    pub chr_bank0_select: u8,
    pub chr_bank1_select: u8,
    pub chr_bank0_1kb_select: u8,
    pub chr_bank1_1kb_select: u8,
    pub chr_bank2_1kb_select: u8,
    pub chr_bank3_1kb_select: u8,
    pub chr_bank0_2kb_select: u8,
    pub chr_bank1_2kb_select: u8,

    pub screen_mirroring: Mirroring,

    pub irq_counter: u8,
    pub irq_latch: u8,
    pub irq_reload: bool,
    pub irq_enable: bool,
    pub irq_flag: bool,
}

impl Mapper4 {
    pub fn new() -> Self {
        Mapper4 {
            bank_select: 0,
            prg_bank_select_mode: 0,
            chr_bank_select_mode: 0,
            prg_bank0_select: 0,
            prg_bank1_select: 0,
            chr_bank0_select: 0,
            chr_bank1_select: 0,
            chr_bank0_1kb_select: 0,
            chr_bank1_1kb_select: 0,
            chr_bank2_1kb_select: 0,
            chr_bank3_1kb_select: 0,
            chr_bank0_2kb_select: 0,
            chr_bank1_2kb_select: 0,

            screen_mirroring: Mirroring::Horizontal,

            irq_counter: 0,
            irq_latch: 0,
            irq_reload: false,
            irq_enable: false,
            irq_flag: false,
        }
    }

    #[inline]
    pub fn decrement_irq_counter(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_counter == 0 && self.irq_enable {
            self.set_irq();
        }
    }

    #[inline]
    pub fn poll_irq(&mut self) -> bool {
        return self.irq_flag;
    }

    #[inline]
    pub fn set_irq(&mut self) {
        self.irq_flag = true;
    }

    #[inline]
    pub fn clear_irq(&mut self) {
        self.irq_flag = false
    }
}

impl Mapper for Mapper4 {
    fn read_prg_byte(&mut self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        match address {
            prg_subbank0_range!() => {
                if self.prg_bank_select_mode == 0 {
                    // $8000-$9FFF swappable
                    // 110: R6: Select 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF)
                    let bank_start = (ROM::PRG_ROM_PAGE_SIZE / 2) * self.prg_bank0_select as usize;
                    prg_rom[(bank_start + (address - 0x8000) as usize) % prg_rom.len()]
                } else {
                    // $8000-$9FFF fixed to second-last bank
                    let last_bank_start = prg_rom.len() - ROM::PRG_ROM_PAGE_SIZE;
                    prg_rom[last_bank_start + (address - 0x8000) as usize]
                }
            },
            prg_subbank1_range!() => {
                // 111: R7: Select 8 KB PRG ROM bank at $A000-$BFFF
                let bank_start = (ROM::PRG_ROM_PAGE_SIZE / 2) * self.prg_bank1_select as usize;
                prg_rom[(bank_start + (address - 0xA000) as usize) % prg_rom.len()]
            },
            prg_subbank2_range!() => {
                if self.prg_bank_select_mode == 0 {
                    // $C000-$DFFF fixed to second-last bank;
                    let last_bank_start = prg_rom.len() - ROM::PRG_ROM_PAGE_SIZE;
                    prg_rom[last_bank_start + (address - 0xC000) as usize]
                } else {
                    // $C000-$DFFF swappable
                    // 110: R6: Select 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF)
                    let bank_start = (ROM::PRG_ROM_PAGE_SIZE / 2) * self.prg_bank0_select as usize;
                    prg_rom[(bank_start + (address - 0xC000) as usize) % prg_rom.len()]
                }
            },
            prg_subbank3_range!() => {
                // $E000-$FFFF: 8 KB PRG ROM bank, fixed to the last bank
                let last_bank_start = prg_rom.len() - (ROM::PRG_ROM_PAGE_SIZE / 2);
                prg_rom[last_bank_start + (address - 0xE000) as usize]
            },
            _ => panic!("can't be")
        }
    }

    fn read_chr_byte(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        if self.chr_bank_select_mode == 0 {
            match address {
                chr_subbank0_2kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 4) * (self.chr_bank0_2kb_select / 2) as usize;
                    chr_rom[(bank_start + address as usize) % chr_rom.len()]
                },
                chr_subbank1_2kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 4) * (self.chr_bank1_2kb_select / 2) as usize;
                    chr_rom[(bank_start + address as usize - 0x0800) % chr_rom.len()]
                },
                chr_subbank4_1kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank0_1kb_select as usize;
                    chr_rom[(bank_start + address as usize - 0x1000) % chr_rom.len()]
                },
                chr_subbank5_1kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank1_1kb_select as usize;
                    chr_rom[(bank_start + address as usize - 0x1400) % chr_rom.len()]
                },
                chr_subbank6_1kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank2_1kb_select as usize;
                    chr_rom[(bank_start + address as usize - 0x1800) % chr_rom.len()]
                },
                chr_subbank7_1kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank3_1kb_select as usize;
                    chr_rom[(bank_start + address as usize - 0x1C00) % chr_rom.len()]
                },
                _ => panic!("Address out of range on mapper 4: {}", address)
            }
        } else {
            match address {
                chr_subbank0_1kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank0_1kb_select as usize;
                    chr_rom[(bank_start + address as usize) % chr_rom.len()]
                },
                chr_subbank1_1kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank1_1kb_select as usize;
                    chr_rom[(bank_start + address as usize - 0x0400) % chr_rom.len()]
                },
                chr_subbank2_1kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank2_1kb_select as usize;
                    chr_rom[(bank_start + address as usize - 0x0800) % chr_rom.len()]
                },
                chr_subbank3_1kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 8) * self.chr_bank3_1kb_select as usize;
                    chr_rom[(bank_start + address as usize - 0x0C00) % chr_rom.len()]
                },
                chr_subbank2_2kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 4) * (self.chr_bank0_2kb_select / 2) as usize;
                    chr_rom[(bank_start + address as usize - 0x1000) % chr_rom.len()]
                },
                chr_subbank3_2kb_range!() => {
                    let bank_start = (ROM::CHR_ROM_PAGE_SIZE / 4) * (self.chr_bank1_2kb_select / 2) as usize;
                    chr_rom[(bank_start + address as usize - 0x1800) % chr_rom.len()]
                },
                _ => panic!("Address out of range on mapper 4: {}", address)
            }
        }
    }

    fn write_mapper(&mut self, address: u16, data: u8) {
        match address {
            bank_select_data_range!() => {
                if address % 2 == 0 {
                    // bank select
                    // println!("mapper4: bank select => 0b{:0>8b}", data);
                    self.bank_select = data & 0b0000_0111;
                    self.prg_bank_select_mode = (data & 0b0100_0000) >> 6;
                    self.chr_bank_select_mode = (data & 0b1000_0000) >> 7;
                } else {
                    // bank data
                    // println!("mapper4: bank data => 0x{:0>4x}", data);
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
            bank_mirror_protect_range!() => {
                if address % 2 == 0 {
                    // mirroring
                    // println!("mapper4: mirroring => {:?}", self.screen_mirroring);
                    self.screen_mirroring = if data & 1 == 0 { Mirroring::Vertical } else { Mirroring::Horizontal };
                } else {
                    // prg ram protect
                    // println!("mapper4: prg ram protect => 0b{:0>8b}", data);
                }
            },
            irq_latch_reload_range!() => {
                // todo: implement
                if address % 2 == 0 {
                    // irq latch
                    println!("mapper4: irq latch => {}", data);
                    self.irq_latch = data;
                } else {
                    // irq reload
                    println!("mapper4: irq reload");
                    self.irq_reload = true;
                    self.irq_counter = 0;
                }
            },
            irq_disable_enable_range!() => {
                // todo: implement
                if address % 2 == 0 {
                    // irq disable
                    println!("mapper4: irq disable");
                    self.irq_enable = false;
                    self.clear_irq();
                } else {
                    // irq enable
                    println!("mapper4: irq enable");
                    self.irq_enable = true;
                }
            },
            _ => panic!("Address out of range on mapper 4: {}", address)
        }
    }
}
