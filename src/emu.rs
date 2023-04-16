use crate::nes::NES;
use crate::nes::cpu::CPU;
use crate::nes::mem::CPUMemory;
use crate::io::rom::ROM;

pub struct Emulator {
    pub nes: NES
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            nes: NES::new()
        }
    }

    pub fn load(&mut self, program: &Vec<u8>) {
        self.nes.load(program)
    }

    pub fn load_at_addr(&mut self, addr: u16, program: &Vec<u8>) {
        self.nes.load_at_addr(addr, program);
    }

    pub fn load_rom(&mut self, rom: &ROM) {
        self.nes.load_rom(rom);
    }

    pub fn load_and_run(&mut self, program: &Vec<u8>) {
        self.nes.load(program);
        self.run()
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F) where F: FnMut(&mut NES) {
        loop {
            callback(&mut self.nes);
            let Ok(_) = self.nes.step() else { return };
        }
    }

    pub fn reset(&mut self) {
        self.nes.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_and_reset() {
        let mut emu = Emulator::new();
        emu.load(&vec![0xff]);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START);
        assert_eq!(emu.nes.mem.read_byte(CPUMemory::PRG_ROM_START), 0xff);
    }

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut emu = Emulator::new();
        emu.load(&vec![CPU::LDA_IM, 5, CPU::BRK]);
        emu.run();
        assert_eq!(emu.nes.cpu.register_a, 0x05);
        assert_eq!(emu.nes.cpu.status & 0b0000_0010, 0);
        assert_eq!(emu.nes.cpu.status & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0, CPU::BRK]);
        assert_eq!(emu.nes.cpu.status & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0xff, CPU::BRK]);
        assert_eq!(emu.nes.cpu.status & 0b1000_0000, 0b1000_0000);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut emu = Emulator::new();
        emu.load(&vec![CPU::TAX, CPU::BRK]);
        emu.nes.cpu.register_a = 10;
        emu.run();
        assert_eq!(emu.nes.cpu.register_x, 10)
    }

    #[test]
    fn test_inx_overflow() {
        let mut emu = Emulator::new();
        emu.load(&vec![CPU::INX, CPU::INX, CPU::BRK]);
        emu.nes.cpu.register_x = 0xff;
        emu.run();
        assert_eq!(emu.nes.cpu.register_x, 1)
    }

    #[test]
    fn test_5_ops() {
        let mut emu = Emulator::new();
        emu.load_and_run(&vec![CPU::LDA_IM, 0xc0, CPU::TAX, CPU::INX, CPU::BRK]);
        assert_eq!(emu.nes.cpu.register_x, 0xc1)
    }

    #[test]
    fn test_program_simple() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa9, 0x01, 0x8d, 0x00, 0x02, 0xa9, 0x05, 0x8d, 0x01, 0x02,
            0xa9, 0x08, 0x8d, 0x02, 0x02, 0x00
        ];
        emu.load_and_run(&program);
        assert_eq!(emu.nes.cpu.register_a, 0x08);
        assert_eq!(emu.nes.mem.read_byte(0x0202), 0x08);
        assert_eq!(emu.nes.cpu.status, 0b0010_0100);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_adc() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa9, 0xc0, 0xaa, 0xe8, 0x69, 0xc4, 0x00
        ];
        emu.load_and_run(&program);
        assert_eq!(emu.nes.cpu.register_a, 0x84);
        assert_eq!(emu.nes.cpu.register_x, 0xc1);
        assert_eq!(emu.nes.cpu.status, 0b1010_0101);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_signed_division_by_four() {
        let mut emu = Emulator::new();
        let program = vec![
            CPU::LDA_IM, 0x88, CPU::CMP_IM, 0x80, CPU::ARR, 0xff, CPU::ROR, CPU::BRK
        ];
        emu.load_and_run(&program);
        assert_eq!(emu.nes.cpu.register_a, 0xe2);
        assert_eq!(emu.nes.cpu.register_x, 0x00);
        assert_eq!(emu.nes.cpu.register_y, 0x00);
        assert_eq!(emu.nes.cpu.status, 0b1110_0100);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_adc_carry_overflow() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa9, 0xff, 0x69, 0xff, 0xa9, 0x0f, 0x69, 0x70, 0x00
        ];
        emu.load_and_run(&program);
        assert_eq!(emu.nes.cpu.register_a, 0x80);
        assert_eq!(emu.nes.cpu.status, 0b1110_0100);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_branch() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa2, 0x08, 0xca, 0x8e, 0x00, 0x02, 0xe0, 0x03, 0xd0, 0xf8,
            0x8e, 0x01, 0x02, 0x00
        ];
        emu.load_and_run(&program);
        assert_eq!(emu.nes.cpu.register_x, 0x03);
        assert_eq!(emu.nes.cpu.status, 0b0010_0111);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_subroutines() {
        let mut emu = Emulator::new();
        let program = vec![
            0x20, 0x09, 0x06, 0x20, 0x0c, 0x06, 0x20, 0x12, 0x06, 0xa2,
            0x00, 0x60, 0xe8, 0xe0, 0x05, 0xd0, 0xfb, 0x60, 0x00
        ];
        emu.load_at_addr(0x600, &program);
        emu.run();
        assert_eq!(emu.nes.cpu.register_x, 0x05);
        assert_eq!(emu.nes.cpu.stack, 0xfb);
        assert_eq!(emu.nes.cpu.status, 0b0010_0111);
        assert_eq!(emu.nes.cpu.program_counter, 0x600 + program.len() as u16);
    }

    #[test]
    fn test_program_indexed_indirect_x() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa2, 0x01, 0xa9, 0x05, 0x85, 0x01, 0xa9, 0x07, 0x85, 0x02,
            0xa0, 0x0a, 0x8c, 0x05, 0x07, 0xa1, 0x00, 0x00
        ];
        emu.load_and_run(&program);
        assert_eq!(emu.nes.cpu.register_a, 0x0a);
        assert_eq!(emu.nes.cpu.register_x, 0x01);
        assert_eq!(emu.nes.cpu.register_y, 0x0a);
        assert_eq!(emu.nes.cpu.status, 0b0010_0100);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_indirect_indexed_y() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa0, 0x01, 0xa9, 0x03, 0x85, 0x01, 0xa9, 0x07, 0x85, 0x02,
            0xa2, 0x0a, 0x8e, 0x04, 0x07, 0xb1, 0x01, 0x00
        ];
        emu.load_and_run(&program);
        assert_eq!(emu.nes.cpu.register_a, 0x0a);
        assert_eq!(emu.nes.cpu.register_x, 0x0a);
        assert_eq!(emu.nes.cpu.register_y, 0x01);
        assert_eq!(emu.nes.cpu.status, 0b0010_0100);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_stack_operations() {
        let mut emu = Emulator::new();
        let program = vec![
            0xa2, 0x00, 0xa0, 0x00, 0x8a, 0x99, 0x00, 0x02, 0x48, 0xe8,
            0xc8, 0xc0, 0x10, 0xd0, 0xf5, 0x68, 0x99, 0x00, 0x02, 0xc8,
            0xc0, 0x20, 0xd0, 0xf7, 0x00
        ];
        emu.load_and_run(&program);
        for i in 0..16 {
            assert_eq!(emu.nes.mem.read_byte(0x200 + i), i as u8);
            assert_eq!(emu.nes.mem.read_byte(0x200 + (31 - i)), i as u8);
        }
        assert_eq!(emu.nes.cpu.register_a, 0x00);
        assert_eq!(emu.nes.cpu.register_x, 0x10);
        assert_eq!(emu.nes.cpu.register_y, 0x20);
        assert_eq!(emu.nes.cpu.stack, 0xfd);
        assert_eq!(emu.nes.cpu.status, 0b0010_0111);
        assert_eq!(emu.nes.cpu.program_counter, CPUMemory::PRG_ROM_START + program.len() as u16);
    }

    #[test]
    fn test_program_snake_game() {
        let mut emu = Emulator::new();
        let program = vec![
            0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20,
            0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85, 0x02, 0xa9, 0x04, 0x85,
            0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9,
            0x0f, 0x85, 0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85,
            0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe, 0x29, 0x03,
            0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20,
            0x8d, 0x06, 0x20, 0xc3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20,
            0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
            0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0,
            0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60, 0xa9, 0x04, 0x24, 0x02,
            0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24,
            0x02, 0xd0, 0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01,
            0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02, 0x60, 0xa9,
            0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60,
            0x60, 0x20, 0x94, 0x06, 0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00,
            0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
            0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02,
            0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06, 0xb5, 0x11, 0xc5, 0x11,
            0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa,
            0x06, 0x4c, 0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5,
            0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02, 0x4a, 0xb0,
            0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f,
            0xa5, 0x10, 0x38, 0xe9, 0x20, 0x85, 0x10, 0x90, 0x01, 0x60,
            0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
            0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10,
            0x18, 0x69, 0x20, 0x85, 0x10, 0xb0, 0x01, 0x60, 0xe6, 0x11,
            0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5,
            0x10, 0x29, 0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35,
            0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60, 0xa6, 0x03,
            0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10,
            0x60, 0xa2, 0x00, 0xea, 0xea, 0xca, 0xd0, 0xfb, 0x60, 0x00
        ];
        emu.load_at_addr(0x600, &program);
        emu.run();

        assert_eq!(emu.nes.cpu.register_a, 0x1f);
        assert_eq!(emu.nes.cpu.register_x, 0xff);
        assert_eq!(emu.nes.cpu.register_y, 0x00);
        assert_eq!(emu.nes.cpu.stack, 0xf9);
        assert_eq!(emu.nes.cpu.status, 0b0010_0111);
        assert_eq!(emu.nes.cpu.program_counter, 0x736);
    }
}