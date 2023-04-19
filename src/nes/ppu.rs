pub mod mem;
mod oam;
mod registers;

use crate::io::bitvec::BitVector;
use crate::io::rom::Mirroring;
use crate::nes::ppu::mem::Memory;
use crate::nes::ppu::oam::OAM;
use crate::nes::ppu::registers::addr::AddressRegister;
use crate::nes::ppu::registers::ctrl::ControlFlag::GenerateNmi;
use crate::nes::ppu::registers::ctrl::ControlRegister;
use crate::nes::ppu::registers::mask::MaskRegister;
use crate::nes::ppu::registers::stat::StatusFlag::VerticalBlank;
use crate::nes::ppu::registers::stat::StatusRegister;

pub struct PPU {
    pub addr: AddressRegister,
    pub data: u8,
    pub ctrl: ControlRegister,
    pub stat: StatusRegister,
    pub mask: MaskRegister,
    pub memory: Memory,
    pub oam: OAM, // todo: should be private
    pub data_buffer: u8, // todo: should be private
    pub scanline: u16,
    pub cycles: usize,
    pub nmi_flag: bool, // todo: should be private
}

impl PPU {
    pub fn new() -> Self {
        Self {
            addr: AddressRegister::new(),
            data: 0,
            ctrl: ControlRegister::new(),
            stat: StatusRegister::new(),
            mask: MaskRegister::new(),
            memory: Memory::new(),
            oam: OAM::new(),
            data_buffer: 0,
            scanline: 0,
            cycles: 0,
            nmi_flag: false,
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        if self.cycles >= 341 {
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                if self.ctrl.is_set(GenerateNmi) {
                    // NMI is triggered when PPU enters VBLANK state
                    self.stat.set(VerticalBlank);
                    self.set_nmi();
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.stat.clear(VerticalBlank);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn write_addr_register(&mut self, value: u8) {
        self.addr.write(value);
    }

    pub fn read_data_register(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        let result = self.data_buffer;
        self.data_buffer = self.memory.read_byte(addr);
        result
    }

    pub fn write_ctrl_register(&mut self, value: u8) {
        // NMI is triggered if:
        //  1. PPU is in VBLANK state
        //  2. "Generate NMI" bit in the control Register is updated from 0 to 1.
        let before_nmi_status = self.ctrl.is_set(GenerateNmi);
        self.ctrl.set_value(value);
        if !before_nmi_status && self.ctrl.is_set(GenerateNmi) && self.stat.is_set(VerticalBlank) {
            self.set_nmi();
        }
    }


    pub fn poll_nmi(&self) -> bool {
        return self.nmi_flag;
    }

    pub fn set_nmi(&mut self) {
        self.nmi_flag = true;
    }

    pub fn clear_nmi(&mut self) {
        self.nmi_flag = false;
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