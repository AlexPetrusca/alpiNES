use bitvec::order::Lsb0;
use bitvec::view::BitView;
use crate::io::bitvec::BitVector;

// 7  bit  0
// ---- ----
// NVss DIZC
// |||| ||||
// |||| |||+- Carry
// |||| ||+-- Zero
// |||| |+--- Interrupt Disable
// |||| +---- Decimal
// ||++------ No CPU effect, see: the B flag
// |+-------- Overflow
// +--------- Negative

pub enum StatusFlag {
    Carry,
    Zero,
    InterruptDisable,
    DecimalMode,
    BreakCommand,
    Unused,
    Overflow,
    Negative,
}

pub struct StatusRegister {
    value: u8,
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
    fn get_value(&self) -> u8 {
        self.value
    }

    #[inline]
    fn set_value(&mut self, value: u8) {
        self.value = value;
    }
}