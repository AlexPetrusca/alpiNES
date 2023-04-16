use crate::io::rom::ROM;

// CPU memory map
macro_rules! ram_range {() => {0x0000..=0x1FFF}}
macro_rules! ppu_registers_range {() => {0x2000..=0x3FFF}}
macro_rules! apu_registers_range {() => {0x4000..=0x4017}}
macro_rules! prg_rom_range {() => {0x8000..=0xFFFF}}

pub struct Memory {
    memory: [u8; Memory::MEM_SIZE],
    prg_mirror_enabled: bool
}

impl Memory {
    const MEM_SIZE: usize = 0x10000 as usize; // 64kB

    pub const PRG_ROM_START: u16 = *prg_rom_range!().start();
    pub const IRQ_INT_VECTOR: u16 = 0xFFFE;
    pub const RESET_INT_VECTOR: u16 = 0xFFFC;
    pub const NMI_INT_VECTOR: u16 = 0xFFFA;

    pub fn new() -> Self {
        Memory {
            memory: [0; Memory::MEM_SIZE],
            prg_mirror_enabled: false
        }
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.prg_mirror_enabled = rom.prg_rom_mirroring;
        for i in 0..rom.prg_rom.len() {
            let idx = Memory::PRG_ROM_START.wrapping_add(i as u16);
            self.memory[idx as usize] = rom.prg_rom[i];
        }
    }

    pub fn load_at_addr(&mut self, address: u16, program: &Vec<u8>) {
        for i in 0..program.len() {
            self.memory[address.wrapping_add(i as u16) as usize] = program[i];
        }
        let addr_bytes = &u16::to_le_bytes(address);
        self.memory[Memory::RESET_INT_VECTOR as usize] = addr_bytes[0];
        self.memory[Memory::RESET_INT_VECTOR.wrapping_add(1) as usize] = addr_bytes[1];
    }

