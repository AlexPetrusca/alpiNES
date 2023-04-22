pub mod cpu;
pub mod ppu;
pub mod io;

use crate::util::rom::ROM;
use crate::nes::cpu::CPU;
use crate::nes::cpu::mem::Memory;
use crate::nes::ppu::PPU;

pub struct NES {
    pub cpu: CPU,
}

impl NES {
    pub const SYSTEM_PALLETE: [(u8, u8, u8); 64] = [
        (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
        (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
        (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
        (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
        (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
        (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
        (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
        (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
        (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
        (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
        (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
        (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
        (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
    ];

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
        nes.cpu.memory.ppu.data_buffer = 0xaa;
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
