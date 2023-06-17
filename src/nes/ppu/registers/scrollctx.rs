// Ref: https://www.nesdev.org/wiki/PPU_scrolling

// v & t registers:
// =====================================
// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| ++-------------- nametable select
// +++----------------- fine Y scroll

pub struct ScrollContext {
    pub v: u16, // Current VRAM address (15 bits)
    pub t: u16, // Temporary VRAM address (15 bits)
    pub x: u8, // Fine X scroll (3 bits)
    pub w: bool, // First or second write toggle (1 bit)
}

impl ScrollContext {
    pub fn new() -> Self {
        ScrollContext {
            v: 0,
            t: 0,
            x: 0,
            w: false
        }
    }

    pub fn handle_cntl_reg_write(&mut self, value: u8) {
        self.t &= 0b1111_0011_1111_1111;
        self.t |= (value as u16 & 0b0000_0011) << 10;
    }

    pub fn handle_scroll_reg_write(&mut self, value: u8) {
        if !self.w {
            // first write
            self.x = value & 0b0000_0111;
            self.t &= 0b1111_1111_1110_0000;
            self.t |= (value as u16 & 0b1111_1000) >> 3;
        } else {
            // second write
            self.t &= 0b1000_1111_1111_1111;
            self.t |= (value as u16 & 0b0000_0111) << 12;
            self.t &= 0b1111_1100_0001_1111;
            self.t |= (value as u16 & 0b1111_1000) << 2;
        }
    }

    pub fn handle_addr_reg_write(&mut self, value: u8) {
        if !self.w {
            // first write
            self.t &= 0b1100_0000_1111_1111;
            self.t |= (value as u16 & 0b0011_1111) << 8;
            self.t &= 0b0011_1111_1111_1111;
        } else {
            // second write
            self.t &= 0b1111_1111_0000_0000;
            self.t |= value as u16;
            self.v = self.t;
        }
    }

    pub fn handle_data_reg_read_write(&mut self) {
        self.scroll_x_increment();
        self.scroll_y_increment();
    }

    pub fn handle_scanline_start(&mut self, scanline: u16) {
        if scanline == 0 {
            self.v = self.t;
        } else {
            self.v &= 0b1111_1011_1110_0000;
            self.v |= self.t & 0b0000_0100_0001_1111;
        }
    }

    // coarse X is incremented when the next tile is reached
    pub fn scroll_x_increment(&mut self) {
        if (self.v & 0x001F) == 31 { // if coarse X == 31
            self.v &= !0x001F; // coarse X = 0
            self.v ^= 0x0400; // switch horizontal nametable
        } else {
            self.v += 1; // increment coarse X
        }
    }

    // fine Y is incremented at dot 256 of each scanline, overflowing to coarse Y
    pub fn scroll_y_increment(&mut self) {
        if (self.v & 0x7000) != 0x7000 { // if fine Y < 7
            self.v += 0x1000; // increment fine Y
        } else {
            self.v &= !0x7000; // fine Y = 0
            let mut y = (self.v & 0x03E0) >> 5; // let y = coarse Y
            if y == 29 {
                y = 0; // coarse Y = 0
                self.v ^= 0x0800; // switch vertical nametable
            } else if y == 31 {
                y = 0; // coarse Y = 0, nametable not switched
            } else {
                y += 1; // increment coarse Y
            }
            self.v = (self.v & !0x03E0) | (y << 5); // put coarse Y back into v
        }
    }

    #[inline]
    pub fn get_nametable_address(&self) -> u16 {
        0x2000 | (self.v & 0x0C00)
    }

    #[inline]
    pub fn get_tile_address(&self) -> u16 {
        0x2000 | (self.v & 0x0FFF)
    }

    #[inline]
    pub fn get_attribute_address(&self) -> u16 {
        0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07)
    }

    #[inline]
    pub fn get_coarse_scroll_x(&self) -> u8 {
        (self.v & 0b0001_1111) as u8
    }

    #[inline]
    pub fn get_fine_scroll_x(&self) -> u8 {
        self.x
    }

    #[inline]
    pub fn get_coarse_scroll_y(&self) -> u8 {
        ((self.v & 0b0000_0011_1110_0000) >> 5) as u8
    }

    #[inline]
    pub fn get_fine_scroll_y(&self) -> u8 {
        ((self.v & 0b0111_0000_0000_0000) >> 12) as u8
    }
}