    #[inline]
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            ram_range!() => {
                let mirror_addr = address & 0b00000111_11111111;
                self.memory[mirror_addr as usize]
            }
            ppu_registers_range!() => {
                let mirror_addr = address & 0b00100000_00000111;
                self.memory[mirror_addr as usize]
            }
            prg_rom_range!() => {
                let mut offset = address - Memory::PRG_ROM_START;
                if self.prg_mirror_enabled && address >= 0x4000 {
                    offset = offset % 0x4000;
                }
                self.memory[(Memory::PRG_ROM_START + offset) as usize]
            },
            _ => {
                // self.memory[address as usize]
                panic!("Attempt to read from unmapped memory: 0x{:0>4X}", address);
            }
        }
    }

    #[inline]
    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            ram_range!() => {
                let mirror_addr = address & 0b00000111_11111111;
                self.memory[mirror_addr as usize] = data;
            }
            ppu_registers_range!() => {
                let mirror_addr = address & 0b00100000_00000111;
                self.memory[mirror_addr as usize] = data;
            }
            prg_rom_range!() => {
                panic!("Attempt to write to Cartridge ROM space: 0x{:0>4X}", address)
            },
            _ => {
                // self.memory[address as usize] = data;
                panic!("Attempt to write to unmapped memory: 0x{:0>4X}", address);
            }
        }
    }

    #[inline]
    pub fn write_bulk(&mut self, address: u16, data: &[u8]) {
        for i in 0..data.len() {
            self.write_byte(address.wrapping_add(i as u16), data[i]);
        }
    }

    #[inline]
    pub fn read_addr(&self, address: u16) -> u16 {
        u16::from_le_bytes([
            self.read_byte(address),
            self.read_byte(address.wrapping_add(1))
        ])
    }

    #[inline]
    pub fn read_addr_zp(&self, address: u8) -> u16 {
        u16::from_le_bytes([
            self.read_byte(address as u16),
            self.read_byte(address.wrapping_add(1) as u16)
        ])
    }

    #[inline]
    pub fn read_addr_in(&self, address: u16) -> u16 {
        let upper_addr = address & 0xff00;
        let lower_addr = (address & 0x00ff) as u8;
        u16::from_le_bytes([
            self.read_byte(address),
            self.read_byte(upper_addr + lower_addr.wrapping_add(1) as u16)
        ])
    }

    #[inline]
    pub fn write_addr(&mut self, address: u16, waddr: u16) {
        let bytes = u16::to_le_bytes(waddr);
        self.write_byte(address, bytes[0]);
        self.write_byte(address.wrapping_add(1), bytes[1]);
    }

    #[inline]
    pub fn zp_read(&self, address: u8) -> u8 {
        self.read_byte(address as u16)
    }

    #[inline]
    pub fn zp_x_read(&self, address: u8, register_x: u8) -> u8 {
        self.read_byte(address.wrapping_add(register_x) as u16)
    }

    #[inline]
    pub fn zp_y_read(&self, address: u8, register_y: u8) -> u8 {
        self.read_byte(address.wrapping_add(register_y) as u16)
    }

    #[inline]
    pub fn ab_read(&self, address: u16) -> u8 {
        self.read_byte(address)
    }

    #[inline]
    pub fn ab_x_read(&self, address: u16, register_x: u8) -> u8 {
        self.read_byte(address.wrapping_add(register_x as u16))
    }

    #[inline]
    pub fn ab_y_read(&self, address: u16, register_y: u8) -> u8 {
        self.read_byte(address.wrapping_add(register_y as u16))
    }

    #[inline]
    pub fn in_x_read(&self, address: u8, register_x: u8) -> u8 {
        let pointer = self.read_addr_zp(address.wrapping_add(register_x));
        self.read_byte(pointer)
    }

    #[inline]
    pub fn in_y_read(&self, address: u8, register_y: u8) -> u8 {
        let pointer = self.read_addr_zp(address);
        self.read_byte(pointer.wrapping_add(register_y as u16))
    }

    #[inline]
    pub fn zp_write(&mut self, address: u8, value: u8) {
        self.write_byte(address as u16, value);
    }

    #[inline]
    pub fn zp_x_write(&mut self, address: u8, register_x: u8, value: u8) {
        self.write_byte(address.wrapping_add(register_x) as u16, value);
    }

    #[inline]
    pub fn zp_y_write(&mut self, address: u8, register_y: u8, value: u8) {
        self.write_byte(address.wrapping_add(register_y) as u16, value);
    }

    #[inline]
    pub fn ab_write(&mut self, address: u16, value: u8) {
        self.write_byte(address, value);
    }

    #[inline]
    pub fn ab_x_write(&mut self, address: u16, register_x: u8, value: u8) {
        self.write_byte(address.wrapping_add(register_x as u16), value);
    }

    #[inline]
    pub fn ab_y_write(&mut self, address: u16, register_y: u8, value: u8) {
        self.write_byte(address.wrapping_add(register_y as u16), value)
    }

    #[inline]
    pub fn in_x_write(&mut self, address: u8, register_x: u8, value: u8) {
        let pointer = self.read_addr_zp(address.wrapping_add(register_x));
        self.write_byte(pointer, value);
    }

    #[inline]
    pub fn in_y_write(&mut self, address: u8, register_y: u8, value: u8) {
        let pointer = self.read_addr_zp(address);
        self.write_byte(pointer.wrapping_add(register_y as u16), value);
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
        let mut mem = Memory::new();
        mem.write_byte(0x0001, BYTE_A);
        mem.write_byte(0x0002, BYTE_B);
        assert_eq!(mem.read_byte(0x0001), BYTE_A);
        assert_eq!(mem.read_byte(0x0002), BYTE_B);
    }

    #[test]
    fn test_write_bulk() {
        let mut mem = Memory::new();
        mem.write_bulk(0x0001, &[BYTE_A, BYTE_B]);
        assert_eq!(mem.read_byte(0x0001), BYTE_A);
        assert_eq!(mem.read_byte(0x0002), BYTE_B);
    }

    #[test]
    fn test_read_write_addr() {
        let mut mem = Memory::new();
        mem.write_addr(0x0100, 0x0a0b);
        assert_eq!(mem.read_byte(0x0100), 0x0b);
        assert_eq!(mem.read_addr(0x0101), 0x0a);
        assert_eq!(mem.read_addr(0x0100), 0x0a0b);
    }
}