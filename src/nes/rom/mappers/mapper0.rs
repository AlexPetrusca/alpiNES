use crate::nes::rom::mappers::mapper::Mapper;

#[derive(Clone)]
pub struct Mapper0 { }

impl Mapper0 {
    pub fn new() -> Self {
        Mapper0 { }
    }
}

impl Mapper for Mapper0 {
    fn read_prg_byte(&mut self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        prg_rom[(address - 0x8000) as usize]
    }

    fn read_chr_byte(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        chr_rom[address as usize]
    }

    fn write_mapper(&mut self, address: u16, _data: u8) {
        panic!("Attempt to write to Cartridge PRG ROM space: 0x{:0>4X}", address);
    }
}
