pub struct ScrollRegister {
    pub value: (u8, u8),
    pub latch: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            value: (0, 0),
            latch: false,
        }
    }

    pub fn write(&mut self, data: u8) {
        if !self.latch {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }
    }

    pub fn get_scroll_x(&self) -> u8 {
        return self.value.0;
    }

    pub fn get_scroll_y(&self) -> u8 {
        return self.value.1;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }

    pub fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xff) as u8;
    }
}