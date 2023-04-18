pub mod cpu;
pub mod ppu;

use crate::io::rom::ROM;
use crate::nes::cpu::CPU;
use crate::nes::cpu::mem::Memory;
use crate::nes::ppu::PPU;

pub struct NES {
    pub cpu: CPU,
}

impl NES {
    pub fn new() -> Self {
        NES {
            cpu: CPU::new(),
        }
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        self.cpu.step()?;
        self.cpu.memory.ppu.step()
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

    #[test]
    fn test_nes_nop_dop_top() {
        let mut nes = NES::new();
        let program = vec![
            CPU::NOP, // single nop
            CPU::DOP_IM_1, 0xff, // double nop
            CPU::TOP_AB, 0xff, 0xff, // triple nop
            CPU::BRK
        ];
        nes.load(&program);

        let mut old_pc = nes.cpu.program_counter;
        nes.step().unwrap();
        assert_eq!(nes.cpu.program_counter, old_pc + 1);

        old_pc = nes.cpu.program_counter;
        nes.step().unwrap();
        assert_eq!(nes.cpu.program_counter, old_pc + 2);

        old_pc = nes.cpu.program_counter;
        nes.step().unwrap();
        assert_eq!(nes.cpu.program_counter, old_pc + 3);

        nes.step().unwrap_or_default();
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_nes_read_ppu_ram() {
        let mut nes = NES::new();
        nes.cpu.memory.ppu.memory.write_byte(0x26ab, 0xff);
        nes.cpu.memory.ppu.buffer = 0xaa;
        let program = vec![
            // write addr 0x0600 to addr register
            CPU::LDA_IM, 0x26, CPU::STA_AB, 0x06, 0x20,
            CPU::LDA_IM, 0xab, CPU::STA_AB, 0x06, 0x20,
            // read data register twice to get value at 0x0600
            CPU::LDA_AB, 0x07, 0x20,
            CPU::LDA_AB, 0x07, 0x20,
            CPU::BRK
        ];
        nes.load(&program);

        nes.step().unwrap();
        nes.step().unwrap();
        assert_eq!(nes.cpu.memory.ppu.addr.get(), 0x2600);

        nes.step().unwrap();
        nes.step().unwrap();
        assert_eq!(nes.cpu.memory.ppu.addr.get(), 0x26ab);

        nes.step().unwrap();
        assert_eq!(nes.cpu.register_a, 0xaa);

        nes.step().unwrap();
        assert_eq!(nes.cpu.register_a, 0xff);

        nes.step().unwrap_or_default();
        assert_eq!(nes.cpu.program_counter, Memory::PRG_ROM_START + program.len() as u16);
    }
}
