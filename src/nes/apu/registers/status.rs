use crate::util::bitvec::BitVector;

// 7  bit  0
// ---- ----
// IF-D NT21
// || | ||||
// || | |||+- Pulse 1 enable
// || | ||+-- Pulse 2 enable
// || | |+--- Triangle enable
// || | +---- Noise enable
// || +------ DMC enable
// |+-------- Frame interrupt
// +--------- DMC interrupt

pub struct StatusRegister {
    value: u8,
}

pub enum StatusFlag {
    PulseOneEnable,
    PulseTwoEnable,
    TriangleEnable,
    NoiseEnable,
    DmcEnable,
    Unused,
    FrameInterrupt,
    DmcInterrupt,
}

impl BitVector for StatusRegister {
    type Flag = StatusFlag;

    #[inline]
    fn is_set(&self, flag: Self::Flag) -> bool {
        self.value & (1 << (flag as u8)) != 0
    }

    #[inline]
    fn set(&mut self, flag: Self::Flag) {
        self.value |= (1 << (flag as u8))
    }

    #[inline]
    fn clear(&mut self, flag: Self::Flag) {
        self.value &= !(1 << (flag as u8))
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister { value: 0 }
    }

    pub fn from(value: u8) -> Self {
        StatusRegister { value }
    }

    #[inline]
    pub fn get_value(&self) -> u8 {
        self.value
    }

    #[inline]
    pub fn set_value(&mut self, value: u8) {
        self.value = value;
    }
}