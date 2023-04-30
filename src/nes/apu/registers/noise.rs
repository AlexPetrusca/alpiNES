pub struct NoiseRegisters {
    register_a: u8, // --LC VVVV	Envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
    register_b: u8, // ---- ----	Unused
    register_c: u8, // M--- PPPP	Mode (M), noise period (P)
    register_d: u8, // LLLL L---	Length counter load (L)
}

impl NoiseRegisters {
    const PERIOD_LOOKUP: [u16; 16] = [
        4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
    ];

    pub fn new() -> Self {
        NoiseRegisters {
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
        self.register_a & 0b0010_0000 > 0
    }

    pub fn is_one_shot_play(&self) -> bool {
        !self.is_infinite_play()
    }

    pub fn is_constant_volume(&self) -> bool {
        self.register_a & 0b0001_0000 > 0
    }

    pub fn is_envelope_volume(&self) -> bool {
        !self.is_constant_volume()
    }

    pub fn get_volume(&self) -> u8 {
        self.register_a & 0b0000_1111
    }

    pub fn get_envelope_rate(&self) -> u8 {
        self.get_volume()
    }

    pub fn get_period_idx(&self) -> u8 {
        self.register_c & 0b0000_1111
    }

    pub fn get_period(&self) -> u16 {
        return NoiseRegisters::PERIOD_LOOKUP[self.get_period_idx() as usize];
    }

    pub fn is_tone_mode(&self) -> bool {
        self.register_c & 0b1000_0000 > 0
    }

    pub fn get_length_counter(&self) -> u8 {
        (self.register_d & 0b1111_1000) >> 3
    }

    pub fn clear_length_counter(&mut self) {
        self.register_d = self.register_d & 0b0000_0111;
    }

    pub fn get_frequency(&self) -> f32 {
        (39_375_000.0 / 44.0) / self.get_period() as f32
    }
}