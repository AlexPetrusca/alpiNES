pub mod mem;
mod oam;
mod registers;

use crate::nes::ppu::mem::Memory;
use crate::nes::ppu::oam::OAM;

pub struct PPU {
    memory: Memory,
    oam: OAM,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            oam: OAM::new(),
        }
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