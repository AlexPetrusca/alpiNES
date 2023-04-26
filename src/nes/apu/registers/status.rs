pub struct StatusRegister {
    value: u8,
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister {
            value: 0
        }
    }
}