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
use crate::nes::ppu::registers::ctrl::ControlFlag::GenerateNmi;
use crate::nes::ppu::registers::mask::{MaskFlag, MaskRegister};
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
        if self.cycles > 340 {
            // todo: condition x <= cycles is always true in is_sprite_0_hit()
            self.status.update(SpriteZeroHit, self.is_sprite_0_hit(self.cycles));
            self.render_scanline();
            // self.render_tileline();

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
        // if self.scanline == 240 { self.render_sprites(true); } // todo: remove
        if self.scanline == 260 { self.frame.clear(); }
        // if self.scanline == 261 { self.render_sprites(false); } // todo: remove
        if self.scanline > 240 { return }

        let background_bank = self.ctrl.get_background_chrtable_address();
        let (nametable1, nametable2) = self.get_nametables();

        let scroll_x = self.scroll.get_scroll_x() as usize;
        // let scroll_y = self.scroll.get_scroll_y() as usize;

        let screen_y = self.scanline as usize;
        for screen_x in 0..Frame::WIDTH {

            let tile_x = screen_x / 8;
            let tile_y = screen_y / 8;
            let palette = self.bg_palette(nametable, tile_x, tile_y);

            let tile_idx = nametable + 32 * tile_y as u16 + tile_x as u16;
            let tile_value = self.memory.read_byte(tile_idx) as u16;
            let tile_address = background_bank + 16 * tile_value;

            let chr_x = 7 - (screen_x % 8) as u16;
            let chr_y = (screen_y % 8) as u16;
            let mut lower_chr = self.memory.read_byte(tile_address + chr_y) >> chr_x;
            let mut upper_chr = self.memory.read_byte(tile_address + chr_y + 8) >> chr_x;

            let palette_value = (1 & upper_chr) << 1 | (1 & lower_chr);
            let rgb = match palette_value {
                0 => NES::SYSTEM_PALLETE[palette[0] as usize],
                1 => NES::SYSTEM_PALLETE[palette[1] as usize],
                2 => NES::SYSTEM_PALLETE[palette[2] as usize],
                3 => NES::SYSTEM_PALLETE[palette[3] as usize],
                _ => panic!("can't be"),
            };
            let alpha = if palette_value == 0 { Frame::BACKGROUND } else { Frame::FOREGROUND };
            let rgba = (rgb.0, rgb.1, rgb.2, alpha);
            self.frame.set_pixel(screen_x, screen_y, rgba)
        }
    }

    #[inline]
    pub fn render_tileline(&mut self) {
        if self.scanline == 261 { self.frame.clear(); }
        if self.scanline > 240 ||  self.scanline % 8 != 0 { return }

        let tile_y = self.scanline as usize / 8;
        self.render_background_tileline(tile_y);
        self.render_sprites_tileline(tile_y);
    }

    #[inline]
    fn render_sprites_tileline(&mut self, tile_y: usize) {
        if self.mask.is_clear(ShowSprites) { return }

        let bank = self.ctrl.get_sprite_chrtable_address();
        for i in (0..self.oam.memory.len()).step_by(4).rev() {
            let sprite_x = self.oam.memory[i + 3] as usize;
            let sprite_y = self.oam.memory[i] as usize;

            if !(sprite_y >= tile_y * 8 && sprite_y < (tile_y + 1) * 8) { continue }

            let priority = self.oam.memory[i + 2] >> 5 & 1;
            let tile_value = self.oam.memory[i + 1] as u16;

            let flip_vertical = self.oam.memory[i + 2] >> 7 & 1 == 1;
            let flip_horizontal = self.oam.memory[i + 2] >> 6 & 1 == 1;
            let palette_idx = self.oam.memory[i + 2] & 0b0000_0011;
            let sprite_palette = self.sprite_palette(palette_idx);

            for y in 0..8 {
                let tile_addr = bank + 16 * tile_value + y as u16;
                let mut lower = self.memory.read_byte(tile_addr);
                let mut upper = self.memory.read_byte(tile_addr + 8);
                'sprite_render: for x in (0..8).rev() {
                    let value = (1 & upper) << 1 | (1 & lower);
                    lower = lower >> 1;
                    upper = upper >> 1;
                    let rgb = match value {
                        0 => continue 'sprite_render, // skip coloring the pixel
                        1 => NES::SYSTEM_PALLETE[sprite_palette[1] as usize],
                        2 => NES::SYSTEM_PALLETE[sprite_palette[2] as usize],
                        3 => NES::SYSTEM_PALLETE[sprite_palette[3] as usize],
                        _ => panic!("can't be"),
                    };
                    let alpha = if priority == 0 { Frame::FOREGROUND_SPRITE } else { Frame::BACKGROUND_SPRITE };
                    let rgba = (rgb.0, rgb.1, rgb.2, alpha);
                    match (flip_horizontal, flip_vertical) {
                        (false, false) => self.frame.set_pixel(sprite_x + x, sprite_y + y + 1, rgba),
                        (true, false) => self.frame.set_pixel(sprite_x + 7 - x, sprite_y + y + 1, rgba),
                        (false, true) => self.frame.set_pixel(sprite_x + x, sprite_y + 8 - y, rgba),
                        (true, true) => self.frame.set_pixel(sprite_x + 7 - x, sprite_y + 8 - y, rgba),
                    }
                }
            }
        }
    }

    #[inline]
    fn render_background_tileline(&mut self, tile_y: usize) {
        if self.mask.is_clear(MaskFlag::ShowBackground) { return }

        let scroll_x = self.scroll.get_scroll_x() as usize;
        let scroll_y = self.scroll.get_scroll_y() as usize;

        let (nametable1, nametable2) = self.get_nametables();

        self.render_name_table_tileline(nametable1,
            Viewport::new(scroll_x, scroll_y, 256, 240),
            -(scroll_x as isize), -(scroll_y as isize), tile_y
        );
        if scroll_x > 0 {
            self.render_name_table_tileline(nametable2,
                Viewport::new(0, 0, scroll_x, 240),
                (256 - scroll_x) as isize, 0, tile_y
            );
        } else if scroll_y > 0 {
            if scroll_y >= 240 {
                self.render_name_table_tileline(nametable1,
                    Viewport::new(0, 0, 256, scroll_y),
                    0, (256 - scroll_y) as isize, tile_y
                );
            } else {
                self.render_name_table_tileline(nametable2,
                    Viewport::new(0, 0, 256, scroll_y),
                    0, 240 - scroll_y as isize, tile_y
                );
            }
        }
    }

    #[inline]
    fn render_name_table_tileline(&mut self, nametable_addr: u16, viewport: Viewport, shift_x: isize, shift_y: isize, tile_y: usize) {
        let bank = self.ctrl.get_background_chrtable_address();

        for tile_x in 0..32 {
            let tile_idx = nametable_addr + 32 * tile_y as u16 + tile_x as u16;
            let tile_value = self.memory.read_byte(tile_idx) as u16;
            let palette = self.bg_palette(nametable_addr, tile_x, tile_y);

            for y in 0..8 {
                let tile_addr = bank + 16 * tile_value + y;
                let mut upper = self.memory.read_byte(tile_addr);
                let mut lower = self.memory.read_byte(tile_addr + 8);

                for x in (0..8).rev() {
                    let value = (1 & lower) << 1 | (1 & upper);
                    upper = upper >> 1;
                    lower = lower >> 1;
                    let rgb = match value {
                        0 => NES::SYSTEM_PALLETE[palette[0] as usize],
                        1 => NES::SYSTEM_PALLETE[palette[1] as usize],
                        2 => NES::SYSTEM_PALLETE[palette[2] as usize],
                        3 => NES::SYSTEM_PALLETE[palette[3] as usize],
                        _ => panic!("can't be"),
                    };
                    let alpha = if value == 0 { Frame::BACKGROUND } else { Frame::FOREGROUND };
                    let rgba = (rgb.0, rgb.1, rgb.2, alpha);
                    let pixel_x = 8 * tile_x + x as usize;
                    let pixel_y = 8 * tile_y + y as usize;
                    if pixel_x >= viewport.x1 && pixel_x < viewport.x2 && pixel_y >= viewport.y1 && pixel_y < viewport.y2 {
                        let scroll_pixel_x = (shift_x + pixel_x as isize) as usize;
                        let scroll_pixel_y = (shift_y + pixel_y as isize) as usize;
                        self.frame.set_pixel(scroll_pixel_x, scroll_pixel_y, rgba);
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