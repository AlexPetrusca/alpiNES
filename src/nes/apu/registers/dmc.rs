pub struct DMCRegisters {
    register_a: u8, // IL-- RRRR 	IRQ enable (I), loop (L), rate (R)
    register_b: u8, // -DDD DDDD	Load counter (D)
    register_c: u8, // AAAA AAAA	Sample address (A)
    register_d: u8, // LLLL LLLL	Sample length (L)
}

impl DMCRegisters {
    const RATE_LOOKUP: [u16; 16] = [
        428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54
    ];

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

    pub fn is_irq_enable(&self) -> bool {
        self.register_a & 0b1000_0000 > 0
    }

    pub fn is_loop(&self) -> bool {
        self.register_a & 0b0100_0000 > 0
    }

    pub fn is_one_shot(&self) -> bool {
        !self.is_loop()
    }

    pub fn get_rate_idx(&self) -> u8 {
        self.register_a & 0b0000_1111
    }

    pub fn get_rate(&self) -> u16 {
        return DMCRegisters::RATE_LOOKUP[self.get_rate_idx() as usize];
    }

    pub fn get_volume(&self) -> u8 {
        self.register_b & 0b0111_1111
    }

    pub fn get_sample_address(&self) -> u16 {
        0xC000 + 64 * self.register_c as u16
    }

    pub fn get_sample_length(&self) -> u16 {
        16 * self.register_d as u16 + 1
    }

    pub fn get_frequency(&self) -> f32 {
        1_789_773.0 / self.get_rate() as f32
    }
}