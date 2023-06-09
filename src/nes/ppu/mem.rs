use crate::nes::rom::{Mirroring, ROM};

// PPU memory map
#[macro_export] macro_rules! chr_rom_range { () => {0x0000..=0x1FFF} }
#[macro_export] macro_rules! vram_range { () => {0x2000..=0x3EFF} }
#[macro_export] macro_rules! palletes_ram_range { () => {0x3F00..=0x3FFF} }

pub struct PPUMemory {
    pub memory: [u8; PPUMemory::MEM_SIZE],
    pub rom: ROM,
}

impl PPUMemory {
    pub const MEM_SIZE: usize = 0x4000; // 16kB
    pub const NAMETABLE_SIZE: usize = 0x03c0;

    pub const CHR_ROM_START: u16 = *chr_rom_range!().start();
    pub const VRAM_START: u16 = *vram_range!().start();
    pub const PALLETES_START: u16 = *palletes_ram_range!().start();

    pub const BACKGROUND_PALLETES_START: u16 = *palletes_ram_range!().start() + 0x01;
    pub const SPRITE_PALLETES_START: u16 = *palletes_ram_range!().start() + 0x11;

    pub fn new() -> Self {
        PPUMemory {
            memory: [0; PPUMemory::MEM_SIZE],
            rom: ROM::new(),
        }
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.rom = rom.clone();
    }

    #[inline]
    pub fn read_byte(&self, address: u16) -> u8 {
        let ppu_addr = address % PPUMemory::MEM_SIZE as u16;
        match ppu_addr {
            chr_rom_range!() => {
                self.rom.read_chr_byte(ppu_addr)
            },
            vram_range!() => {
                let mirror_addr = self.mirror_vram_addr(ppu_addr);
                self.memory[mirror_addr as usize]
            },
            palletes_ram_range!() => {
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
                self.rom.write_chr_byte(ppu_addr, data)
            },
            vram_range!() => {
                let mirror_addr = self.mirror_vram_addr(ppu_addr);
                self.memory[mirror_addr as usize] = data;
            },
            palletes_ram_range!() => {
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
        // todo: does this need to be changed with the introduction of SingleScreen?
        let mirrored_addr = addr & 0b0010_1111_1111_1111; // mirror down 0x3000-0x3eff to 0x2000-0x2eff
        let name_table = (mirrored_addr - PPUMemory::VRAM_START) / 0x400; // to the name table index
        match (&self.rom.screen_mirroring, name_table) {
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