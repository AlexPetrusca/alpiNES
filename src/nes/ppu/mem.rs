use crate::util::rom::{Mirroring, ROM};

// PPU memory map
macro_rules! chr_rom_range {() => {0x0000..=0x1FFF}}
macro_rules! vram_range {() => {0x2000..=0x3EFF}}
macro_rules! palletes_range {() => {0x3F00..=0x3FFF}}

pub struct PPUMemory {
    pub memory: [u8; PPUMemory::MEM_SIZE],
    pub screen_mirroring: Mirroring,
    pub is_chr_ram: bool,
}

impl PPUMemory {
    pub const MEM_SIZE: usize = 0x4000; // 16kB
    pub const NAMETABLE_SIZE: usize = 0x03c0;

    pub const CHR_ROM_START: u16 = *chr_rom_range!().start();
    pub const VRAM_START: u16 = *vram_range!().start();
    pub const PALLETES_START: u16 = *palletes_range!().start();

    pub const BACKGROUND_PALLETES_START: u16 = *palletes_range!().start() + 0x01;
    pub const SPRITE_PALLETES_START: u16 = *palletes_range!().start() + 0x11;

    pub fn new() -> Self {
        PPUMemory {
            memory: [0; PPUMemory::MEM_SIZE],
            screen_mirroring: Mirroring::Horizontal,
            is_chr_ram: false,
        }
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.screen_mirroring = rom.screen_mirroring.clone();
        self.is_chr_ram = rom.is_chr_ram;
        for i in 0..rom.chr_rom.len() {
            let idx = PPUMemory::CHR_ROM_START.wrapping_add(i as u16);
            self.memory[idx as usize] = rom.chr_rom[i];
        }
    }

    #[inline]
    pub fn read_byte(&self, address: u16) -> u8 {
        let ppu_addr = address % PPUMemory::MEM_SIZE as u16;
        match ppu_addr {
            chr_rom_range!() => {
                self.memory[ppu_addr as usize]
            },
            vram_range!() => {
                let mirror_addr = self.mirror_vram_addr(ppu_addr);
                self.memory[mirror_addr as usize]
            },
            palletes_range!() => {
                let mirror_addr = PPUMemory::mirror_palette_addr(ppu_addr);
                self.memory[mirror_addr as usize]
            },
            _ => {
                panic!("Attempt to read from unmapped PPU memory: 0x{:0>4X}", ppu_addr);
            }
        }
    }


    #[inline]
    pub fn write_byte(&mut self, address: u16, data: u8) {
        let ppu_addr = address % PPUMemory::MEM_SIZE as u16;
        match ppu_addr {
            chr_rom_range!() => {
                if self.is_chr_ram {
                    self.memory[address as usize] = data;
                } else {
                    panic!("Attempt to write to Cartridge CHR ROM space: 0x{:0>4X}", ppu_addr)
                }
            },
            vram_range!() => {
                let mirror_addr = self.mirror_vram_addr(ppu_addr);
                self.memory[mirror_addr as usize] = data;
            },
            palletes_range!() => {
                let mirror_addr = PPUMemory::mirror_palette_addr(ppu_addr);
                self.memory[mirror_addr as usize] = data;
            },
            _ => {
                panic!("Attempt to write to unmapped PPU memory: 0x{:0>4X}", ppu_addr);
            }
        }
    }

    #[inline]
    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_addr = addr & 0b0010_1111_1111_1111; // mirror down 0x3000-0x3eff to 0x2000-0x2eff
        let name_table = (mirrored_addr - PPUMemory::VRAM_START) / 0x400; // to the name table index
        match (&self.screen_mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => mirrored_addr - 0x800,
            (Mirroring::Horizontal, 1) | (Mirroring::Horizontal, 2) => mirrored_addr - 0x400,
            (Mirroring::Horizontal, 3) => mirrored_addr - 0x800,
            _ => mirrored_addr,
        }
    }

    #[inline]
    fn mirror_palette_addr(ppu_addr: u16) -> u16 {
        let mirror_addr = ppu_addr & 0b0011_1111_0001_1111;
        match mirror_addr {
            0x3F10 => 0x3F00,
            0x3F14 => 0x3F04,
            0x3F18 => 0x3F08,
            0x3F1C => 0x3F0C,
            _ => mirror_addr
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
        let memory = PPUMemory::new();
    }
}