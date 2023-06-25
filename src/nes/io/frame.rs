pub struct Frame {
    pub background: Vec<u8>,
    pub background_priority: Vec<u8>,
    pub sprite: Vec<u8>,
    pub sprite_priority: Vec<u8>,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub const EMPTY_PRIORITY: u8 = 0;
    pub const BG_PRIORITY: u8 = 1;
    pub const FG_PRIORITY: u8 = 2;

    pub fn new() -> Self {
        Frame {
            background: vec![0; 3 * Frame::WIDTH * Frame::HEIGHT],
            background_priority: vec![0; Frame::WIDTH * Frame::HEIGHT],
            sprite: vec![0; 3 * Frame::WIDTH * Frame::HEIGHT],
            sprite_priority: vec![0; Frame::WIDTH * Frame::HEIGHT],
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        // todo: hold on... do we even need to clear the screen?
        //  Frame gets redrawn each time anyways
        self.background = vec![0; 3 * Frame::WIDTH * Frame::HEIGHT];
        self.background_priority = vec![0; Frame::WIDTH * Frame::HEIGHT];
        self.sprite = vec![0; 3 * Frame::WIDTH * Frame::HEIGHT];
        self.sprite_priority = vec![0; Frame::WIDTH * Frame::HEIGHT];
    }

    // #[inline]
    // pub fn is_color_set(&self, x: usize, y: usize) -> bool {
    //     if x < Frame::WIDTH && y < Frame::HEIGHT {
    //         let base = 3 * Frame::WIDTH * y + 3 * x;
    //         return self.background[base] != 0 || self.background[base + 1] != 0 || self.background[base + 2] != 0;
    //     }
    //     return false;
    // }

    #[inline]
    pub fn get_background_color(&self, x: usize, y: usize) -> (u8, u8, u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * Frame::WIDTH * y + 3 * x;
            return (self.background[base], self.background[base + 1], self.background[base + 2]);
        }
        return (0, 0, 0);
    }

    #[inline]
    pub fn get_sprite_color(&self, x: usize, y: usize) -> (u8, u8, u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * Frame::WIDTH * y + 3 * x;
            return (self.sprite[base], self.sprite[base + 1], self.sprite[base + 2]);
        }
        return (0, 0, 0);
    }

    #[inline]
    pub fn get_background_priority(&self, x: usize, y: usize) -> u8 {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            return self.background_priority[Frame::WIDTH * y + x];
        }
        return 0;
    }

    #[inline]
    pub fn get_sprite_priority(&self, x: usize, y: usize) -> u8 {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            return self.sprite_priority[Frame::WIDTH * y + x];
        }
        return 0;
    }

    #[inline]
    pub fn set_background_color(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * Frame::WIDTH * y + 3 * x;
            self.background[base] = rgb.0;
            self.background[base + 1] = rgb.1;
            self.background[base + 2] = rgb.2;
        }
    }

    // #[inline]
    // pub fn set_priority(&mut self, x: usize, y: usize, priority: u8) {
    //     if x < Frame::WIDTH && y < Frame::HEIGHT {
    //         self.background_priority[Frame::WIDTH * y + x] = priority;
    //     }
    // }

    #[inline]
    pub fn set_background_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8), priority: u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base_pri = Frame::WIDTH * y + x;
            let base_rgb = 3 * base_pri;
            self.background[base_rgb] = rgb.0;
            self.background[base_rgb + 1] = rgb.1;
            self.background[base_rgb + 2] = rgb.2;
            self.background_priority[base_pri] = priority;
        }
    }

    #[inline]
    pub fn set_sprite_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8), priority: u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base_pri = Frame::WIDTH * y + x;
            let base_rgb = 3 * (Frame::WIDTH * y + x);
            self.sprite[base_rgb] = rgb.0;
            self.sprite[base_rgb + 1] = rgb.1;
            self.sprite[base_rgb + 2] = rgb.2;
            self.sprite_priority[base_pri] = priority;
        }
    }

    #[inline]
    pub fn compose(&mut self) -> &Vec<u8> {
        for y in 0..Frame::HEIGHT {
            for x in 0..Frame::WIDTH {
                let sp = self.get_sprite_priority(x, y);
                let bp = self.get_background_priority(x, y);
                let is_foreground = sp == Frame::FG_PRIORITY || bp == Frame::BG_PRIORITY;
                if sp != Frame::EMPTY_PRIORITY && is_foreground {
                    self.set_background_color(x, y, self.get_sprite_color(x, y));
                }
            }
        }
        return &self.background;
    }
}