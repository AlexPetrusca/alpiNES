use crate::nes::rom::mappers::mapper::Mapper;
use crate::nes::rom::ROM;

#[derive(Clone)]
pub struct Mapper3 {
    pub chr_bank_select: u8,
}

impl Mapper3 {
    pub fn new() -> Self {
        Mapper3 {
            chr_bank_select: 0,
        }
    }
}

impl Mapper for Mapper3 {
    fn read_prg_byte(&mut self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        prg_rom[(address - 0x8000) as usize]
    }

    fn read_chr_byte(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        let bank_start = ROM::CHR_ROM_PAGE_SIZE * self.chr_bank_select as usize;
        chr_rom[(bank_start + address as usize) % chr_rom.len()]
    }

    fn write_mapper(&mut self, _address: u16, data: u8) {
        self.chr_bank_select = data;
    }
}
