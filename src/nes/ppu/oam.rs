pub struct OAM {
    pub memory: [u8; OAM::MEM_SIZE],
}

impl OAM {
    const MEM_SIZE: usize = 0x100 as usize; // 256 bytes

    pub fn new() -> Self {
        OAM {
            memory: [0; OAM::MEM_SIZE],
        }
    }

    pub fn get_sprite(&self, sprite_idx: u8) -> [u8; 4] {
        let sprite_start_idx = 4 * sprite_idx as usize;
        return [
            self.memory[sprite_start_idx],
            self.memory[sprite_start_idx + 1],
            self.memory[sprite_start_idx + 2],
            self.memory[sprite_start_idx + 3],
        ]
    }

    #[inline]
    pub fn read_byte(&self, addr: u8) -> u8 {
        self.memory[addr as usize]
    }

    #[inline]
    pub fn write_byte(&mut self, addr: u8, data: u8) {
        self.memory[addr as usize] = data;
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