pub trait Mapper {
    #[inline]
    fn read_prg_byte(&mut self, address: u16, prg_rom: &Vec<u8>) -> u8;

    #[inline]
    fn read_chr_byte(&self, address: u16, chr_rom: &Vec<u8>) -> u8;

    #[inline]
    fn write_mapper(&mut self, address: u16, data: u8);
}