pub struct Frame {
    pub rgb: Vec<u8>,
    pub priority: Vec<u8>,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub const BG_PRIORITY: u8 = 0;
    pub const FG_PRIORITY: u8 = 1;

    pub fn new() -> Self {
        Frame {
            rgb: vec![0; 3 * Frame::WIDTH * Frame::HEIGHT],
            priority: vec![0; Frame::WIDTH * Frame::HEIGHT]
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        for i in 0..self.rgb.len() {
            self.rgb[i] = 0;
        }
        for i in 0..self.priority.len() {
            self.priority[i] = 0;
        }
    }

    #[inline]
    pub fn is_color_set(&self, x: usize, y: usize) -> bool {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * Frame::WIDTH * y + 3 * x;
            return self.rgb[base] != 0 || self.rgb[base + 1] != 0 || self.rgb[base + 2] != 0;
        }
        return false;
    }

    #[inline]
    pub fn get_color(&self, x: usize, y: usize) -> (u8, u8, u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * Frame::WIDTH * y + 3 * x;
            return (self.rgb[base], self.rgb[base + 1], self.rgb[base + 2]);
        }
        return (0, 0, 0);
    }

    #[inline]
    pub fn get_priority(&self, x: usize, y: usize) -> u8 {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            return self.priority[Frame::WIDTH * y + x];
        }
        return 0;
    }

    #[inline]
    pub fn set_color(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * Frame::WIDTH * y + 3 * x;
            self.rgb[base] = rgb.0;
            self.rgb[base + 1] = rgb.1;
            self.rgb[base + 2] = rgb.2;
        }
    }

    #[inline]
    pub fn set_priority(&mut self, x: usize, y: usize, priority: u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            self.priority[Frame::WIDTH * y + x] = priority;
        }
    }

    #[inline]
    pub fn set_background_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8), priority: u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base_pri = Frame::WIDTH * y + x;
            let base_rgb = 3 * base_pri;
            self.rgb[base_rgb] = rgb.0;
            self.rgb[base_rgb + 1] = rgb.1;
            self.rgb[base_rgb + 2] = rgb.2;
            self.priority[base_pri] = priority;
        }
    }

    #[inline]
    pub fn set_sprite_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8), priority: u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            if priority == Frame::FG_PRIORITY || self.get_priority(x, y) == Frame::BG_PRIORITY {
                let base_rgb = 3 * (Frame::WIDTH * y + x);
                self.rgb[base_rgb] = rgb.0;
                self.rgb[base_rgb + 1] = rgb.1;
                self.rgb[base_rgb + 2] = rgb.2;
            }
        }
    }
}