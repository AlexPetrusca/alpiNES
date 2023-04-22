use crate::util::rom::{Mirroring, ROM};

// PPU memory map
macro_rules! chr_rom_range {() => {0x0000..=0x1FFF}}
macro_rules! vram_range {() => {0x2000..=0x3EFF}}
macro_rules! palletes_range {() => {0x3F00..=0x3FFF}}

pub struct Memory {
    pub memory: [u8; Memory::MEM_SIZE],
    pub screen_mirroring: Mirroring,
}

impl Memory {
    const MEM_SIZE: usize = 0x4000 as usize; // 16kB

    pub const CHR_ROM_START: u16 = *chr_rom_range!().start();

    pub fn new() -> Self {
        Memory {
            memory: [0; Memory::MEM_SIZE],
            screen_mirroring: Mirroring::FourScreen,
        }
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.screen_mirroring = rom.screen_mirroring.clone();
        for i in 0..rom.chr_rom.len() {
            let idx = Memory::CHR_ROM_START.wrapping_add(i as u16);
            self.memory[idx as usize] = rom.chr_rom[i];
        }
    }

    #[inline]
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_addr = addr & 0b0010_1111_1111_1111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let name_table = (mirrored_addr - 0x2000) / 0x400; // to the name table index
        match (&self.screen_mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => mirrored_addr - 0x800,
            (Mirroring::Horizontal, 1) | (Mirroring::Horizontal, 2) => mirrored_addr - 0x400,
            (Mirroring::Horizontal, 3) => mirrored_addr - 0x800,
            _ => mirrored_addr,
        }
    }

    #[inline]
    pub fn read_byte(&self, address: u16) -> u8 {
        let ppu_addr = address % Memory::MEM_SIZE as u16;
        match ppu_addr {
            chr_rom_range!() => {
                self.memory[ppu_addr as usize]
            },
            vram_range!() => {
                let mirror_addr = self.mirror_vram_addr(ppu_addr);
                self.memory[mirror_addr as usize]
            },
            palletes_range!() => {
                self.memory[ppu_addr as usize]
            },
            _ => {
                panic!("Attempt to read from unmapped PPU memory: 0x{:0>4X}", ppu_addr);
            }
        }
    }

    #[inline]
    pub fn write_byte(&mut self, address: u16, data: u8) {
        let ppu_addr = address % Memory::MEM_SIZE as u16;
        match ppu_addr {
            chr_rom_range!() => {
                panic!("Attempt to write to Cartridge CHR ROM space: 0x{:0>4X}", ppu_addr)
            },
            vram_range!() => {
                let mirror_addr = self.mirror_vram_addr(ppu_addr);
                self.memory[mirror_addr as usize] = data;
            },
            palletes_range!() => {
                self.memory[ppu_addr as usize] = data;
            },
            _ => {
                panic!("Attempt to write to unmapped PPU memory: 0x{:0>4X}", ppu_addr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTE_A: u8 = 0x0a;
    const BYTE_B: u8 = 0x0b;

    // todo: add more tests for memory

    #[test]
    fn test_read_write() {
        let memory = Memory::new();
    }
}