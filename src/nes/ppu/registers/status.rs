use crate::util::bitvec::BitVector;

// 7  bit  0
// ---- ----
// VSO. ....
// |||| ||||
// |||+-++++- PPU open bus. Returns stale PPU bus contents.
// ||+------- Sprite overflow. The intent was for this flag to be set
// ||         whenever more than eight sprites appear on a scanline, but a
// ||         hardware bug causes the actual behavior to be more complicated
// ||         and generate false positives as well as false negatives; see
// ||         PPU sprite evaluation. This flag is set during sprite
// ||         evaluation and cleared at dot 1 (the second dot) of the
// ||         pre-render line.
// |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
// |          a nonzero background pixel; cleared at dot 1 of the pre-render
// |          line.  Used for raster timing.
// +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
//            Set at dot 1 of line 241 (the line *after* the post-render
//            line); cleared after reading $2002 and at dot 1 of the
//            pre-render line.

pub enum StatusFlag {
    OpenBus1,
    OpenBus2,
    OpenBus3,
    OpenBus4,
    OpenBus5,
    SpriteOverflow,
    SpriteZeroHit,
    VerticalBlank
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