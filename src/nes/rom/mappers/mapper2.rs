use std::rc::Rc;
use crate::nes::rom::mappers::mapper::Mapper;
use crate::nes::rom::ROM;

macro_rules! prg_bank0_range { () => {0x8000..=0xBFFF} }
macro_rules! prg_bank1_range { () => {0xC000..=0xFFFF} }

#[derive(Clone)]
pub struct Mapper2 {
    pub prg_bank_select: u8,
}

impl Mapper2 {
    pub fn new() -> Self {
        Mapper2 {
            prg_bank_select: 0
        }
    }
}

impl Mapper for Mapper2 {
    fn read_prg_byte(&mut self, address: u16, prg_rom: &Vec<u8>) -> u8 {
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
                panic!("Address out of range on mapper 2: {}", address);
            }
        }
    }

    fn read_chr_byte(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        chr_rom[address as usize]
    }

    fn write_mapper(&mut self, _address: u16, data: u8) {
        self.prg_bank_select = data & 0b0000_1111;
    }
}
