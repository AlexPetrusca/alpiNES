pub mod mem;
mod oam;
mod registers;

use crate::io::rom::Mirroring;
use crate::nes::ppu::mem::Memory;
use crate::nes::ppu::oam::OAM;
use crate::nes::ppu::registers::addr::AddressRegister;
use crate::nes::ppu::registers::ctrl::ControlRegister;

pub struct PPU {
    pub addr: AddressRegister,
    pub data: u8,
    pub ctrl: ControlRegister,
    pub memory: Memory,
    pub buffer: u8, // todo: should be private
    pub oam: OAM, // todo: should be private
    pub scanline: u16,
    pub cycles: usize,
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
            scanline: 0,
            cycles: 0
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        // if self.cycles >= 341 {
        //     self.cycles = self.cycles - 341;
        //     self.scanline += 1;
        //
        //     if self.scanline == 241 {
        //         if self.ctrl.generate_vblank_nmi() {
        //             self.status.set_vblank_status(true);
        //             todo!("Should trigger NMI interrupt")
        //         }
        //     }
        //
        //     if self.scanline >= 262 {
        //         self.scanline = 0;
        //         self.status.reset_vblank_status();
        //         OK(true);
        //     }
        // }
        Ok(false)
    }

    pub fn write_addr_register(&mut self, value: u8) {
        self.addr.write(value);
    }

    pub fn read_data_register(&mut self) -> u8 {
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