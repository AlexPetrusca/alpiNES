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

    pub fn get_timer(&self) -> u16 {
        ((self.register_d as u16 & 0b0000_0111) << 8) | self.register_c as u16
    }

    pub fn get_length_counter(&self) -> u8 {
        (self.register_d & 0b1111_1000) >> 3
    }

    pub fn clear_length_counter(&mut self) {
        self.register_d = self.register_d & 0b0000_0111;
    }
}