use crate::rom::ROM;

const MEM_LEN: usize = 0x10000 as usize;

pub struct Memory {
    memory: [u8; MEM_LEN]
}

impl Memory {
    pub const RAM_START: u16 = 0x0000;
    pub const RAM_END: u16 = 0x1FFF;
    pub const PPU_REGISTERS_START: u16 = 0x2000;
    pub const PPU_REGISTERS_END: u16 = 0x3FFF;
    pub const PRG_ROM_START: u16 = 0x8000;
    pub const PRG_ROM_END: u16 = 0xFFFF;

    pub const IRQ_INT_VECTOR: u16 = 0xFFFE;
    pub const RESET_INT_VECTOR: u16 = 0xFFFC;
    pub const NMI_INT_VECTOR: u16 = 0xFFFA;

    pub fn new() -> Self {
        Memory {
            memory: [0; MEM_LEN]
        }
    }

    pub fn load_at_addr(&mut self, addr: u16, program: &Vec<u8>) {
        for i in 0..program.len() {
            self.memory[addr.wrapping_add(i as u16) as usize] = program[i];
        }
        let addr_bytes = &u16::to_le_bytes(addr);
        self.memory[Memory::RESET_INT_VECTOR as usize] = addr_bytes[0];
        self.memory[Memory::RESET_INT_VECTOR.wrapping_add(1) as usize] = addr_bytes[1];
    }

    #[inline]
    pub fn load_rom(&mut self, rom: &ROM) {
        for i in 0..rom.prg_rom.len() {
            let idx = Memory::PRG_ROM_START.wrapping_add(i as u16);
            self.memory[idx as usize] = rom.prg_rom[i];
        }
    }

    #[inline]
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            Memory::RAM_START..=Memory::RAM_END => {
                let mirror_addr = addr & 0b00000111_11111111;
                self.memory[mirror_addr as usize]
            }
            Memory::PPU_REGISTERS_START..=Memory::PPU_REGISTERS_END => {
                let mirror_addr = addr & 0b00100000_00000111;
                self.memory[mirror_addr as usize]
            }
            Memory::PRG_ROM_START..=Memory::PRG_ROM_END => {
                self.memory[addr as usize]
            },
            _ => {
                // self.memory[addr as usize]
                panic!("Attempt to read from unmapped memory: {}", addr);
            }
        }
    }

    #[inline]
    pub fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            Memory::RAM_START..=Memory::RAM_END => {
                let mirror_addr = addr & 0b00000111_11111111;
                self.memory[mirror_addr as usize] = data;
            }
            Memory::PPU_REGISTERS_START..=Memory::PPU_REGISTERS_END => {
                let mirror_addr = addr & 0b00100000_00000111;
                self.memory[mirror_addr as usize] = data;
            }
            Memory::PRG_ROM_START..=Memory::PRG_ROM_END => {
                panic!("Attempt to write to Cartridge ROM space: {}", addr)
            },
            _ => {
                // self.memory[addr as usize] = data;
                panic!("Attempt to write to unmapped memory: {}", addr);
            }
        }
    }

    #[inline]
    pub fn write_bulk(&mut self, addr: u16, data: &[u8]) {
        for i in 0..data.len() {
            self.write_byte(addr.wrapping_add(i as u16), data[i]);
        }
    }

    #[inline]
    pub fn read_addr(&self, addr: u16) -> u16 {
        u16::from_le_bytes([
            self.read_byte(addr),
            self.read_byte(addr.wrapping_add(1))
        ])
    }

    #[inline]
    pub fn write_addr(&mut self, addr: u16, waddr: u16) {
        self.write_bulk(addr, &u16::to_le_bytes(waddr));
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
        let pointer = self.read_addr(address.wrapping_add(register_x) as u16);
        self.read_byte(pointer)
    }

    #[inline]
    pub fn in_y_read(&self, address: u8, register_y: u8) -> u8 {
        let pointer = self.read_addr(address as u16);
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
        let pointer = self.read_addr(address.wrapping_add(register_x) as u16);
        self.write_byte(pointer, value);
    }

    #[inline]
    pub fn in_y_write(&mut self, address: u8, register_y: u8, value: u8) {
        let pointer = self.read_addr(address as u16);
        self.write_byte(pointer.wrapping_add(register_y as u16), value);
    }

    #[inline]
    pub fn len(&self) -> usize {
        MEM_LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTE_A: u8 = 0x0a;
    const BYTE_B: u8 = 0x0b;

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