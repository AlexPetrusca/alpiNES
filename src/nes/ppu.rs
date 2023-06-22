pub mod mem;
pub mod oam;
pub mod registers;

use crate::nes::io::frame::Frame;
use crate::nes::io::viewport::Viewport;
use crate::nes::NES;
use crate::util::bitvec::BitVector;
use crate::nes::ppu::mem::PPUMemory;
use crate::nes::ppu::oam::OAM;
use crate::nes::ppu::registers::addr::AddressRegister;
use crate::nes::ppu::registers::scroll::ScrollRegister;
use crate::nes::ppu::registers::ctrl::ControlRegister;
use crate::nes::ppu::registers::ctrl::ControlFlag::{GenerateNmi, SpriteSize};
use crate::nes::ppu::registers::mask::{MaskFlag, MaskRegister};
use crate::nes::ppu::registers::mask::MaskFlag::{ShowBackground, ShowSprites};
use crate::nes::ppu::registers::scrollctx::ScrollContext;
use crate::nes::ppu::registers::status::StatusRegister;
use crate::nes::ppu::registers::status::StatusFlag::{SpriteZeroHit, VerticalBlank};
use crate::nes::rom::Mirroring;

pub struct PPU {
    pub addr: AddressRegister,
    pub data: u8,
    pub ctrl: ControlRegister,
    pub status: StatusRegister,
    pub mask: MaskRegister,
    pub scroll: ScrollRegister,
    pub oam_addr: u8,
    pub oam_data: u8,

    // todo:
    //   Sprite data is delayed by one scanline; you must subtract 1 from the sprite's Y
    //   coordinate before writing it here. Hide a sprite by moving it down offscreen, by
    //   writing any values between #$EF-#$FF here. Sprites are never displayed on the first
    //   line of the picture, and it is impossible to place a sprite partially off the top of
    //   the screen.

    pub memory: PPUMemory,
    pub frame: Frame,
    pub oam: OAM,
    pub scroll_ctx: ScrollContext,
    pub data_buffer: u8,

