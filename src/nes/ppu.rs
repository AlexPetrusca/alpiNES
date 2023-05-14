pub mod mem;
pub mod oam;
pub mod registers;

use crate::nes::io::frame::Frame;
use crate::nes::io::viewport::Viewport;
use crate::nes::NES;
use crate::util::bitvec::BitVector;
use crate::util::rom::Mirroring;
use crate::nes::ppu::mem::PPUMemory;
use crate::nes::ppu::oam::OAM;
use crate::nes::ppu::registers::addr::AddressRegister;
use crate::nes::ppu::registers::scroll::ScrollRegister;
use crate::nes::ppu::registers::ctrl::ControlRegister;
use crate::nes::ppu::registers::ctrl::ControlFlag::{GenerateNmi, SpriteSize};
use crate::nes::ppu::registers::mask::{MaskFlag, MaskRegister};
use crate::nes::ppu::registers::mask::MaskFlag::{ShowBackground, ShowSprites};
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

    // todo:
    //   Sprite data is delayed by one scanline; you must subtract 1 from the sprite's Y
    //   coordinate before writing it here. Hide a sprite by moving it down offscreen, by
    //   writing any values between #$EF-#$FF here. Sprites are never displayed on the first
    //   line of the picture, and it is impossible to place a sprite partially off the top of
    //   the screen.

    pub memory: PPUMemory,
    pub frame: Frame,
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
            frame: Frame::new(),
            oam: OAM::new(),
            data_buffer: 0,

            scanline: 0,
            cycles: 0,
            nmi_flag: false,
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += 3 * cycles as usize;
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        // let sprite_x = self.oam.memory[3] as usize;
        // let sprite_y = self.oam.memory[0] as usize;
        // let mut scanline = if self.scanline == 0 { 0 } else { self.scanline - 1 };
        // if self.cycles > sprite_x && scanline as usize >= sprite_y {
        //     let mut sprite_zero_hit = false;
        //     let sprites_bank = self.ctrl.get_sprite_chrtable_address();
        //     let sprite_tile = self.oam.memory[1] as u16;
        //     let mut tile_addr = sprites_bank + 16 * sprite_tile + scanline - sprite_y as u16;
        //     let mut lower = self.memory.read_byte(tile_addr);
        //     let mut upper = self.memory.read_byte(tile_addr + 8);
        //     for _ in (0..8).rev() {
        //         let value = (1 & upper) << 1 | (1 & lower);
        //         if value != 0 {
        //             sprite_zero_hit = true;
        //         }
        //         lower = lower >> 1;
        //         upper = upper >> 1;
        //     }
        //     self.status.update(SpriteZeroHit, sprite_zero_hit);
        // }

        if self.cycles > 340 {
            self.render_scanline();

            if self.scanline < 240 {
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

            if self.scanline == 261 {
                self.scanline = 0;
                self.clear_nmi();
                self.status.clear(VerticalBlank);
                self.status.clear(SpriteZeroHit);
                return Ok(true);
            }

            self.cycles = self.cycles - 341;
            self.scanline += 1;
        }

        Ok(false)
    }


    #[inline]
    pub fn render_scanline(&mut self) {
        if self.scanline == 260 { self.frame.clear(); }
        if self.scanline >= 240 { return }

        // if (self.scanline >= 31 && self.scanline <= 50) || (self.scanline > 100 && self.scanline <= 140) {
        self.render_background_scanline();
        self.render_sprites_scanline();
        // }
    }

    #[inline]
    pub fn render_background_scanline(&mut self) {
        if self.mask.is_clear(ShowBackground) { return }

        let background_bank = self.ctrl.get_background_chrtable_address();
        let (nametable1, nametable2) = self.get_nametables();

        let scroll_x = self.scroll.get_scroll_x() as isize;
        let mut scroll_y = self.scroll.get_scroll_y() as isize;
        if scroll_y >= 240 { scroll_y = scroll_y - 256; }

        let mut nametable = nametable1;
        let mut palette = [0, 0, 0, 0];
        let mut tile_upper_chr = 0;
        let mut tile_lower_chr = 0;

        let screen_y = self.scanline as usize;
        let mut pixel_y = screen_y as isize + scroll_y;
        if pixel_y >= 240 {
            pixel_y -= 240;
            nametable = nametable2
        } else if pixel_y < 0 {
            pixel_y += 240;
        }

        let mut tile_y = pixel_y as usize / 8;
        let mut tile_x = 255;
        for screen_x in 0..Frame::WIDTH {
            let mut pixel_x = screen_x as isize + scroll_x;
            if pixel_x >= 256 {
                pixel_x -= 256;
                nametable = nametable2;
            }

            let tile_x_new = pixel_x as usize / 8;
            if tile_x != tile_x_new {
                tile_x = tile_x_new;
                palette = self.bg_palette(nametable, tile_x, tile_y);
                let tile_idx = nametable + 32 * tile_y as u16 + tile_x as u16;
                let tile_value = self.memory.read_byte(tile_idx) as u16;
                let tile_address = background_bank + 16 * tile_value;

                let chr_y = (pixel_y % 8) as u16;
                tile_lower_chr = self.memory.read_byte(tile_address + chr_y);
                tile_upper_chr = self.memory.read_byte(tile_address + chr_y + 8);
            }

            let chr_x = 7 - (pixel_x % 8) as u16;
            let lower = tile_lower_chr >> chr_x;
            let upper = tile_upper_chr >> chr_x;
            let palette_value = (1 & upper) << 1 | (1 & lower);
            let palette_idx = palette[palette_value as usize];
            let rgb = NES::SYSTEM_PALLETE[palette_idx as usize];
            let priority = if palette_value == 0 { Frame::BG_PRIORITY } else { Frame::FG_PRIORITY };
            self.frame.set_background_pixel(screen_x, screen_y, rgb, priority);
        }
    }

    #[inline]
    pub fn render_sprites_scanline(&mut self) {
        if self.mask.is_clear(ShowSprites) { return }

        let sprites_bank = self.ctrl.get_sprite_chrtable_address();
        let sprite_size = if self.ctrl.is_set(SpriteSize) { 16 } else { 8 };

        let screen_y = if self.scanline == 0 { 0 } else { self.scanline - 1 } as usize;
        for sprite_idx in (0..self.oam.memory.len()).step_by(4).rev() {
            let sprite_x = self.oam.memory[sprite_idx + 3] as usize;
            let sprite_y = self.oam.memory[sprite_idx] as usize;

            if screen_y < sprite_y || screen_y >= sprite_y + sprite_size { continue }

            let priority = (self.oam.memory[sprite_idx + 2] >> 5 & 1 == 0) as u8;
            let tile_value = self.oam.memory[sprite_idx + 1] as u16;

            let flip_vertical = self.oam.memory[sprite_idx + 2] >> 7 & 1 == 1;
            let flip_horizontal = self.oam.memory[sprite_idx + 2] >> 6 & 1 == 1;
            let palette_idx = self.oam.memory[sprite_idx + 2] & 0b0000_0011;
            let sprite_palette = self.sprite_palette(palette_idx);

            let y = screen_y - sprite_y;
            let mut chr_y = if flip_vertical { sprite_size - 1 - y } else { y } as u16;
            let mut tile_addr = sprites_bank + 16 * tile_value;
            if sprite_size == 16 {
                let sprites_bank = if tile_value & 1 == 1 { 0x1000 } else { 0x0000 };
                let tile_value = if y >= 8 { tile_value + 1 } else { tile_value };
                tile_addr = sprites_bank + 16 * tile_value;
                chr_y = chr_y % 8;
            }

            let mut lower_chr = self.memory.read_byte(tile_addr + chr_y);
            let mut upper_chr = self.memory.read_byte(tile_addr + chr_y + 8);
            for x in 0..8 {
                let screen_x = sprite_x + x;
                let chr_x = if flip_horizontal { x } else { 7 - x };
                let lower = lower_chr >> chr_x;
                let upper = upper_chr >> chr_x;
                let value = (1 & upper) << 1 | (1 & lower);
                if value != 0 {
                    let rgb = NES::SYSTEM_PALLETE[sprite_palette[value as usize] as usize];
                    self.frame.set_sprite_pixel(screen_x, screen_y + 1, rgb, priority); // todo: "screen_y + 1" might be wrong here
                    if sprite_idx == 0 {
                        self.status.set(SpriteZeroHit);
                    }
                }
            }
        }
    }

    #[inline]
    fn get_nametables(&mut self) -> (u16, u16) {
        match (&self.memory.rom.screen_mirroring, self.ctrl.get_base_nametable_address()) {
            (Mirroring::Vertical, 0x2000) | (Mirroring::Vertical, 0x2800) => {
                (0x2000, 0x2400)
            },
            (Mirroring::Vertical, 0x2400) | (Mirroring::Vertical, 0x2C00) => {
                (0x2400, 0x2000)
            },
            (Mirroring::Horizontal, 0x2000) | (Mirroring::Horizontal, 0x2400) => {
                (0x2000, 0x2800)
            },
            (Mirroring::Horizontal, 0x2800) | (Mirroring::Horizontal, 0x2C00) => {
                (0x2800, 0x2000)
            },
            (_, _) => {
                panic!("Not supported mirroring type {:?}", self.memory.rom.screen_mirroring);
            }
        }
    }

    #[inline]
    fn bg_palette(&self, nametable_addr: u16, tile_x: usize, tile_y: usize) -> [u8; 4] {
        let attr_table_idx = 8 * (tile_y / 4) + tile_x / 4;
        let attr_byte = self.memory.read_byte(nametable_addr + PPUMemory::NAMETABLE_SIZE as u16 + attr_table_idx as u16);
        let pallete = match ((tile_x % 4) / 2, (tile_y % 4) / 2) {
            (0, 0) => attr_byte & 0b0000_0011,
            (1, 0) => (attr_byte >> 2) & 0b0000_0011,
            (0, 1) => (attr_byte >> 4) & 0b0000_0011,
            (1, 1) => (attr_byte >> 6) & 0b0000_0011,
            (_, _) => panic!("can't be"),
        };
        let pallete_idx = 4 * pallete as u16;
        [
            self.memory.read_byte(PPUMemory::PALLETES_START),
            self.memory.read_byte(PPUMemory::BACKGROUND_PALLETES_START + pallete_idx),
            self.memory.read_byte(PPUMemory::BACKGROUND_PALLETES_START + pallete_idx + 1),
            self.memory.read_byte(PPUMemory::BACKGROUND_PALLETES_START + pallete_idx + 2),
        ]
    }

    #[inline]
    fn sprite_palette(&self, pallete: u8) -> [u8; 4] {
        let pallete_idx = 4 * pallete as u16;
        [
            0,
            self.memory.read_byte(PPUMemory::SPRITE_PALLETES_START + pallete_idx),
            self.memory.read_byte(PPUMemory::SPRITE_PALLETES_START + pallete_idx + 1),
            self.memory.read_byte(PPUMemory::SPRITE_PALLETES_START + pallete_idx + 2),
        ]
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
        self.oam_addr += 1; // todo: handle oam_addr overflow

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