use crate::util::bitvec::BitVector;

// 7  bit  0
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Generate an NMI at the start of the
//            vertical blanking interval (0: off; 1: on)

pub enum ControlFlag {
    NameTableAddrLow,
    NameTableAddrHigh,
    VramAddIncrement,
    SpritePatternAddr,
    BackgroundPatternAddr,
    SpriteSize,
    MasterSlaveSelect,
    GenerateNmi,
}

pub struct ControlRegister {
    pub value: u8,
}

impl BitVector for ControlRegister {
    type Flag = ControlFlag;

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

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister { value: 0 }
    }

    pub fn from(value :u8) -> Self {
        ControlRegister { value }
    }

    #[inline]
    pub fn get_vram_addr_increment(&self) -> u8 {
        if self.is_set(ControlFlag::VramAddIncrement) { 32 } else { 1 }
    }

    #[inline]
    pub fn get_sprite_chrtable_address(&self) -> u16 {
        if self.is_set(ControlFlag::SpritePatternAddr) { 0x1000 } else { 0x0000 }
    }

    #[inline]
    pub fn get_background_chrtable_address(&self) -> u16 {
        if self.is_set(ControlFlag::BackgroundPatternAddr) { 0x1000 } else { 0x0000 }
    }

    #[inline]
    pub fn get_base_nametable_address(&self) -> u16 {
        let nametable_index = self.get_value() & 0b0000_0011;
        return 0x2000 + 0x400 * nametable_index as u16;
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