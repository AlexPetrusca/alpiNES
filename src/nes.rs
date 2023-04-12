use crate::cpu::CPU;
use crate::mem::Memory;
use crate::ppu::PPU;
use crate::rom::ROM;

pub struct NES {
    pub cpu: CPU,
    pub ppu: PPU,
    pub mem: Memory,
}

impl NES {
    pub fn new() -> Self {
        NES {
            cpu: CPU::new(),
            ppu: PPU::new(),
            mem: Memory::new(),
        }
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        self.cpu.step(&mut self.mem)
    }

    pub fn load(&mut self, program: &Vec<u8>) {
        self.load_at_addr(Memory::PRG_ROM_START, program);
    }

    pub fn load_at_addr(&mut self, addr: u16, program: &Vec<u8>) {
        self.mem.load_at_addr(addr, program);
        self.reset();
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.mem.load_rom(rom);
        self.reset();
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.program_counter = self.mem.read_addr(Memory::RESET_INT_VECTOR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nes_load() {
        let mut nes = NES::new();
        assert_eq!(nes.cpu.program_counter, 0);
        nes.load(&vec![CPU::LDA_IM, 5, CPU::ROR, CPU::BRK]);
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START);
    }

    #[test]
    fn test_nes_reset() {
        let mut nes = NES::new();
        nes.load(&vec![CPU::LDA_IM, 5, CPU::ROR, CPU::BRK]);
        nes.step().unwrap();
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START + 2);
        nes.reset();
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START);
    }

    #[test]
    fn test_nes_step() {
        let mut nes = NES::new();
        nes.load(&vec![CPU::LDA_IM, 5, CPU::ROR, CPU::BRK]);
        nes.step().unwrap();
        assert_eq!(nes.cpu.register_a, 0x05);
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START + 2);
        nes.step().unwrap();
        assert_eq!(nes.cpu.register_a, 0x02);
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START + 3);
        nes.step().unwrap_or_default();
        assert_eq!(nes.cpu.status, 0b0011_0001);
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START + 4);
    }
}