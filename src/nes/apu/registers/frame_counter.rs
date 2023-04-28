pub struct FrameCounterRegister {
    value: u8, // MI-- ----	   Mode (M, 0 = 4-step, 1 = 5-step), IRQ inhibit flag (I)
    pub counter: u16,
}

impl FrameCounterRegister {
    pub fn new() -> Self {
        FrameCounterRegister {
            value: 0,
            counter: 0
        }
    }

    pub fn read(&self) -> u8 {
        self.value & 0b1100_0000
    }

    pub fn write(&mut self, data: u8) {
        self.value = data;
    }

    pub fn is_irq_enabled(&self) -> bool {
        self.value & 0b0100_0000 > 0
    }

    pub fn is_irq_disabled(&self) -> bool {
        !self.is_irq_enabled()
    }

    pub fn set_irq_inhibit(&mut self) {
        self.value = self.value | 0b0100_0000;
    }

    pub fn clear_irq_inhibit(&mut self) {
        self.value = self.value & 0b1011_1111;
    }

    pub fn is_five_step_mode(&self) -> bool {
        self.value & 0b1000_0000 > 0
    }

    pub fn is_four_step_mode(&self) -> bool {
        !self.is_five_step_mode()
    }

    pub fn increment(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    pub fn get_counter(&self) -> u16 {
        self.counter
    }

    pub fn get_step(&self) -> u8 {
        if self.is_four_step_mode() {
            (self.counter % 4) as u8
        } else {
            (self.counter % 5) as u8
        }
    }

    pub fn reset(&mut self) {
        self.counter = 0
    }
}