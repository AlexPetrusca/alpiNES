#[derive(Clone)]
pub struct ShiftRegister {
    pub value: u8,
    pub shift: u8
}

impl ShiftRegister {
    pub fn new() -> Self {
        Self {
            value: 0,
            shift: 0
        }
    }

    pub fn write(&mut self, value: u8) {
        if self.shift == 5 {
            self.clear();
        }
        if (value >> 7) & 1 == 0 {
            let bit_0 = value & 1;
            self.value |= bit_0 << self.shift;
            self.shift += 1;
        } else {
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.value = 0;
        self.shift = 0;
    }

    pub fn is_fifth_write(&self) -> bool {
        self.shift == 5
    }
}