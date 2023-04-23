pub struct AddressRegister {
    value: (u8, u8),
    latch: bool,
}

impl AddressRegister {
    const MIRROR_MASK: u16 = 0x8FFF; // todo: refactor away

    pub fn new() -> Self {
        AddressRegister {
            value: (0, 0), // high byte first, lo byte second
            latch: true,
        }
    }

    pub fn write(&mut self, data: u8) {
        if self.latch {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        if self.get() > 0x3fff {
            // mirror down addr above 0x3fff
            self.set(self.get() & AddressRegister::MIRROR_MASK);
        }
        self.latch = !self.latch;
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }
        if self.get() > 0x3fff {
            // mirror down addr above 0x3fff
            self.set(self.get() & AddressRegister::MIRROR_MASK);
        }
    }

    pub fn reset_latch(&mut self) {
        self.latch = true;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xff) as u8;
    }
}