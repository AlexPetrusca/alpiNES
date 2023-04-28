pub struct TriangleRegisters {
    register_a: u8, // CRRR RRRR	Length counter halt / linear counter control (C), linear counter load (R)
    register_b: u8, // ---- ----	Unused
    register_c: u8, // TTTT TTTT	Timer low (T)
    register_d: u8, // LLLL LTTT	Length counter load (L), timer high (T), set linear counter reload flag
}

impl TriangleRegisters {
    pub fn new() -> Self {
        TriangleRegisters {
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

    pub fn is_infinite_play(&self) -> bool {
        self.register_a & 0b100_0000 > 0
    }

    pub fn is_one_shot_play(&self) -> bool {
        !self.is_infinite_play()
    }

    pub fn get_linear_counter(&self) -> u8 {
        self.register_a & 0b0111_1111
    }

    fn set_linear_counter(&mut self, value: u8) {
        self.register_a = (self.register_a & 0b1000_0000) | value;
    }

    pub fn decrement_linear_counter(&mut self) {
        let length_counter = self.get_linear_counter();
        if length_counter != 0 {
            self.set_linear_counter(length_counter - 1);
        }
    }

    pub fn get_timer(&self) -> u16 {
        ((self.register_d as u16 & 0b0000_0111) << 8) | self.register_c as u16
    }

    pub fn get_length_counter(&self) -> u8 {
        (self.register_d & 0b1111_1000) >> 3
    }

    fn set_length_counter(&mut self, value: u8) {
        self.register_d = (self.register_d & 0b0000_0111) | (value << 3);
    }

    pub fn decrement_length_counter(&mut self) {
        let length_counter = self.get_length_counter();
        if length_counter != 0 {
            self.set_length_counter(length_counter - 1);
        }
    }

    pub fn clear_length_counter(&mut self) {
        self.set_length_counter(0);
    }

    pub fn get_frequency(&self) -> f32 {
        1_789_773.0 / (32.0 * (self.get_timer() as f32 + 1.0))
    }
}