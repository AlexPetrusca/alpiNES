pub mod mem;
mod oam;
mod registers;

use crate::nes::ppu::mem::Memory;
use crate::nes::ppu::oam::OAM;
use crate::nes::ppu::registers::addr::AddressRegister;
use crate::nes::ppu::registers::ctrl::ControlRegister;

pub struct PPU {
    pub addr: AddressRegister,
    pub data: u8,
    pub ctrl: ControlRegister,
    pub memory: Memory,
    oam: OAM,
    buffer: u8,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            addr: AddressRegister::new(),
            data: 0,
            ctrl: ControlRegister::new(),
            memory: Memory::new(),
            oam: OAM::new(),
            buffer: 0,
        }
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        Ok(true)
    }

    fn read_byte(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        let result = self.buffer;
        self.buffer = self.memory.read_byte(addr);
        result
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.get_vram_addr_increment());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_() {
        let mut ppu = PPU::new();
    }
}