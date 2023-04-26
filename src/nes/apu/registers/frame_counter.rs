pub struct FrameCounterRegister {
    value: u8,
}

impl FrameCounterRegister {
    pub fn new() -> Self {
        FrameCounterRegister {
            value: 0
        }
    }
}