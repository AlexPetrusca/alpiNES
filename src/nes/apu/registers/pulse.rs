pub struct PulseRegisters {
    register_a: u8, // DDLC VVVV	Duty (D), envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
    register_b: u8, // EPPP NSSS	Sweep unit: enabled (E), period (P), negate (N), shift (S)
    register_c: u8, // TTTT TTTT	Timer low (T)
    register_d: u8, // LLLL LTTT	Length counter load (L), timer high (T)
}

impl PulseRegisters {
    pub fn new() -> Self {
        PulseRegisters {
            register_a: 0b0011_0000,
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

    pub fn get_duty_bits(&self) -> u8 {
        (self.register_a & 0b1100_0000) >> 6
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

    pub fn is_sweep_enabled(&self) -> bool {
        self.register_b & 0b1000_0000 > 0
    }

    pub fn get_sweep_period(&self) -> u8 {
        self.register_b & 0b0111_0000 >> 4
    }

    pub fn is_sweep_negate(&self) -> bool {
        self.register_b & 0b0000_1000 > 0
    }

    pub fn get_sweep_shift(&self) -> u8 {
        self.register_b & 0b0000_0111
    }

    pub fn get_timer(&self) -> u16 {
        ((self.register_d as u16 & 0b0000_0111) << 8) | self.register_c as u16
    }

    pub fn get_length_counter_load(&self) -> u8 {
        (self.register_d & 0b1111_1000) >> 3
    }

    // pub fn get_frequency(&self) -> f32 {
    //
    // }
 }