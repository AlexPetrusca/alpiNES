use bitvec::order::Lsb0;
use bitvec::view::BitView;
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
        self.value.view_bits::<Lsb0>()[flag as usize]
    }

    #[inline]
    fn set(&mut self, flag: Self::Flag) {
        self.value.view_bits_mut::<Lsb0>().set(flag as usize, true);
    }

    #[inline]
    fn clear(&mut self, flag: Self::Flag) {
        self.value.view_bits_mut::<Lsb0>().set(flag as usize, false);
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