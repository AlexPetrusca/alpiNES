use crate::util::bitvec::BitVector;

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
    pub value: u8,
}

impl BitVector for StatusRegister {
    type Flag = StatusFlag;

    #[inline]
    fn is_set(&self, flag: Self::Flag) -> bool {
        self.value & 1 << (flag as u8) != 0
    }

    #[inline]
    fn set(&mut self, flag: Self::Flag) {
        self.value |= 1 << (flag as u8)
    }

    #[inline]
    fn clear(&mut self, flag: Self::Flag) {
        self.value &= !(1 << (flag as u8))
    }
}

impl StatusRegister {
    const B_FLAG_MASK: u8 = 0b0011_0000;
    const B_FLAG_INTERRUPT_SET_MASK: u8 = 0b0010_0000;
    const B_FLAG_INTERRUPT_CLEAR_MASK: u8 = 0b1110_1111;

    pub fn new() -> Self {
        StatusRegister { value: 0b0011_0000 }
    }

    pub fn from(value: u8) -> Self {
        StatusRegister { value }
    }

    #[inline]
    pub fn get_value(&self) -> u8 {
        self.value | Self::B_FLAG_MASK
    }

    #[inline]
    pub fn get_value_interrupt(&self) -> u8 {
        (self.value | Self::B_FLAG_INTERRUPT_SET_MASK) & Self::B_FLAG_INTERRUPT_CLEAR_MASK
    }

    #[inline]
    pub fn set_value(&mut self, value: u8) {
        self.value = value;
    }

    #[inline]
    pub fn set_value_interrupt(&mut self, value: u8) {
        self.value = (value | Self::B_FLAG_INTERRUPT_SET_MASK) & Self::B_FLAG_INTERRUPT_CLEAR_MASK
    }
}