    pub cycles: usize,
    pub scanline: u16,
    pub nmi_flag: bool,
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
            scroll_ctx: ScrollContext::new(),
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
        if self.cycles > 340 {
            self.render_scanline();

            if self.scanline < 240 {
                self.oam_addr = 0; // todo: is this enough? https://www.nesdev.org/wiki/PPU_registers
            }

            if self.scanline == 241 {
                self.status.set(VerticalBlank);
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

        self.render_background_scanline();
        self.render_sprites_scanline();
    }

    #[inline]
    pub fn render_background_scanline(&mut self) {
        if self.mask.is_clear(ShowBackground) { return }

        self.scroll_ctx.handle_scanline_start(self.scanline);

        let background_bank = self.ctrl.get_background_chrtable_address();
        let screen_y = self.scanline as usize;
        let tile_y = self.scroll_ctx.get_coarse_scroll_y();
        let pixel_y = 8 * tile_y + self.scroll_ctx.get_fine_scroll_y();
        for screen_x in 0..Frame::WIDTH {
            let tile_address = self.scroll_ctx.get_tile_address();
            let tile_x = self.scroll_ctx.get_coarse_scroll_x();
            let pixel_x = screen_x + self.scroll_ctx.get_fine_scroll_x() as usize;

            if pixel_x % 8 == 7 {
                self.scroll_ctx.scroll_x_increment();
            }

            let pallete = self.bg_pal(tile_x, tile_y);
            let tile_value = self.memory.read_byte(tile_address) as u16;
            let chr_address = background_bank + 16 * tile_value;

            let chr_y = (pixel_y % 8) as u16;
            let tile_lower_chr = self.memory.read_byte(chr_address + chr_y);
            let tile_upper_chr = self.memory.read_byte(chr_address + chr_y + 8);

            let chr_x = 7 - (pixel_x % 8);
            let lower = tile_lower_chr >> chr_x;
            let upper = tile_upper_chr >> chr_x;
            let palette_value = (1 & upper) << 1 | (1 & lower);
            let palette_index = pallete[palette_value as usize];
            let rgb = NES::SYSTEM_PALLETE[palette_index as usize];
            let priority = if palette_value == 0 { Frame::BG_PRIORITY } else { Frame::FG_PRIORITY };
            self.frame.set_background_pixel(screen_x, screen_y, rgb, priority);
        }

        self.scroll_ctx.scroll_y_increment();
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

            let priority = if self.oam.memory[sprite_idx + 2] >> 5 & 1 == 0 { Frame::FG_PRIORITY } else { Frame::BG_PRIORITY } ;
            let mut tile_value = self.oam.memory[sprite_idx + 1] as u16;

            let flip_vertical = self.oam.memory[sprite_idx + 2] >> 7 & 1 == 1;
            let flip_horizontal = self.oam.memory[sprite_idx + 2] >> 6 & 1 == 1;
            let palette_idx = self.oam.memory[sprite_idx + 2] & 0b0000_0011;
            let sprite_palette = self.sprite_palette(palette_idx);

            let y = screen_y - sprite_y;
            let mut chr_y = if flip_vertical { sprite_size - 1 - y } else { y } as u16;
            let mut tile_addr = sprites_bank + 16 * tile_value;
            if sprite_size == 16 {
                let sprites_bank = if tile_value & 1 == 1 { 0x1000 } else { 0x0000 };
                tile_value = if tile_value % 2 == 1 { tile_value - 1 } else { tile_value };
                tile_value = if chr_y >= 8 { tile_value + 1 } else { tile_value };
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
                    // todo: "screen_y + 1" might be wrong here
                    self.frame.set_sprite_pixel(screen_x, screen_y + 1, rgb, priority);
                    if sprite_idx == 0 {
                        self.status.set(SpriteZeroHit);
                    }
                }
            }
        }
    }

    #[inline]
    fn bg_palette(&mut self, tile_x: u8, tile_y: u8) -> [u8; 4] {
        let attribute_address = self.scroll_ctx.get_attribute_address();
        let attr_byte = self.memory.read_byte(attribute_address);
        let pallete_val = match ((tile_x % 4) / 2, (tile_y % 4) / 2) {
            (0, 0) => attr_byte & 0b0000_0011,
            (1, 0) => (attr_byte >> 2) & 0b0000_0011,
            (0, 1) => (attr_byte >> 4) & 0b0000_0011,
            (1, 1) => (attr_byte >> 6) & 0b0000_0011,
            (_, _) => panic!("can't be"),
        };
        let pallete_idx = 4 * pallete_val as u16;
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
        self.scroll_ctx.handle_scroll_reg_write(value);
        self.flip_address_latch();
    }

    pub fn write_addr_register(&mut self, value: u8) {
        self.addr.write(value);
        self.scroll_ctx.handle_addr_reg_write(value);
        self.flip_address_latch();
    }

    pub fn read_data_register(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        let result = self.data_buffer;
        self.data_buffer = self.memory.read_byte(addr);
        self.scroll_ctx.handle_data_reg_read_write();
        result
    }

    pub fn write_data_register(&mut self, value: u8) {
        let addr = self.addr.get();
        self.increment_vram_addr();

        self.data = value;
        self.memory.write_byte(addr, value);
        self.scroll_ctx.handle_data_reg_read_write();
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
        self.scroll_ctx.handle_cntl_reg_write(value);
        if !before_nmi_status && self.ctrl.is_set(GenerateNmi) && self.status.is_set(VerticalBlank) {
            self.set_nmi();
        }
    }

    pub fn write_mask_register(&mut self, value: u8) {
        self.mask.set_value(value);
    }

    pub fn read_status_register(&mut self) -> u8 {
        let status = self.status.get_value();
        self.status.clear(VerticalBlank);
        self.clear_address_latch();
        status
    }

    #[inline]
    pub fn get_address_latch(&self) -> bool {
        self.scroll_ctx.w
    }

    #[inline]
    pub fn flip_address_latch(&mut self) {
        if self.get_address_latch() {
            self.clear_address_latch();
        } else {
            self.set_address_latch();
        }
    }

    #[inline]
    pub fn set_address_latch(&mut self) {
        self.scroll_ctx.w = true;
        self.scroll.latch = true;
        self.addr.latch = true;
    }

    #[inline]
    pub fn clear_address_latch(&mut self) {
        self.scroll_ctx.w = false;
        self.scroll.latch = false;
        self.addr.latch = false;
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