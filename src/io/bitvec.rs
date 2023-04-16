pub trait BitVector {
    type Flag;

    fn is_set(&self, flag: Self::Flag) -> bool;

    fn is_clear(&self, flag: Self::Flag) -> bool {
        !self.is_set(flag)
    }

    fn set(&mut self, flag: Self::Flag);

    fn clear(&mut self, flag: Self::Flag);

    fn update(&mut self, flag: Self::Flag, value: bool) {
        if value {
            self.set(flag);
        } else {
            self.clear(flag);
        };
    }

    fn get_value(&self) -> u8;

    fn set_value(&mut self, value: u8);
}