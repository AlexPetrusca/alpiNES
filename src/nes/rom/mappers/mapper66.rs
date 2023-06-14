use crate::nes::rom::mappers::mapper::Mapper;
use crate::nes::rom::ROM;

#[derive(Clone)]
pub struct Mapper66 {
    pub prg_bank_select: u8,
    pub chr_bank_select: u8,
}

impl Mapper66 {
    pub fn new() -> Self {
        Mapper66 {
            prg_bank_select: 0,
            chr_bank_select: 0,
        }
    }
}

impl Mapper for Mapper66 {
    fn read_prg_byte(&mut self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        let bank_start = 2 * ROM::PRG_ROM_PAGE_SIZE * self.prg_bank_select as usize;
        prg_rom[(bank_start + (address - 0x8000) as usize) % prg_rom.len()]
    }

    fn read_chr_byte(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        let bank_start = ROM::CHR_ROM_PAGE_SIZE * self.chr_bank_select as usize;
        chr_rom[(bank_start + address as usize) % chr_rom.len()]
    }

    fn write_mapper(&mut self, _address: u16, data: u8) {
        self.chr_bank_select = data & 0b0000_0011;
        self.prg_bank_select = (data >> 4) & 0b0000_0011;
    }
}
