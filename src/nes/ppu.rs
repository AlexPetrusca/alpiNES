pub mod mem;
mod oam;
mod registers;

use crate::util::bitvec::BitVector;
use crate::util::rom::Mirroring;
use crate::nes::ppu::mem::PPUMemory;
use crate::nes::ppu::oam::OAM;
use crate::nes::ppu::registers::addr::AddressRegister;
use crate::nes::ppu::registers::scroll::ScrollRegister;
use crate::nes::ppu::registers::ctrl::ControlRegister;
use crate::nes::ppu::registers::ctrl::ControlFlag::GenerateNmi;
use crate::nes::ppu::registers::mask::MaskRegister;
use crate::nes::ppu::registers::mask::MaskFlag::ShowSprites;
use crate::nes::ppu::registers::status::StatusRegister;
use crate::nes::ppu::registers::status::StatusFlag::{SpriteZeroHit, VerticalBlank};

pub struct PPU {
    pub addr: AddressRegister,
    pub data: u8, // todo: Use DataRegister instead?
    pub ctrl: ControlRegister,
    pub status: StatusRegister,
    pub mask: MaskRegister,
    pub scroll: ScrollRegister,
    pub oam_addr: u8, // todo: Use OAMAddrRegister instead?
    pub oam_data: u8, // todo: Use OAMDataRegister instead?

    pub memory: PPUMemory,
    pub oam: OAM, // todo: should be private
    pub data_buffer: u8, // todo: should be private

    pub cycles: usize,
    pub scanline: u16,
    pub nmi_flag: bool, // todo: should be private
}

impl PPU {
    pub fn new() -> Self {
        Self {
            addr: AddressRegister::new(),
            data: 0,
            ctrl: ControlRegister::new(),
            status: StatusRegister::new(),
            mask: MaskRegister::new(),
            scroll: ScrollRegister::new(),
            oam_addr: 0,
            oam_data: 0,

            memory: PPUMemory::new(),
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
            // todo: condition x <= cycles is always true in is_sprite_0_hit()
            self.status.update(SpriteZeroHit, self.is_sprite_0_hit(self.cycles));

            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline < 241 {
                self.oam_addr = 0; // todo: is this enough? https://www.nesdev.org/wiki/PPU_registers
            }

            if self.scanline == 241 {
                self.status.set(VerticalBlank);
                self.status.clear(SpriteZeroHit);
                if self.ctrl.is_set(GenerateNmi) {
                    // NMI is triggered when PPU enters VBLANK state
                    self.set_nmi();
                }
            }

            if self.scanline > 261 {
                self.scanline = 0;
                self.clear_nmi();
                self.status.clear(VerticalBlank);
                self.status.clear(SpriteZeroHit);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn is_sprite_0_hit(&self, cycles: usize) -> bool {
        let y = self.oam.read_byte(0) as u16;
        let x = self.oam.read_byte(3) as usize;
        return y == self.scanline && x <= cycles && self.mask.is_set(ShowSprites);
    }

    pub fn write_scroll_register(&mut self, value: u8) {
        self.scroll.write(value);
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

    pub fn write_data_register(&mut self, value: u8) {
        let addr = self.addr.get();
        self.increment_vram_addr();

        self.data = value;
        self.memory.write_byte(addr, value);
    }

    pub fn write_oam_addr_register(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn read_oam_data_register(&mut self) -> u8 {
        let addr = self.oam_addr;
        // todo: is this check necessary? https://www.nesdev.org/wiki/PPU_registers
        // if !self.stat.is_set(VerticalBlank) {
        //     self.oam_addr += 1;
        // }

        self.oam.read_byte(addr)
    }

    pub fn write_oam_data_register(&mut self, value: u8) {
        let addr = self.oam_addr;
        self.oam_addr += 1;

        self.oam.write_byte(addr, value);
    }

    pub fn write_ctrl_register(&mut self, value: u8) {
        // NMI is triggered if:
        //  1. PPU is in VBLANK state
        //  2. "Generate NMI" bit in the control Register is updated from 0 to 1.
        let before_nmi_status = self.ctrl.is_set(GenerateNmi);
        self.ctrl.set_value(value);
        if !before_nmi_status && self.ctrl.is_set(GenerateNmi) && self.status.is_set(VerticalBlank) {
            self.set_nmi();
        }
    }

    pub fn write_mask_register(&mut self, value: u8) {
        self.mask.set_value(value);
    }

    pub fn read_status_register(&mut self) -> u8 {
        let status = self.status.get_value();
        // Reading the status register will clear bit 7 mentioned above
        self.status.clear(VerticalBlank);
        // todo: and also the address latch used by PPUSCROLL and PPUADDR.
        status
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