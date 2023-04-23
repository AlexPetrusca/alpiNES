use bitvec::order::Lsb0;
use bitvec::view::BitView;
use crate::util::bitvec::BitVector;

#[derive(Debug, PartialEq, Clone)]
pub enum JoyconButton {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl JoyconButton {
    pub fn from_value(value: u8) -> Self {
        match value {
            0 => JoyconButton::A,
            1 => JoyconButton::B,
            2 => JoyconButton::Select,
            3 => JoyconButton::Start,
            4 => JoyconButton::Up,
            5 => JoyconButton::Down,
            6 => JoyconButton::Left,
            7 => JoyconButton::Right,
            _ => {
                panic!("Invalid value for JoyconButton: {}", value)
            }
        }
    }
}

pub struct JoyconStatus {
    value: u8
}

impl BitVector for JoyconStatus {
    type Flag = JoyconButton;

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

impl JoyconStatus {
    pub fn new() -> Self {
        JoyconStatus { value: 0 }
    }

    pub fn from(value :u8) -> Self {
        JoyconStatus { value }
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