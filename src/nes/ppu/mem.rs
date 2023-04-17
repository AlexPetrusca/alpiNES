// PPU memory map
macro_rules! chr_rom_range {() => {0x0000..=0x1FFF}}
macro_rules! vram_range {() => {0x2000..=0x3EFF}}
macro_rules! palletes_range {() => {0x3F00..=0x3FFF}}

// macro_rules! pattern_tables_range {() => {0x0000..=0x1FFF}}
// macro_rules! pattern_table_0_range {() => {0x0000..=0x0FFF}}
// macro_rules! pattern_table_1_range {() => {0x1000..=0x1FFF}}

// macro_rules! name_tables_range {() => {0x2000..=0x3EFF}}
// macro_rules! name_table_0_range {() => {0x2000..=0x23FF}}
// macro_rules! name_table_1_range {() => {0x2400..=0x27FF}}
// macro_rules! name_table_2_range {() => {0x2800..=0x2BFF}}
// macro_rules! name_table_3_range {() => {0x2C00..=0x2FFF}}

// macro_rules! palletes_range {() => {0x3F00..=0x3FFF}}

pub struct Memory {
    memory: [u8; Memory::MEM_SIZE],
}

impl Memory {
    const MEM_SIZE: usize = 0x4000 as usize; // 16kB

    pub fn new() -> Self {
        Memory {
            memory: [0; Memory::MEM_SIZE],
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
                let mirror_addr = ppu_addr & 0b0010_1111_1111_1111;
                self.memory[mirror_addr as usize]
            },
            palletes_range!() => {
                self.memory[ppu_addr as usize]
            },
            _ => {
                panic!("Attempt to read from unmapped ppu memory: 0x{:0>4X}", ppu_addr);
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
                let mirror_addr = ppu_addr & 0b0010_1111_1111_1111;
                self.memory[mirror_addr as usize] = data;
            },
            palletes_range!() => {
                self.memory[ppu_addr as usize] = data;
            },
            _ => {
                panic!("Attempt to write to unmapped ppu memory: 0x{:0>4X}", ppu_addr);
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