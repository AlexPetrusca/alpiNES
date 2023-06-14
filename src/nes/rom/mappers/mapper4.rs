use crate::nes::rom::mappers::mapper::Mapper;

#[derive(Clone)]
pub struct Mapper4 {

}

impl Mapper4 {
    pub fn new() -> Self {
        Mapper4 {

        }
    }
}

impl Mapper for Mapper4 {
    fn read_prg_byte(&mut self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        todo!()
    }

    fn read_chr_byte(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        todo!()
    }

    fn write_mapper(&mut self, address: u16, data: u8) {
        todo!()
    }
}
