pub mod cpu;
pub mod ppu;

use crate::io::rom::ROM;
use crate::nes::cpu::CPU;
use crate::nes::cpu::mem::Memory;
use crate::nes::ppu::PPU;

pub struct NES {
    pub cpu: CPU,
    pub ppu: PPU,
}

impl NES {
    pub fn new() -> Self {
        NES {
            cpu: CPU::new(),
            ppu: PPU::new(),
        }
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        self.cpu.step()
    }

    pub fn load(&mut self, program: &Vec<u8>) {
        self.load_at_addr(Memory::PRG_ROM_START, program);
    }

    pub fn load_at_addr(&mut self, addr: u16, program: &Vec<u8>) {
        self.cpu.memory.load_at_addr(addr, program);
        self.reset();
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.cpu.memory.load_rom(rom);
        self.reset();
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.program_counter = self.cpu.memory.read_addr(Memory::RESET_INT_VECTOR);
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
        assert_eq!(nes.cpu.status, 0b0010_0101);
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START + 4);
    }
}
