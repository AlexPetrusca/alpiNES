use joycon_status::JoyconStatus;
use crate::nes::io::joycon::joycon_status::JoyconButton;
use crate::util::bitvec::BitVector;

pub mod joycon_status;

pub struct Joycon {
    strobe: bool,
    button_index: u8,
    button_status: JoyconStatus,
    // turbo control
    turbo_control_a: bool,
    turbo_control_b: bool,
    turbo_control_interval: u8,
    turbo_control_counter: u8,
}

impl Joycon {
    pub fn new() -> Self {
        Joycon {
            strobe: false,
            button_index: 0,
            button_status: JoyconStatus::new(),
            // turbo control
            turbo_control_a: false,
            turbo_control_b: false,
            turbo_control_interval: 32,
            turbo_control_counter: 0,
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
        if !self.strobe {
            self.button_index += 1;
        }

        if (self.turbo_control_a && self.is_button_pressed(JoyconButton::A) && button == JoyconButton::A)
            || (self.turbo_control_b && self.is_button_pressed(JoyconButton::B) && button == JoyconButton::B) {
            self.turbo_control_counter = (self.turbo_control_counter + 1) % self.turbo_control_interval;
            (self.turbo_control_counter < self.turbo_control_interval / 2) as u8
        } else {
            self.is_button_pressed(button) as u8
        }
    }

    pub fn set_button(&mut self, button: JoyconButton) {
        self.button_status.set(button);
    }

    pub fn clear_button(&mut self, button: JoyconButton) {
        if (self.turbo_control_a && button == JoyconButton::A)
            || (self.turbo_control_b && button == JoyconButton::B) {
            self.turbo_control_counter = 0;
        }
        self.button_status.clear(button);
    }

    pub fn toggle_button(&mut self, button: JoyconButton) {
        if self.is_button_pressed(button.clone()) {
            self.button_status.clear(button);
        } else {
            self.button_status.set(button);
        }
    }

    pub fn is_button_pressed(&self, button: JoyconButton) -> bool {
        self.button_status.is_set(button)
    }

    pub fn toggle_turbo_control_a(&mut self) {
        self.turbo_control_a = !self.turbo_control_a;
    }

    pub fn toggle_turbo_control_b(&mut self) {
        self.turbo_control_b = !self.turbo_control_b;
    }

    pub fn speed_up_turbo_control(&mut self) {
        if self.turbo_control_interval > 8 {
            self.turbo_control_interval /= 2
        }
    }

    pub fn slow_down_turbo_control(&mut self) {
        if self.turbo_control_interval < 128 {
            self.turbo_control_interval *= 2
        }
    }
}
