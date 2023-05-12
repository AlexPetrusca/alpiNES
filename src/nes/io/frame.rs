pub struct Frame {
    pub rgb: Vec<u8>,
    pub alpha: Vec<u8>,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub const FOREGROUND: u8 = 1;
    pub const BACKGROUND: u8 = 1;

    pub fn new() -> Self {
        Frame {
            rgb: vec![0; 3 * Frame::WIDTH * Frame::HEIGHT],
            alpha: vec![0; Frame::WIDTH * Frame::HEIGHT]
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        for i in 0..self.rgb.len() {
            self.rgb[i] = 0;
        }
        for i in 0..self.alpha.len() {
            self.alpha[i] = 0;
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
    pub fn is_alpha_set(&self, x: usize, y: usize) -> bool {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            return self.alpha[Frame::WIDTH * y + x] != 0;
        }
        return false;
    }

    #[inline]
    pub fn is_pixel_set(&self, x: usize, y: usize) -> bool {
        return self.is_color_set(x, y) || self.is_alpha_set(x, y);
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
    pub fn get_alpha(&self, x: usize, y: usize) -> u8 {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            return self.alpha[Frame::WIDTH * y + x];
        }
        return 0;
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8, u8) {
        let rgb = self.get_color(x, y);
        let alpha = self.get_alpha(x, y);
        return (rgb.0, rgb.1, rgb.2, alpha);
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
    pub fn set_alpha(&mut self, x: usize, y: usize, alpha: u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            self.rgb[Frame::WIDTH * y + x] = alpha;
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, rgba: (u8, u8, u8, u8)) {
        self.set_color(x, y, (rgba.0, rgba.1, rgba.2));
        self.set_alpha(x, y, rgba.3);
    }
}