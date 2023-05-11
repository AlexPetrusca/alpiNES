pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            data: vec![0; (Frame::WIDTH) * (Frame::HEIGHT) * 3],
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.data.len() {
            self.data[i] = 0;
        }
    }

    pub fn is_pixel_set(&self, x: usize, y: usize) -> bool {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * y * Frame::WIDTH + 3 * x;
            return self.data[base] != 0 || self.data[base + 1] != 0 || self.data[base + 2] != 0;
        }
        return false;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * y * Frame::WIDTH + 3 * x;
            return (self.data[base], self.data[base + 1], self.data[base + 2]);
        }
        return (0, 0, 0);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        if x < Frame::WIDTH && y < Frame::HEIGHT {
            let base = 3 * y * Frame::WIDTH + 3 * x;
            if base + 2 < self.data.len() {
                self.data[base] = rgb.0;
                self.data[base + 1] = rgb.1;
                self.data[base + 2] = rgb.2;
            }
        }
    }
}