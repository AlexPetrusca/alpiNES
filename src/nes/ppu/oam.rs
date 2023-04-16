pub struct OAM {
    memory: [u8; OAM::MEM_SIZE],
}

impl OAM {
    const MEM_SIZE: usize = 0x4000 as usize; // 16kB

    pub fn new() -> Self {
        OAM {
            memory: [0; OAM::MEM_SIZE],
        }
    }

    #[inline]
    pub fn read_byte(&self, address: u8) -> u8 {
        match address {
            _ => {
                panic!("Attempt to read from unmapped oam memory: 0x{:0>2X}", address);
            }
        }
    }

    #[inline]
    pub fn write_byte(&mut self, address: u8, data: u8) {
        match address {
            _ => {
                panic!("Attempt to write to unmapped oam memory: 0x{:0>2X}", address);
            }
        }
    }

    #[inline]
    pub fn oam_read_byte(&self, address: u8) -> u8 {
        todo!();
    }

    #[inline]
    pub fn oam_write_byte(&mut self, address: u8, data: u8) {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTE_A: u8 = 0x0a;
    const BYTE_B: u8 = 0x0b;

    // todo: add more tests for memory

    #[test]
    fn test_read_write() {
        let memory = OAM::new();
    }
}