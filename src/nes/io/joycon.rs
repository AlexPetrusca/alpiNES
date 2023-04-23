use joycon_status::JoyconStatus;
use crate::nes::io::joycon::joycon_status::JoyconButton;
use crate::util::bitvec::BitVector;

pub mod joycon_status;

pub struct Joycon {
    strobe: bool,
    button_index: u8,
    button_status: JoyconStatus,
}

impl Joycon {
    pub fn new() -> Self {
        Joycon {
            strobe: false,
            button_index: 0,
            button_status: JoyconStatus::new(),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_index = 0
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }
        let button = JoyconButton::from_value(self.button_index);
        let response = self.button_status.is_set(button) as u8;
        if !self.strobe {
            self.button_index += 1;
        }
        response
    }

    pub fn set_button(&mut self, button: JoyconButton) {
        self.button_status.set(button);
    }

    pub fn clear_button(&mut self, button: JoyconButton) {
        self.button_status.clear(button);
    }
}
