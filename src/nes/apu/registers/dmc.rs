pub struct DMCRegisters {
    register_a: u8,
    register_b: u8,
    register_c: u8,
    register_d: u8,
}

impl DMCRegisters {
    pub fn new() -> Self {
        DMCRegisters {
            register_a: 0,
            register_b: 0,
            register_c: 0,
            register_d: 0,
        }
    }

    pub fn read(&self, index: u8) -> u8 {
        match index {
            0 => self.register_a,
            1 => self.register_b,
            2 => self.register_c,
            3 => self.register_d,
            _ => {
                panic!("Index out of bounds: {}", index);
            },
        }
    }

    pub fn write(&mut self, index: u8, data: u8) {
        match index {
            0 => self.register_a = data,
            1 => self.register_b = data,
            2 => self.register_c = data,
            3 => self.register_d = data,
            _ => {
                panic!("Index out of bounds: {}", index);
            },
        }
    }
}