use bitvec::order::Lsb0;
use bitvec::view::BitView;
use crate::util::bitvec::BitVector;

// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
// |||| +---- 1: Show background
// |||+------ 1: Show sprites
// ||+------- Emphasize red (green on PAL/Dendy)
// |+-------- Emphasize green (red on PAL/Dendy)
// +--------- Emphasize blue

pub enum MaskFlag {
    Greyscale,
    ShowBackgroundLeftmostEight,
    ShowSpritesLeftmostEight,
    ShowBackground,
    ShowSprites,
    EmphasizeRed,
    EmphasizeGreen,
    EmphasizeBlue
}

pub struct MaskRegister {
    pub value: u8,
}

impl BitVector for MaskRegister {
    type Flag = MaskFlag;

    #[inline]
    fn is_set(&self, flag: Self::Flag) -> bool {
        // self.value.view_bits::<Lsb0>()[flag as usize]
        self.value & (1 << (flag as u8)) > 0
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

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister { value: 0 }
    }

    pub fn from(value :u8) -> Self {
        MaskRegister { value }
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