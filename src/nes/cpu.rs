pub mod mem;
mod registers;

use bitvec::prelude::*;
use rand::Rng;

use crate::nes::cpu::mem::Memory;

const ISB_PATTERN: u8 = 0b1110_0011;
const DCP_PATTERN: u8 = 0b1100_0011;
const LAX_PATTERN: u8 = 0b1010_0011;
const SAX_PATTERN: u8 = 0b1000_0011;
const RRA_PATTERN: u8 = 0b0110_0011;
const SRE_PATTERN: u8 = 0b0100_0011;
const RLA_PATTERN: u8 = 0b0010_0011;
const SLO_PATTERN: u8 = 0b0000_0011;

const INC_PATTERN: u8 = 0b1110_0010;
const DEC_PATTERN: u8 = 0b1100_0010;
const LDX_PATTERN: u8 = 0b1010_0010;
const STX_PATTERN: u8 = 0b1000_0010;
const ROR_PATTERN: u8 = 0b0110_0010;
const LSR_PATTERN: u8 = 0b0100_0010;
const ROL_PATTERN: u8 = 0b0010_0010;
const ASL_PATTERN: u8 = 0b0000_0010;

const SBC_PATTERN: u8 = 0b1110_0001;
const CMP_PATTERN: u8 = 0b1100_0001;
const LDA_PATTERN: u8 = 0b1010_0001;
const STA_PATTERN: u8 = 0b1000_0001;
const ADC_PATTERN: u8 = 0b0110_0001;
const EOR_PATTERN: u8 = 0b0100_0001;
const AND_PATTERN: u8 = 0b0010_0001;
const ORA_PATTERN: u8 = 0b0000_0001;

const CPX_PATTERN: u8 = 0b1110_0000;
const LDY_PATTERN: u8 = 0b1010_0000;
const CPY_PATTERN: u8 = 0b1100_0000;
const STY_PATTERN: u8 = 0b1000_0000;

const CARRY_FLAG: u8 = 0;
const ZERO_FLAG: u8 = 1;
const INTERRUPT_DISABLE: u8 = 2;
const DECIMAL_MODE_FLAG: u8 = 3;
const BREAK_COMMAND: u8 = 4;
const UNUSED_FLAG: u8 = 5;
const OVERFLOW_FLAG: u8 = 6;
const NEGATIVE_FLAG: u8 = 7;

const B_FLAG_MASK: u8 = 0b0011_0000;
const B_FLAG_SET_MASK: u8 = 0b0010_0000;
const B_FLAG_CLEAR_MASK: u8 = 0b1110_1111;

const OP_MASK: u8 = 0b1110_0011;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub stack: u8,
    pub status: u8, // todo: use StatusRegister struct instead
    pub program_counter: u16, // todo: use ProgramCounter struct instead
    pub memory: Memory
}

impl CPU {
    pub const LDA_IM: u8 = 0xa9;
    pub const LDA_ZP: u8 = 0xa5;
    pub const LDA_ZP_X: u8 = 0xb5;
    pub const LDA_AB: u8 = 0xad;
    pub const LDA_AB_X: u8 = 0xbd;
    pub const LDA_AB_Y: u8 = 0xb9;
    pub const LDA_IN_X: u8 = 0xa1;
    pub const LDA_IN_Y: u8 = 0xb1;
    pub const LDX_IM: u8 = 0xa2;
    pub const LDX_ZP: u8 = 0xa6;
    pub const LDX_ZP_Y: u8 = 0xb6;
    pub const LDX_AB: u8 = 0xae;
    pub const LDX_AB_Y: u8 = 0xbe;
    pub const LDY_IM: u8 = 0xa0;
    pub const LDY_ZP: u8 = 0xa4;
    pub const LDY_ZP_X: u8 = 0xb4;
    pub const LDY_AB: u8 = 0xac;
    pub const LDY_AB_X: u8 = 0xbc;

    pub const STA_ZP: u8 = 0x85;
    pub const STA_ZP_X: u8 = 0x95;
    pub const STA_AB: u8 = 0x8d;
    pub const STA_AB_X: u8 = 0x9d;
    pub const STA_AB_Y: u8 = 0x99;
    pub const STA_IN_X: u8 = 0x81;
    pub const STA_IN_Y: u8 = 0x91;
    pub const STX_ZP: u8 = 0x86;
    pub const STX_ZP_Y: u8 = 0x96;
    pub const STX_AB: u8 = 0x8e;
    pub const STY_ZP: u8 = 0x84;
    pub const STY_ZP_X: u8 = 0x94;
    pub const STY_AB: u8 = 0x8c;

    pub const TAX: u8 = 0xaa;
    pub const TAY: u8 = 0xa8;
    pub const TSX: u8 = 0xba;
    pub const TXA: u8 = 0x8a;
    pub const TXS: u8 = 0x9a;
    pub const TYA: u8 = 0x98;

    pub const ADC_IM: u8 = 0x69;
    pub const ADC_ZP: u8 = 0x65;
    pub const ADC_ZP_X: u8 = 0x75;
    pub const ADC_AB: u8 = 0x6d;
    pub const ADC_AB_X: u8 = 0x7d;
    pub const ADC_AB_Y: u8 = 0x79;
    pub const ADC_IN_X: u8 = 0x61;
    pub const ADC_IN_Y: u8 = 0x71;

    pub const SBC_IM: u8 = 0xe9;
    pub const SBC_ZP: u8 = 0xe5;
    pub const SBC_ZP_X: u8 = 0xf5;
    pub const SBC_AB: u8 = 0xed;
    pub const SBC_AB_X: u8 = 0xfd;
    pub const SBC_AB_Y: u8 = 0xf9;
    pub const SBC_IN_X: u8 = 0xe1;
    pub const SBC_IN_Y: u8 = 0xf1;

    pub const EOR_IM: u8 = 0x49;
    pub const EOR_ZP: u8 = 0x45;
    pub const EOR_ZP_X: u8 = 0x55;
    pub const EOR_AB: u8 = 0x4d;
    pub const EOR_AB_X: u8 = 0x5d;
    pub const EOR_AB_Y: u8 = 0x59;
    pub const EOR_IN_X: u8 = 0x41;
    pub const EOR_IN_Y: u8 = 0x51;

    pub const AND_IM: u8 = 0x29;
    pub const AND_ZP: u8 = 0x25;
    pub const AND_ZP_X: u8 = 0x35;
    pub const AND_AB: u8 = 0x2d;
    pub const AND_AB_X: u8 = 0x3d;
    pub const AND_AB_Y: u8 = 0x39;
    pub const AND_IN_X: u8 = 0x21;
    pub const AND_IN_Y: u8 = 0x31;

    pub const ORA_IM: u8 = 0x09;
    pub const ORA_ZP: u8 = 0x05;
    pub const ORA_ZP_X: u8 = 0x15;
    pub const ORA_AB: u8 = 0x0d;
    pub const ORA_AB_X: u8 = 0x1d;
    pub const ORA_AB_Y: u8 = 0x19;
    pub const ORA_IN_X: u8 = 0x01;
    pub const ORA_IN_Y: u8 = 0x11;

    pub const LSR: u8 = 0x4a;
    pub const LSR_ZP: u8 = 0x46;
    pub const LSR_ZP_X: u8 = 0x56;
    pub const LSR_AB: u8 = 0x4e;
    pub const LSR_AB_X: u8 = 0x5e;
    pub const ASL: u8 = 0x0a;
    pub const ASL_ZP: u8 = 0x06;
    pub const ASL_ZP_X: u8 = 0x16;
    pub const ASL_AB: u8 = 0x0e;
    pub const ASL_AB_X: u8 = 0x1e;

    pub const ROR: u8 = 0x6a;
    pub const ROR_ZP: u8 = 0x66;
    pub const ROR_ZP_X: u8 = 0x76;
    pub const ROR_AB: u8 = 0x6e;
    pub const ROR_AB_X: u8 = 0x7e;
    pub const ROL: u8 = 0x2a;
    pub const ROL_ZP: u8 = 0x26;
    pub const ROL_ZP_X: u8 = 0x36;
    pub const ROL_AB: u8 = 0x2e;
    pub const ROL_AB_X: u8 = 0x3e;

    pub const INC_ZP: u8 = 0xe6;
    pub const INC_ZP_X: u8 = 0xf6;
    pub const INC_AB: u8 = 0xee;
    pub const INC_AB_X: u8 = 0xfe;
    pub const INX: u8 = 0xe8;
    pub const INY: u8 = 0xc8;

    pub const DEC_ZP: u8 = 0xc6;
    pub const DEC_ZP_X: u8 = 0xd6;
    pub const DEC_AB: u8 = 0xce;
    pub const DEC_AB_X: u8 = 0xde;
    pub const DEX: u8 = 0xca;
    pub const DEY: u8 = 0x88;

    pub const CMP_IM: u8 = 0xc9;
    pub const CMP_ZP: u8 = 0xc5;
    pub const CMP_ZP_X: u8 = 0xd5;
    pub const CMP_AB: u8 = 0xcd;
    pub const CMP_AB_X: u8 = 0xdd;
    pub const CMP_AB_Y: u8 = 0xd9;
    pub const CMP_IN_X: u8 = 0xc1;
    pub const CMP_IN_Y: u8 = 0xd1;
    pub const CPX_IM: u8 = 0xe0;
    pub const CPX_ZP: u8 = 0xe4;
    pub const CPX_AB: u8 = 0xec;
    pub const CPY_IM: u8 = 0xc0;
    pub const CPY_ZP: u8 = 0xc4;
    pub const CPY_AB: u8 = 0xcc;

    pub const SEC: u8 = 0x38;
    pub const CLC: u8 = 0x18;
    pub const SED: u8 = 0xf8;
    pub const CLD: u8 = 0xd8;
    pub const SEI: u8 = 0x78;
    pub const CLI: u8 = 0x58;
    pub const CLV: u8 = 0xb8;

    pub const JMP_AB: u8 = 0x4c;
    pub const JMP_IN: u8 = 0x6c;
    pub const JSR: u8 = 0x20;
    pub const RTS: u8 = 0x60;
    pub const RTI: u8 = 0x40;
    pub const BEQ: u8 = 0xf0;
    pub const BNE: u8 = 0xd0;
    pub const BCC: u8 = 0x90;
    pub const BCS: u8 = 0xb0;
    pub const BMI: u8 = 0x30;
    pub const BPL: u8 = 0x10;
    pub const BVC: u8 = 0x50;
    pub const BVS: u8 = 0x70;

    pub const PHA: u8 = 0x48;
    pub const PHP: u8 = 0x08;
    pub const PLA: u8 = 0x68;
    pub const PLP: u8 = 0x28;

    pub const BIT_ZP: u8 = 0x24;
    pub const BIT_AB: u8 = 0x2c;

    pub const NOP: u8 = 0xea;
    pub const BRK: u8 = 0x00;

    // undocumented opcodes
    pub const NOP_1: u8 = 0x1a;
    pub const NOP_2: u8 = 0x3a;
    pub const NOP_3: u8 = 0x5a;
    pub const NOP_4: u8 = 0x7a;
    pub const NOP_5: u8 = 0xda;
    pub const NOP_6: u8 = 0xfa;

    pub const DOP_IM_1: u8 = 0x80;
    pub const DOP_IM_2: u8 = 0x82;
    pub const DOP_IM_3: u8 = 0x89;
    pub const DOP_IM_4: u8 = 0xc2;
    pub const DOP_IM_5: u8 = 0xe2;
    pub const DOP_ZP_1: u8 = 0x04;
    pub const DOP_ZP_2: u8 = 0x44;
    pub const DOP_ZP_3: u8 = 0x64;
    pub const DOP_ZP_X_1: u8 = 0x14;
    pub const DOP_ZP_X_2: u8 = 0x34;
    pub const DOP_ZP_X_3: u8 = 0x54;
    pub const DOP_ZP_X_4: u8 = 0x74;
    pub const DOP_ZP_X_5: u8 = 0xd4;
    pub const DOP_ZP_X_6: u8 = 0xf4;

    pub const TOP_AB: u8 = 0x0c;
    pub const TOP_AB_X_1: u8 = 0x1c;
    pub const TOP_AB_X_2: u8 = 0x3c;
    pub const TOP_AB_X_3: u8 = 0x5c;
    pub const TOP_AB_X_4: u8 = 0x7c;
    pub const TOP_AB_X_5: u8 = 0xdc;
    pub const TOP_AB_X_6: u8 = 0xfc;

    pub const JAM_1: u8 = 0x02;
    pub const JAM_2: u8 = 0x12;
    pub const JAM_3: u8 = 0x22;
    pub const JAM_4: u8 = 0x32;
    pub const JAM_5: u8 = 0x42;
    pub const JAM_6: u8 = 0x52;
    pub const JAM_7: u8 = 0x62;
    pub const JAM_8: u8 = 0x72;
    pub const JAM_9: u8 = 0x92;
    pub const JAM_10: u8 = 0xb2;
    pub const JAM_11: u8 = 0xd2;
    pub const JAM_12: u8 = 0xf2;

    pub const LAX_ZP: u8 = 0xa7;
    pub const LAX_ZP_Y: u8 = 0xb7;
    pub const LAX_AB: u8 = 0xaf;
    pub const LAX_AB_Y: u8 = 0xbf;
    pub const LAX_IN_X: u8 = 0xa3;
    pub const LAX_IN_Y: u8 = 0xb3;

    pub const SAX_ZP: u8 = 0x87;
    pub const SAX_ZP_Y: u8 = 0x97;
    pub const SAX_AB: u8 = 0x8f;
    pub const SAX_IN_X: u8 = 0x83;

    pub const DCP_ZP: u8 = 0xc7;
    pub const DCP_ZP_X: u8 = 0xd7;
    pub const DCP_AB: u8 = 0xcf;
    pub const DCP_AB_X: u8 = 0xdf;
    pub const DCP_AB_Y: u8 = 0xdb;
    pub const DCP_IN_X: u8 = 0xc3;
    pub const DCP_IN_Y: u8 = 0xd3;

    pub const ISB_ZP: u8 = 0xe7;
    pub const ISB_ZP_X: u8 = 0xf7;
    pub const ISB_AB: u8 = 0xef;
    pub const ISB_AB_X: u8 = 0xff;
    pub const ISB_AB_Y: u8 = 0xfb;
    pub const ISB_IN_X: u8 = 0xe3;
    pub const ISB_IN_Y: u8 = 0xf3;

    pub const SLO_ZP: u8 = 0x07;
    pub const SLO_ZP_X: u8 = 0x17;
    pub const SLO_AB: u8 = 0x0F;
    pub const SLO_AB_X: u8 = 0x1F;
    pub const SLO_AB_Y: u8 = 0x1B;
    pub const SLO_IN_X: u8 = 0x03;
    pub const SLO_IN_Y: u8 = 0x13;

    pub const RLA_ZP: u8 = 0x27;
    pub const RLA_ZP_X: u8 = 0x37;
    pub const RLA_AB: u8 = 0x2F;
    pub const RLA_AB_X: u8 = 0x3F;
    pub const RLA_AB_Y: u8 = 0x3B;
    pub const RLA_IN_X: u8 = 0x23;
    pub const RLA_IN_Y: u8 = 0x33;

    pub const SRE_ZP: u8 = 0x47;
    pub const SRE_ZP_X: u8 = 0x57;
    pub const SRE_AB: u8 = 0x4F;
    pub const SRE_AB_X: u8 = 0x5F;
    pub const SRE_AB_Y: u8 = 0x5B;
    pub const SRE_IN_X: u8 = 0x43;
    pub const SRE_IN_Y: u8 = 0x53;

    pub const RRA_ZP: u8 = 0x67;
    pub const RRA_ZP_X: u8 = 0x77;
    pub const RRA_AB: u8 = 0x6F;
    pub const RRA_AB_X: u8 = 0x7F;
    pub const RRA_AB_Y: u8 = 0x7B;
    pub const RRA_IN_X: u8 = 0x63;
    pub const RRA_IN_Y: u8 = 0x73;

    pub const ANC_1: u8 = 0x0b;
    pub const ANC_2: u8 = 0x2b;
    pub const SHA_AB_Y: u8 = 0x9f;
    pub const SHA_IN_Y: u8 = 0x93;
    pub const SHX: u8 = 0x9e;
    pub const SHY: u8 = 0x9c;
    pub const SHS: u8 = 0x9b;
    pub const ALR: u8 = 0x4b;
    pub const ARR: u8 = 0x6b;
    pub const ANE: u8 = 0x8b;
    pub const LXA: u8 = 0xab;
    pub const SBX: u8 = 0xcb;
    pub const LAS: u8 = 0xbb;
    pub const SBC_IM_U: u8 = 0xeb;

    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stack: 0xff,
            status: 0b0011_0000,
            program_counter: 0,
            memory: Memory::new(),
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack = 0xfd;
        self.status = 0b0010_0100;
        self.program_counter = 0;
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        let opcode = self.memory.read_byte(self.program_counter);
        match opcode {
            CPU::TAX => self.tax(),
            CPU::TAY => self.tay(),
            CPU::TSX => self.tsx(),
            CPU::TXA => self.txa(),
            CPU::TXS => self.txs(),
            CPU::TYA => self.tya(),
            CPU::INX => self.inx(),
            CPU::INY => self.iny(),
            CPU::DEX => self.dex(),
            CPU::DEY => self.dey(),
            CPU::SEC => self.sec(),
            CPU::CLC => self.clc(),
            CPU::SED => self.sed(),
            CPU::CLD => self.cld(),
            CPU::SEI => self.sei(),
            CPU::CLI => self.cli(),
            CPU::CLV => self.clv(),
            CPU::PHA => self.pha(),
            CPU::PLA => self.pla(),
            CPU::PHP => self.php(),
            CPU::PLP => self.plp(),
            CPU::RTS => self.rts(),
            CPU::RTI => self.rti(),
            CPU::NOP => self.nop(),
            CPU::BIT_ZP => {
                let address = self.fetch_param();
                self.bit_zp(address)
            },
            CPU::BIT_AB => {
                let address = self.fetch_addr_param();
                self.bit_ab(address)
            },
            CPU::JMP_AB => {
                let address = self.fetch_addr_param();
                self.jmp_ab(address);
            },
            CPU::JMP_IN => {
                let address = self.fetch_addr_param();
                self.jmp_in(address);
            },
            CPU::JSR => {
                let address = self.fetch_addr_param();
                self.jsr(address);
            },
            CPU::BEQ => {
                let offset = self.fetch_param();
                self.beq(offset as i8);
            }
            CPU::BNE => {
                let offset = self.fetch_param();
                self.bne(offset as i8);
            },
            CPU::BCC => {
                let offset = self.fetch_param();
                self.bcc(offset as i8);
            },
            CPU::BCS => {
                let offset = self.fetch_param();
                self.bcs(offset as i8);
            },
            CPU::BMI => {
                let offset = self.fetch_param();
                self.bmi(offset as i8);
            },
            CPU::BPL => {
                let offset = self.fetch_param();
                self.bpl(offset as i8);
            },
            CPU::BVS => {
                let offset = self.fetch_param();
                self.bvs(offset as i8);
            },
            CPU::BVC => {
                let offset = self.fetch_param();
                self.bvc(offset as i8);
            },
            CPU::BRK => {
                // todo: this implementation of BRK is not correct (lol)
                self.increment_program_counter();
                return Err(false);
            },
            // undocumented opcodes
            CPU::SBC_IM_U => self.sbc(CPU::SBC_IM),
            CPU::TOP_AB => self.top_ab(),
            CPU::ARR => {
                let immediate = self.fetch_param();
                self.arr(immediate);
            },
            CPU::ALR => {
                let immediate = self.fetch_param();
                self.alr(immediate);
            },
            CPU::LXA => {
                let immediate = self.fetch_param();
                self.lxa(immediate);
            },
            CPU::SBX => {
                let immediate = self.fetch_param();
                self.sbx(immediate);
            },
            CPU::LAS => {
                let address = self.fetch_addr_param();
                self.las(address);
            },
            CPU::ANE => {
                let immediate = self.fetch_param();
                self.ane(immediate);
            },
            CPU::SHA_AB_Y => {
                let address = self.fetch_addr_param();
                self.sha_ab_y(address);
            },
            CPU::SHA_IN_Y => {
                let address = self.fetch_param();
                self.sha_in_y(address);
            },
            CPU::SHX => {
                let address = self.fetch_addr_param();
                self.shx(address);
            }
            CPU::SHY => {
                let address = self.fetch_addr_param();
                self.shy(address);
            }
            CPU::SHS => {
                let address = self.fetch_addr_param();
                self.shs(address);
            }
            CPU::ANC_1 | CPU::ANC_2 => {
                let immediate = self.fetch_param();
                self.anc(immediate);
            },
            CPU::TOP_AB_X_1 | CPU::TOP_AB_X_2 | CPU::TOP_AB_X_3 |
            CPU::TOP_AB_X_4 | CPU::TOP_AB_X_5 | CPU::TOP_AB_X_6 => {
                self.top_ab_x();
            },
            CPU::DOP_IM_1 | CPU::DOP_IM_2 | CPU::DOP_IM_3 |
            CPU::DOP_IM_4 | CPU::DOP_IM_5 => {
                self.dop_im();
            },
            CPU::DOP_ZP_1 | CPU::DOP_ZP_2 | CPU::DOP_ZP_3 => {
                self.dop_zp();
            },
            CPU::DOP_ZP_X_1 | CPU::DOP_ZP_X_2 | CPU::DOP_ZP_X_3 |
            CPU::DOP_ZP_X_4 | CPU::DOP_ZP_X_5 | CPU::DOP_ZP_X_6 => {
                self.dop_zp_x();
            },
            CPU::NOP_1 | CPU::NOP_2 | CPU::NOP_3 |
            CPU::NOP_4 | CPU::NOP_5 | CPU::NOP_6 => {
                self.nop()
            },
            CPU::JAM_1 | CPU::JAM_2 | CPU::JAM_3 | CPU::JAM_4 |
            CPU::JAM_5 | CPU::JAM_6 | CPU::JAM_7 | CPU::JAM_8 |
            CPU::JAM_9 | CPU::JAM_10 | CPU::JAM_11 | CPU::JAM_12 => {
                self.jam();
            },
            _ => match opcode & OP_MASK {
                ISB_PATTERN => self.isb(opcode),
                DCP_PATTERN => self.dcp(opcode),
                LAX_PATTERN => self.lax(opcode),
                SAX_PATTERN => self.sax(opcode),
                RRA_PATTERN => self.rra(opcode),
                SRE_PATTERN => self.sre(opcode),
                RLA_PATTERN => self.rla(opcode),
                SLO_PATTERN => self.slo(opcode),
                INC_PATTERN => self.inc(opcode),
                DEC_PATTERN => self.dec(opcode),
                LDX_PATTERN => self.ldx(opcode),
                STX_PATTERN => self.stx(opcode),
                ROR_PATTERN => self.ror(opcode),
                LSR_PATTERN => self.lsr(opcode),
                ROL_PATTERN => self.rol(opcode),
                ASL_PATTERN => self.asl(opcode),
                SBC_PATTERN => self.sbc(opcode),
                CMP_PATTERN => self._cmp(opcode),
                LDA_PATTERN => self.lda(opcode),
                STA_PATTERN => self.sta(opcode),
                ADC_PATTERN => self.adc(opcode),
                EOR_PATTERN => self.eor(opcode),
                AND_PATTERN => self.and(opcode),
                ORA_PATTERN => self.ora(opcode),
                CPX_PATTERN => self.cpx(opcode),
                LDY_PATTERN => self.ldy(opcode),
                CPY_PATTERN => self.cpy(opcode),
                STY_PATTERN => self.sty(opcode),
                _ =>  panic!("invalid opcode: {:x}", opcode)
            }
        }
        return Ok(true);
    }

    #[inline]
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flag(self.register_x);
        /* todo: increment_program_counter should probably be moved outside of opcodes
            implementations, so as to allow for reuse in other opcode implementations */
        self.increment_program_counter();
    }

    #[inline]
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flag(self.register_y);
        self.increment_program_counter();
    }

    #[inline]
    fn tsx(&mut self) {
        self.register_x = self.stack;
        self.update_zero_and_negative_flag(self.register_x);
        self.increment_program_counter();
    }

    #[inline]
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flag(self.register_a);
        self.increment_program_counter();
    }

    #[inline]
    fn txs(&mut self) {
        self.stack = self.register_x;
        self.increment_program_counter();
    }

    #[inline]
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flag(self.register_a);
        self.increment_program_counter();
    }

    #[inline]
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flag(self.register_x);
        self.increment_program_counter();
    }

    #[inline]
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flag(self.register_y);
        self.increment_program_counter();
    }

    #[inline]
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flag(self.register_x);
        self.increment_program_counter();
    }

    #[inline]
    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flag(self.register_y);
        self.increment_program_counter();
    }
    
    #[inline]
    fn sec(&mut self) {
        self.set_status_flag(CARRY_FLAG);
        self.increment_program_counter();
    }

    #[inline]
    fn clc(&mut self) {
        self.clear_status_flag(CARRY_FLAG);
        self.increment_program_counter();
    }

    #[inline]
    fn sed(&mut self) {
        self.set_status_flag(DECIMAL_MODE_FLAG);
        self.increment_program_counter();
    }

    #[inline]
    fn cld(&mut self) {
        self.clear_status_flag(DECIMAL_MODE_FLAG);
        self.increment_program_counter();
    }

    #[inline]
    fn sei(&mut self) {
        self.set_status_flag(INTERRUPT_DISABLE);
        self.increment_program_counter();
    }

    #[inline]
    fn cli(&mut self) {
        self.clear_status_flag(INTERRUPT_DISABLE);
        self.increment_program_counter();
    }

    #[inline]
    fn clv(&mut self) {
        self.clear_status_flag(OVERFLOW_FLAG);
        self.increment_program_counter();
    }

    #[inline]
    fn pha(&mut self) {
        self.push_byte(self.register_a);
        self.increment_program_counter();
    }

    #[inline]
    fn pla(&mut self) {
        self.register_a = self.pop_byte();
        self.update_zero_and_negative_flag(self.register_a);
        self.increment_program_counter();
    }

    #[inline]
    fn php(&mut self) {
        self.push_byte(self.status | B_FLAG_MASK);
        self.increment_program_counter();
    }

    #[inline]
    fn plp(&mut self) {
        self.status = self.pop_byte();
        self.status = self.status | B_FLAG_SET_MASK;
        self.status = self.status & B_FLAG_CLEAR_MASK;
        self.increment_program_counter();
    }

    #[inline]
    fn bit_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.update_bit_flags(value);
        self.increment_program_counter();
    }

    #[inline]
    fn bit_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.update_bit_flags(value);
        self.increment_program_counter();
    }

    #[inline]
    fn jmp_ab(&mut self, address: u16) {
        self.program_counter = address;
    }

    #[inline]
    fn jmp_in(&mut self, address: u16) {
        let addr = self.memory.read_addr_in(address);
        self.program_counter = addr;
    }

    #[inline]
    fn jsr(&mut self, address: u16) {
        self.push_addr(self.program_counter);
        self.jmp_ab(address);
    }

    #[inline]
    fn rts(&mut self) {
        self.program_counter = self.pop_addr();
        self.increment_program_counter();
    }

    #[inline]
    fn rti(&mut self) {
        self.plp();
        self.program_counter = self.pop_addr();
    }

    #[inline]
    fn beq(&mut self, offset: i8) {
        self.increment_program_counter();
        if self.get_status_flag(ZERO_FLAG) {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    #[inline]
    fn bne(&mut self, offset: i8) {
        self.increment_program_counter();
        if !self.get_status_flag(ZERO_FLAG) {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    #[inline]
    fn bcs(&mut self, offset: i8) {
        self.increment_program_counter();
        if self.get_status_flag(CARRY_FLAG) {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    #[inline]
    fn bcc(&mut self, offset: i8) {
        self.increment_program_counter();
        if !self.get_status_flag(CARRY_FLAG) {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    #[inline]
    fn bmi(&mut self, offset: i8) {
        self.increment_program_counter();
        if self.get_status_flag(NEGATIVE_FLAG) {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    #[inline]
    fn bpl(&mut self, offset: i8) {
        self.increment_program_counter();
        if !self.get_status_flag(NEGATIVE_FLAG) {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    #[inline]
    fn bvs(&mut self, offset: i8) {
        self.increment_program_counter();
        if self.get_status_flag(OVERFLOW_FLAG) {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    #[inline]
    fn bvc(&mut self, offset: i8) {
        self.increment_program_counter();
        if !self.get_status_flag(OVERFLOW_FLAG) {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    #[inline]
    fn arr(&mut self, immediate: u8) {
        self.and_im(immediate);
        self.ror_a();
        let bit_6 = (self.register_a & 0x40 > 0) as u8;
        let bit_5 = (self.register_a & 0x20 > 0) as u8;
        self.update_status_flag(CARRY_FLAG, bit_6 > 0);
        self.update_status_flag(OVERFLOW_FLAG, bit_6 ^ bit_5 > 0);
        self.increment_program_counter();
    }

    #[inline]
    fn alr(&mut self, immediate: u8) {
        self.and_im(immediate);
        self.lsr_a();
        self.increment_program_counter();
    }

    #[inline]
    fn lxa(&mut self, immediate: u8) {
        self.and_im(immediate);
        self.tax();
        self.increment_program_counter();
    }

    #[inline]
    fn sbx(&mut self, immediate: u8) {
        self.register_x = self.register_x & self.register_a;
        let sum = (self.register_x as u16).wrapping_add(immediate.wrapping_neg() as u16);
        self.register_x = sum as u8;
        self.update_status_flag(CARRY_FLAG, sum > 0xff);
        self.update_zero_and_negative_flag(self.register_x);
        self.increment_program_counter();
    }

    #[inline]
    fn las(&mut self, address: u16) {
        let result = self.memory.ab_y_read(address, self.register_y) & self.stack;
        self.register_a = result;
        self.register_x = result;
        self.stack = result;
        self.update_zero_and_negative_flag(result);
        self.increment_program_counter();
    }

    #[inline]
    fn ane(&mut self, immediate: u8) {
        let magic_digit = rand::thread_rng().gen_range(0..0xf) as u8;
        let magic = (magic_digit << 4) | magic_digit;
        self.register_a = (self.register_a | magic) & self.register_x & immediate;
        self.update_zero_and_negative_flag(self.register_a);
        self.increment_program_counter();
    }

    #[inline]
    fn sha_ab_y(&mut self, address: u16) {
        let high_byte = ((address & 0xff00) >> 8) as u8;
        let result = self.register_x & self.register_a & high_byte.wrapping_add(1);
        self.memory.ab_y_write(address, self.register_y, result);
        self.increment_program_counter();
    }

    #[inline]
    fn sha_in_y(&mut self, address: u8) {
        let result = self.register_x & self.register_a & address.wrapping_add(1);
        self.memory.in_y_write(address, self.register_y, result);
        self.increment_program_counter();
    }

    #[inline]
    fn shx(&mut self, address: u16) {
        let high_byte = ((address & 0xff00) >> 8) as u8;
        let result = self.register_x & high_byte.wrapping_add(1);
        self.memory.ab_y_write(address, self.register_y, result);
        self.increment_program_counter();
    }

    #[inline]
    fn shy(&mut self, address: u16) {
        let high_byte = ((address & 0xff00) >> 8) as u8;
        let result = self.register_y & high_byte.wrapping_add(1);
        self.memory.ab_x_write(address, self.register_x, result);
        self.increment_program_counter();
    }

    #[inline]
    fn shs(&mut self, address: u16) {
        let high_byte = ((address & 0xff00) >> 8) as u8;
        self.stack = self.register_x & self.register_a;
        self.memory.ab_y_write(address, self.register_y, self.stack & high_byte.wrapping_add(1));
        self.increment_program_counter();
    }

    #[inline]
    fn anc(&mut self, immediate: u8) {
        self.and_im(immediate);
        self.update_status_flag(CARRY_FLAG, self.register_a & 0x80 > 0);
        self.increment_program_counter();
    }

    #[inline]
    fn nop(&mut self) {
        self.increment_program_counter();
    }

    #[inline]
    fn dop(&mut self) {
        self.nop();
        self.nop();
    }

    #[inline]
    fn dop_im(&mut self) {
        self.dop();
    }

    #[inline]
    fn dop_zp(&mut self) {
        self.dop();
    }

    #[inline]
    fn dop_zp_x(&mut self) {
        self.dop();
    }

    #[inline]
    fn top(&mut self) {
        self.dop();
        self.nop();
    }

    #[inline]
    fn top_ab(&mut self) {
        self.top();
    }

    #[inline]
    fn top_ab_x(&mut self) {
        self.top();
    }

    #[inline]
    fn jam(&self) {
        // do nothing
    }

    /* todo: other commands with IMMEDIATE addressing support can have their opcodes defined in
        terms of their "_IM" variant
     */
    fn adc(&mut self, opcode: u8) {
        match opcode {
            CPU::ADC_IM => {
                let immediate = self.fetch_param();
                self.adc_im(immediate);
            },
            CPU::ADC_ZP => {
                let address = self.fetch_param();
                self.adc_zp(address);
            },
            CPU::ADC_ZP_X => {
                let address = self.fetch_param();
                self.adc_zp_x(address);
            },
            CPU::ADC_AB => {
                let address = self.fetch_addr_param();
                self.adc_ab(address);
            },
            CPU::ADC_AB_X => {
                let address = self.fetch_addr_param();
                self.adc_ab_x(address);
            },
            CPU::ADC_AB_Y => {
                let address = self.fetch_addr_param();
                self.adc_ab_y(address);
            },
            CPU::ADC_IN_X => {
                let address = self.fetch_param();
                self.adc_in_x(address);
            },
            CPU::ADC_IN_Y => {
                let address = self.fetch_param();
                self.adc_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn adc_im(&mut self, immediate: u8) {
        let mut sum = (self.register_a as u16).wrapping_add(immediate as u16);
        let mut overflow = (self.register_a ^ (sum as u8)) & (immediate ^ (sum as u8)) & 0x80 != 0;
        if self.get_status_flag(CARRY_FLAG) {
            let carry_sum = sum.wrapping_add(1);
            overflow = overflow || ((sum as u8) ^ (carry_sum as u8)) & (carry_sum as u8) & 0x80 != 0;
            sum = carry_sum;
        }
        self.register_a = sum as u8;
        self.update_status_flag(OVERFLOW_FLAG, overflow);
        self.update_status_flag(CARRY_FLAG, sum > 0xff);
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn adc_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.adc_im(value);
    }

    #[inline]
    fn adc_zp_x(&mut self, address: u8) {
        let value = self.memory.zp_x_read(address, self.register_x);
        self.adc_im(value);
    }

    #[inline]
    fn adc_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.adc_im(value);
    }

    #[inline]
    fn adc_ab_x(&mut self, address: u16) {
        let value = self.memory.ab_x_read(address, self.register_x);
        self.adc_im(value);
    }

    #[inline]
    fn adc_ab_y(&mut self, address: u16) {
        let value = self.memory.ab_y_read(address, self.register_y);
        self.adc_im(value);
    }

    #[inline]
    fn adc_in_x(&mut self, address: u8) {
        let value = self.memory.in_x_read(address, self.register_x);
        self.adc_im(value);
    }

    #[inline]
    fn adc_in_y(&mut self, address: u8) {
        let value = self.memory.in_y_read(address, self.register_y);
        self.adc_im(value);
    }

    fn sbc(&mut self, opcode: u8) {
        match opcode {
            CPU::SBC_IM => {
                let immediate = self.fetch_param();
                self.sbc_im(immediate);
            },
            CPU::SBC_ZP => {
                let address = self.fetch_param();
                self.sbc_zp(address);
            },
            CPU::SBC_ZP_X => {
                let address = self.fetch_param();
                self.sbc_zp_x(address);
            },
            CPU::SBC_AB => {
                let address = self.fetch_addr_param();
                self.sbc_ab(address);
            },
            CPU::SBC_AB_X => {
                let address = self.fetch_addr_param();
                self.sbc_ab_x(address);
            },
            CPU::SBC_AB_Y => {
                let address = self.fetch_addr_param();
                self.sbc_ab_y(address);
            },
            CPU::SBC_IN_X => {
                let address = self.fetch_param();
                self.sbc_in_x(address);
            },
            CPU::SBC_IN_Y => {
                let address = self.fetch_param();
                self.sbc_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn sbc_im(&mut self, immediate: u8) {
        self.adc_im(!immediate);
    }

    #[inline]
    fn sbc_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.sbc_im(value);
    }

    #[inline]
    fn sbc_zp_x(&mut self, address: u8) {
        let value = self.memory.zp_x_read(address, self.register_x);
        self.sbc_im(value);
    }

    #[inline]
    fn sbc_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.sbc_im(value);
    }

    #[inline]
    fn sbc_ab_x(&mut self, address: u16) {
        let value = self.memory.ab_x_read(address, self.register_x);
        self.sbc_im(value);
    }

    #[inline]
    fn sbc_ab_y(&mut self, address: u16) {
        let value = self.memory.ab_y_read(address, self.register_y);
        self.sbc_im(value);
    }

    #[inline]
    fn sbc_in_x(&mut self, address: u8) {
        let value = self.memory.in_x_read(address, self.register_x);
        self.sbc_im(value);
    }

    #[inline]
    fn sbc_in_y(&mut self, address: u8) {
        let value = self.memory.in_y_read(address, self.register_y);
        self.sbc_im(value);
    }

    fn eor(&mut self, opcode: u8) {
        match opcode {
            CPU::EOR_IM => {
                let immediate = self.fetch_param();
                self.eor_im(immediate);
            },
            CPU::EOR_ZP => {
                let address = self.fetch_param();
                self.eor_zp(address);
            },
            CPU::EOR_ZP_X => {
                let address = self.fetch_param();
                self.eor_zp_x(address);
            },
            CPU::EOR_AB => {
                let address = self.fetch_addr_param();
                self.eor_ab(address);
            },
            CPU::EOR_AB_X => {
                let address = self.fetch_addr_param();
                self.eor_ab_x(address);
            },
            CPU::EOR_AB_Y => {
                let address = self.fetch_addr_param();
                self.eor_ab_y(address);
            },
            CPU::EOR_IN_X => {
                let address = self.fetch_param();
                self.eor_in_x(address);
            },
            CPU::EOR_IN_Y => {
                let address = self.fetch_param();
                self.eor_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn eor_im(&mut self, immediate: u8) {
        self.register_a = self.register_a ^ immediate;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn eor_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.eor_im(value);
    }

    #[inline]
    fn eor_zp_x(&mut self, address: u8) {
        let value = self.memory.zp_x_read(address, self.register_x);
        self.eor_im(value);
    }

    #[inline]
    fn eor_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.eor_im(value);
    }

    #[inline]
    fn eor_ab_x(&mut self, address: u16) {
        let value = self.memory.ab_x_read(address, self.register_x);
        self.eor_im(value);
    }

    #[inline]
    fn eor_ab_y(&mut self, address: u16) {
        let value = self.memory.ab_y_read(address, self.register_y);
        self.eor_im(value);
    }

    #[inline]
    fn eor_in_x(&mut self, address: u8) {
        let value = self.memory.in_x_read(address, self.register_x);
        self.eor_im(value);
    }

    #[inline]
    fn eor_in_y(&mut self, address: u8) {
        let value = self.memory.in_y_read(address, self.register_y);
        self.eor_im(value);
    }

    fn and(&mut self, opcode: u8) {
        match opcode {
            CPU::AND_IM => {
                let immediate = self.fetch_param();
                self.and_im(immediate);
            },
            CPU::AND_ZP => {
                let address = self.fetch_param();
                self.and_zp(address);
            },
            CPU::AND_ZP_X => {
                let address = self.fetch_param();
                self.and_zp_x(address);
            },
            CPU::AND_AB => {
                let address = self.fetch_addr_param();
                self.and_ab(address);
            },
            CPU::AND_AB_X => {
                let address = self.fetch_addr_param();
                self.and_ab_x(address);
            },
            CPU::AND_AB_Y => {
                let address = self.fetch_addr_param();
                self.and_ab_y(address);
            },
            CPU::AND_IN_X => {
                let address = self.fetch_param();
                self.and_in_x(address);
            },
            CPU::AND_IN_Y => {
                let address = self.fetch_param();
                self.and_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn and_im(&mut self, immediate: u8) {
        self.register_a = self.register_a & immediate;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn and_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.and_im(value);
    }

    #[inline]
    fn and_zp_x(&mut self, address: u8) {
        let value = self.memory.zp_x_read(address, self.register_x);
        self.and_im(value);
    }

    #[inline]
    fn and_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.and_im(value);
    }

    #[inline]
    fn and_ab_x(&mut self, address: u16) {
        let value = self.memory.ab_x_read(address, self.register_x);
        self.and_im(value);
    }

    #[inline]
    fn and_ab_y(&mut self, address: u16) {
        let value = self.memory.ab_y_read(address, self.register_y);
        self.and_im(value);
    }

    #[inline]
    fn and_in_x(&mut self, address: u8) {
        let value = self.memory.in_x_read(address, self.register_x);
        self.and_im(value);
    }

    #[inline]
    fn and_in_y(&mut self, address: u8) {
        let value = self.memory.in_y_read(address, self.register_y);
        self.and_im(value);
    }

    fn ora(&mut self, opcode: u8) {
        match opcode {
            CPU::ORA_IM => {
                let immediate = self.fetch_param();
                self.ora_im(immediate);
            },
            CPU::ORA_ZP => {
                let address = self.fetch_param();
                self.ora_zp(address);
            },
            CPU::ORA_ZP_X => {
                let address = self.fetch_param();
                self.ora_zp_x(address);
            },
            CPU::ORA_AB => {
                let address = self.fetch_addr_param();
                self.ora_ab(address);
            },
            CPU::ORA_AB_X => {
                let address = self.fetch_addr_param();
                self.ora_ab_x(address);
            },
            CPU::ORA_AB_Y => {
                let address = self.fetch_addr_param();
                self.ora_ab_y(address);
            },
            CPU::ORA_IN_X => {
                let address = self.fetch_param();
                self.ora_in_x(address);
            },
            CPU::ORA_IN_Y => {
                let address = self.fetch_param();
                self.ora_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn ora_im(&mut self, immediate: u8) {
        self.register_a = self.register_a | immediate;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn ora_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.ora_im(value);
    }

    #[inline]
    fn ora_zp_x(&mut self, address: u8) {
        let value = self.memory.zp_x_read(address, self.register_x);
        self.ora_im(value);
    }

    #[inline]
    fn ora_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.ora_im(value);
    }

    #[inline]
    fn ora_ab_x(&mut self, address: u16) {
        let value = self.memory.ab_x_read(address, self.register_x);
        self.ora_im(value);
    }

    #[inline]
    fn ora_ab_y(&mut self, address: u16) {
        let value = self.memory.ab_y_read(address, self.register_y);
        self.ora_im(value);
    }

    #[inline]
    fn ora_in_x(&mut self, address: u8) {
        let value = self.memory.in_x_read(address, self.register_x);
        self.ora_im(value);
    }

    #[inline]
    fn ora_in_y(&mut self, address: u8) {
        let value = self.memory.in_y_read(address, self.register_y);
        self.ora_im(value);
    }

    fn lsr(&mut self, opcode: u8) {
        match opcode {
            CPU::LSR => {
                self.lsr_a();
            },
            CPU::LSR_ZP => {
                let address = self.fetch_param();
                self.lsr_zp(address);
            },
            CPU::LSR_ZP_X => {
                let address = self.fetch_param();
                self.lsr_zp_x(address);
            },
            CPU::LSR_AB => {
                let address = self.fetch_addr_param();
                self.lsr_ab(address);
            },
            CPU::LSR_AB_X => {
                let address = self.fetch_addr_param();
                self.lsr_ab_x(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn lsr_a(&mut self) {
        self.update_status_flag(CARRY_FLAG, self.register_a & 1 != 0);
        self.register_a = self.register_a >> 1;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lsr_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.zp_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn lsr_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.zp_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn lsr_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.ab_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn lsr_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.ab_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    fn sre(&mut self, opcode: u8) {
        match opcode {
            CPU::SRE_ZP => {
                let address = self.fetch_param();
                self.sre_zp(address);
            },
            CPU::SRE_ZP_X => {
                let address = self.fetch_param();
                self.sre_zp_x(address);
            },
            CPU::SRE_AB => {
                let address = self.fetch_addr_param();
                self.sre_ab(address);
            },
            CPU::SRE_AB_X => {
                let address = self.fetch_addr_param();
                self.sre_ab_x(address);
            },
            CPU::SRE_AB_Y => {
                let address = self.fetch_addr_param();
                self.sre_ab_y(address);
            },
            CPU::SRE_IN_X => {
                let address = self.fetch_param();
                self.sre_in_x(address);
            },
            CPU::SRE_IN_Y => {
                let address = self.fetch_param();
                self.sre_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn sre_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.zp_write(address, value);
        self.eor_zp(address);
    }

    #[inline]
    fn sre_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.zp_x_write(address, self.register_x, value);
        self.eor_zp_x(address);
    }

    #[inline]
    fn sre_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.ab_write(address, value);
        self.eor_ab(address);
    }

    #[inline]
    fn sre_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.ab_x_write(address, self.register_x, value);
        self.eor_ab_x(address);
    }

    #[inline]
    fn sre_ab_y(&mut self, address: u16) {
        let mut value = self.memory.ab_y_read(address, self.register_y);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.ab_y_write(address, self.register_y, value);
        self.eor_ab_y(address);
    }

    #[inline]
    fn sre_in_x(&mut self, address: u8) {
        let mut value = self.memory.in_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.in_x_write(address, self.register_x, value);
        self.eor_in_x(address);
    }

    #[inline]
    fn sre_in_y(&mut self, address: u8) {
        let mut value = self.memory.in_y_read(address, self.register_y);
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = value >> 1;
        self.memory.in_y_write(address, self.register_y, value);
        self.eor_in_y(address);
    }

    fn asl(&mut self, opcode: u8) {
        match opcode {
            CPU::ASL => {
                self.asl_a();
            },
            CPU::ASL_ZP => {
                let address = self.fetch_param();
                self.asl_zp(address);
            },
            CPU::ASL_ZP_X => {
                let address = self.fetch_param();
                self.asl_zp_x(address);
            },
            CPU::ASL_AB => {
                let address = self.fetch_addr_param();
                self.asl_ab(address);
            },
            CPU::ASL_AB_X => {
                let address = self.fetch_addr_param();
                self.asl_ab_x(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn asl_a(&mut self) {
        self.update_status_flag(CARRY_FLAG, self.register_a & 0x80 != 0);
        self.register_a = self.register_a << 1;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn asl_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.zp_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn asl_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.zp_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn asl_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.ab_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn asl_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.ab_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    fn slo(&mut self, opcode: u8) {
        match opcode {
            CPU::SLO_ZP => {
                let address = self.fetch_param();
                self.slo_zp(address);
            },
            CPU::SLO_ZP_X => {
                let address = self.fetch_param();
                self.slo_zp_x(address);
            },
            CPU::SLO_AB => {
                let address = self.fetch_addr_param();
                self.slo_ab(address);
            },
            CPU::SLO_AB_X => {
                let address = self.fetch_addr_param();
                self.slo_ab_x(address);
            },
            CPU::SLO_AB_Y => {
                let address = self.fetch_addr_param();
                self.slo_ab_y(address);
            },
            CPU::SLO_IN_X => {
                let address = self.fetch_param();
                self.slo_in_x(address);
            },
            CPU::SLO_IN_Y => {
                let address = self.fetch_param();
                self.slo_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn slo_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.zp_write(address, value);
        self.ora_zp(address);
    }

    #[inline]
    fn slo_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.zp_x_write(address, self.register_x, value);
        self.ora_zp_x(address);
    }

    #[inline]
    fn slo_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.ab_write(address, value);
        self.ora_ab(address);
    }

    #[inline]
    fn slo_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.ab_x_write(address, self.register_x, value);
        self.ora_ab_x(address);
    }

    #[inline]
    fn slo_ab_y(&mut self, address: u16) {
        let mut value = self.memory.ab_y_read(address, self.register_y);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.ab_y_write(address, self.register_y, value);
        self.ora_ab_y(address);
    }

    #[inline]
    fn slo_in_x(&mut self, address: u8) {
        let mut value = self.memory.in_x_read(address, self.register_x);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.in_x_write(address, self.register_x, value);
        self.ora_in_x(address);
    }

    #[inline]
    fn slo_in_y(&mut self, address: u8) {
        let mut value = self.memory.in_y_read(address, self.register_y);
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = value << 1;
        self.memory.in_y_write(address, self.register_y, value);
        self.ora_in_y(address);
    }

    fn ror(&mut self, opcode: u8) {
        match opcode {
            CPU::ROR => {
                self.ror_a();
            },
            CPU::ROR_ZP => {
                let address = self.fetch_param();
                self.ror_zp(address);
            },
            CPU::ROR_ZP_X => {
                let address = self.fetch_param();
                self.ror_zp_x(address);
            },
            CPU::ROR_AB => {
                let address = self.fetch_addr_param();
                self.ror_ab(address);
            },
            CPU::ROR_AB_X => {
                let address = self.fetch_addr_param();
                self.ror_ab_x(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn ror_a(&mut self) {
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, self.register_a & 1 != 0);
        self.register_a = (self.register_a >> 1) | (old_carry << 7);
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn ror_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.zp_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn ror_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.zp_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn ror_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.ab_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn ror_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.ab_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    fn rra(&mut self, opcode: u8) {
        match opcode {
            CPU::RRA_ZP => {
                let address = self.fetch_param();
                self.rra_zp(address);
            },
            CPU::RRA_ZP_X => {
                let address = self.fetch_param();
                self.rra_zp_x(address);
            },
            CPU::RRA_AB => {
                let address = self.fetch_addr_param();
                self.rra_ab(address);
            },
            CPU::RRA_AB_X => {
                let address = self.fetch_addr_param();
                self.rra_ab_x(address);
            },
            CPU::RRA_AB_Y => {
                let address = self.fetch_addr_param();
                self.rra_ab_y(address);
            },
            CPU::RRA_IN_X => {
                let address = self.fetch_param();
                self.rra_in_x(address);
            },
            CPU::RRA_IN_Y => {
                let address = self.fetch_param();
                self.rra_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn rra_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.zp_write(address, value);
        self.adc_zp(address);
    }

    #[inline]
    fn rra_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.zp_x_write(address, self.register_x, value);
        self.adc_zp_x(address);
    }

    #[inline]
    fn rra_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.ab_write(address, value);
        self.adc_ab(address);
    }

    #[inline]
    fn rra_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.ab_x_write(address, self.register_x, value);
        self.adc_ab_x(address);
    }

    #[inline]
    fn rra_ab_y(&mut self, address: u16) {
        let mut value = self.memory.ab_y_read(address, self.register_y);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.ab_y_write(address, self.register_y, value);
        self.adc_ab_y(address);
    }

    #[inline]
    fn rra_in_x(&mut self, address: u8) {
        let mut value = self.memory.in_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.in_x_write(address, self.register_x, value);
        self.adc_in_x(address);
    }

    #[inline]
    fn rra_in_y(&mut self, address: u8) {
        let mut value = self.memory.in_y_read(address, self.register_y);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 1 != 0);
        value = (value >> 1) | (old_carry << 7);
        self.memory.in_y_write(address, self.register_y, value);
        self.adc_in_y(address);
    }

    fn rol(&mut self, opcode: u8) {
        match opcode {
            CPU::ROL => {
                self.rol_a();
            },
            CPU::ROL_ZP => {
                let address = self.fetch_param();
                self.rol_zp(address);
            },
            CPU::ROL_ZP_X => {
                let address = self.fetch_param();
                self.rol_zp_x(address);
            },
            CPU::ROL_AB => {
                let address = self.fetch_addr_param();
                self.rol_ab(address);
            },
            CPU::ROL_AB_X => {
                let address = self.fetch_addr_param();
                self.rol_ab_x(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn rol_a(&mut self) {
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, self.register_a & 0x80 != 0);
        self.register_a = (self.register_a << 1) | old_carry;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn rol_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.zp_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn rol_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.zp_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn rol_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.ab_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn rol_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.ab_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    fn rla(&mut self, opcode: u8) {
        match opcode {
            CPU::RLA_ZP => {
                let address = self.fetch_param();
                self.rla_zp(address);
            },
            CPU::RLA_ZP_X => {
                let address = self.fetch_param();
                self.rla_zp_x(address);
            },
            CPU::RLA_AB => {
                let address = self.fetch_addr_param();
                self.rla_ab(address);
            },
            CPU::RLA_AB_X => {
                let address = self.fetch_addr_param();
                self.rla_ab_x(address);
            },
            CPU::RLA_AB_Y => {
                let address = self.fetch_addr_param();
                self.rla_ab_y(address);
            },
            CPU::RLA_IN_X => {
                let address = self.fetch_param();
                self.rla_in_x(address);
            },
            CPU::RLA_IN_Y => {
                let address = self.fetch_param();
                self.rla_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn rla_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.zp_write(address, value);
        self.and_zp(address);
    }

    #[inline]
    fn rla_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.zp_x_write(address, self.register_x, value);
        self.and_zp_x(address);
    }

    #[inline]
    fn rla_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.ab_write(address, value);
        self.and_ab(address);
    }

    #[inline]
    fn rla_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.ab_x_write(address, self.register_x, value);
        self.and_ab_x(address);
    }

    #[inline]
    fn rla_ab_y(&mut self, address: u16) {
        let mut value = self.memory.ab_y_read(address, self.register_y);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.ab_y_write(address, self.register_y, value);
        self.and_ab_y(address);
    }

    #[inline]
    fn rla_in_x(&mut self, address: u8) {
        let mut value = self.memory.in_x_read(address, self.register_x);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.in_x_write(address, self.register_x, value);
        self.and_in_x(address);
    }

    #[inline]
    fn rla_in_y(&mut self, address: u8) {
        let mut value = self.memory.in_y_read(address, self.register_y);
        let old_carry = self.get_status_flag(CARRY_FLAG) as u8;
        self.update_status_flag(CARRY_FLAG, value & 0x80 != 0);
        value = (value << 1) | old_carry;
        self.memory.in_y_write(address, self.register_y, value);
        self.and_in_y(address);
    }

    fn lda(&mut self, opcode: u8) {
        match opcode {
            CPU::LDA_IM => {
                let immediate = self.fetch_param();
                self.lda_im(immediate);
            },
            CPU::LDA_ZP => {
                let address = self.fetch_param();
                self.lda_zp(address);
            },
            CPU::LDA_ZP_X => {
                let address = self.fetch_param();
                self.lda_zp_x(address);
            },
            CPU::LDA_AB => {
                let address = self.fetch_addr_param();
                self.lda_ab(address);
            },
            CPU::LDA_AB_X => {
                let address = self.fetch_addr_param();
                self.lda_ab_x(address);
            },
            CPU::LDA_AB_Y => {
                let address = self.fetch_addr_param();
                self.lda_ab_y(address);
            },
            CPU::LDA_IN_X => {
                let address = self.fetch_param();
                self.lda_in_x(address);
            },
            CPU::LDA_IN_Y => {
                let address = self.fetch_param();
                self.lda_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn lda_im(&mut self, immediate: u8) {
        self.register_a = immediate;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lda_zp(&mut self, address: u8) {
        self.register_a = self.memory.zp_read(address);
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lda_zp_x(&mut self, address: u8) {
        self.register_a = self.memory.zp_x_read(address, self.register_x);
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lda_ab(&mut self, address: u16) {
        self.register_a = self.memory.ab_read(address);
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lda_ab_x(&mut self, address: u16) {
        self.register_a = self.memory.ab_x_read(address, self.register_x);
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lda_ab_y(&mut self, address: u16) {
        self.register_a = self.memory.ab_y_read(address, self.register_y);
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lda_in_x(&mut self, address: u8) {
        self.register_a = self.memory.in_x_read(address, self.register_x);
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lda_in_y(&mut self, address: u8) {
        self.register_a = self.memory.in_y_read(address, self.register_y);
        self.update_zero_and_negative_flag(self.register_a);
    }

    fn ldx(&mut self, opcode: u8) {
        match opcode {
            CPU::LDX_IM => {
                let immediate = self.fetch_param();
                self.ldx_im(immediate);
            },
            CPU::LDX_ZP => {
                let address = self.fetch_param();
                self.ldx_zp(address);
            },
            CPU::LDX_ZP_Y => {
                let address = self.fetch_param();
                self.ldx_zp_y(address);
            },
            CPU::LDX_AB => {
                let address = self.fetch_addr_param();
                self.ldx_ab(address);
            },
            CPU::LDX_AB_Y => {
                let address = self.fetch_addr_param();
                self.ldx_ab_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn ldx_im(&mut self, immediate: u8) {
        self.register_x = immediate;
        self.update_zero_and_negative_flag(self.register_x);
    }

    #[inline]
    fn ldx_zp(&mut self, address: u8) {
        self.register_x = self.memory.zp_read(address);
        self.update_zero_and_negative_flag(self.register_x);
    }

    #[inline]
    fn ldx_zp_y(&mut self, address: u8) {
        self.register_x = self.memory.zp_y_read(address, self.register_y);
        self.update_zero_and_negative_flag(self.register_x);
    }

    #[inline]
    fn ldx_ab(&mut self, address: u16) {
        self.register_x = self.memory.ab_read(address);
        self.update_zero_and_negative_flag(self.register_x);
    }

    #[inline]
    fn ldx_ab_y(&mut self, address: u16) {
        self.register_x = self.memory.ab_y_read(address, self.register_y);
        self.update_zero_and_negative_flag(self.register_x);
    }

    fn ldy(&mut self, opcode: u8) {
        match opcode {
            CPU::LDY_IM => {
                let immediate = self.fetch_param();
                self.ldy_im(immediate);
            },
            CPU::LDY_ZP => {
                let address = self.fetch_param();
                self.ldy_zp(address);
            },
            CPU::LDY_ZP_X => {
                let address = self.fetch_param();
                self.ldy_zp_x(address);
            },
            CPU::LDY_AB => {
                let address = self.fetch_addr_param();
                self.ldy_ab(address);
            },
            CPU::LDY_AB_X => {
                let address = self.fetch_addr_param();
                self.ldy_ab_x(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn ldy_im(&mut self, immediate: u8) {
        self.register_y = immediate;
        self.update_zero_and_negative_flag(self.register_y);
    }

    #[inline]
    fn ldy_zp(&mut self, address: u8) {
        self.register_y = self.memory.zp_read(address);
        self.update_zero_and_negative_flag(self.register_y);
    }

    #[inline]
    fn ldy_zp_x(&mut self, address: u8) {
        self.register_y = self.memory.zp_x_read(address, self.register_x);
        self.update_zero_and_negative_flag(self.register_y);
    }

    #[inline]
    fn ldy_ab(&mut self, address: u16) {
        self.register_y = self.memory.ab_read(address);
        self.update_zero_and_negative_flag(self.register_y);
    }

    #[inline]
    fn ldy_ab_x(&mut self, address: u16) {
        self.register_y = self.memory.ab_x_read(address, self.register_x);
        self.update_zero_and_negative_flag(self.register_y);
    }

    fn lax(&mut self, opcode: u8) {
        match opcode {
            CPU::LAX_ZP => {
                let address = self.fetch_param();
                self.lax_zp(address);
            },
            CPU::LAX_ZP_Y => {
                let address = self.fetch_param();
                self.lax_zp_y(address);
            },
            CPU::LAX_AB => {
                let address = self.fetch_addr_param();
                self.lax_ab(address);
            },
            CPU::LAX_AB_Y => {
                let address = self.fetch_addr_param();
                self.lax_ab_y(address);
            },
            CPU::LAX_IN_X => {
                let address = self.fetch_param();
                self.lax_in_x(address);
            },
            CPU::LAX_IN_Y => {
                let address = self.fetch_param();
                self.lax_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn lax_zp(&mut self, address: u8) {
        self.register_a = self.memory.zp_read(address);
        self.register_x = self.register_a;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lax_zp_y(&mut self, address: u8) {
        self.register_a = self.memory.zp_y_read(address, self.register_y);
        self.register_x = self.register_a;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lax_ab(&mut self, address: u16) {
        self.register_a = self.memory.ab_read(address);
        self.register_x = self.register_a;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lax_ab_y(&mut self, address: u16) {
        self.register_a = self.memory.ab_y_read(address, self.register_y);
        self.register_x = self.register_a;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lax_in_x(&mut self, address: u8) {
        self.register_a = self.memory.in_x_read(address, self.register_x);
        self.register_x = self.register_a;
        self.update_zero_and_negative_flag(self.register_a);
    }

    #[inline]
    fn lax_in_y(&mut self, address: u8) {
        self.register_a = self.memory.in_y_read(address, self.register_y);
        self.register_x = self.register_a;
        self.update_zero_and_negative_flag(self.register_a);
    }

    fn sta(&mut self, opcode: u8) {
        match opcode {
            CPU::STA_ZP => {
                let address = self.fetch_param();
                self.sta_zp(address);
            },
            CPU::STA_ZP_X => {
                let address = self.fetch_param();
                self.sta_zp_x(address);
            },
            CPU::STA_AB => {
                let address = self.fetch_addr_param();
                self.sta_ab(address);
            },
            CPU::STA_AB_X => {
                let address = self.fetch_addr_param();
                self.sta_ab_x(address);
            },
            CPU::STA_AB_Y => {
                let address = self.fetch_addr_param();
                self.sta_ab_y(address);
            },
            CPU::STA_IN_X => {
                let address = self.fetch_param();
                self.sta_in_x(address);
            },
            CPU::STA_IN_Y => {
                let address = self.fetch_param();
                self.sta_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn sta_zp(&mut self, address: u8) {
        self.memory.zp_write(address, self.register_a);
    }

    #[inline]
    fn sta_zp_x(&mut self, address: u8) {
        self.memory.zp_x_write(address, self.register_x, self.register_a);
    }

    #[inline]
    fn sta_ab(&mut self, address: u16) {
        self.memory.ab_write(address, self.register_a);
    }

    #[inline]
    fn sta_ab_x(&mut self, address: u16) {
        self.memory.ab_x_write(address, self.register_x, self.register_a);
    }

    #[inline]
    fn sta_ab_y(&mut self, address: u16) {
        self.memory.ab_y_write(address, self.register_y, self.register_a);
    }

    #[inline]
    fn sta_in_x(&mut self, address: u8) {
        self.memory.in_x_write(address, self.register_x, self.register_a);
    }

    #[inline]
    fn sta_in_y(&mut self, address: u8) {
        self.memory.in_y_write(address, self.register_y, self.register_a);
    }

    fn stx(&mut self, opcode: u8) {
        match opcode {
            CPU::STX_ZP => {
                let address = self.fetch_param();
                self.stx_zp(address);
            },
            CPU::STX_ZP_Y => {
                let address = self.fetch_param();
                self.stx_zp_y(address);
            },
            CPU::STX_AB => {
                let address = self.fetch_addr_param();
                self.stx_ab(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn stx_zp(&mut self, address: u8) {
        self.memory.zp_write(address, self.register_x);
    }

    #[inline]
    fn stx_zp_y(&mut self, address: u8) {
        self.memory.zp_y_write(address, self.register_y, self.register_x);
    }

    #[inline]
    fn stx_ab(&mut self, address: u16) {
        self.memory.ab_write(address, self.register_x);
    }

    fn sty(&mut self, opcode: u8) {
        match opcode {
            CPU::STY_ZP => {
                let address = self.fetch_param();
                self.sty_zp(address);
            },
            CPU::STY_ZP_X => {
                let address = self.fetch_param();
                self.sty_zp_x(address);
            },
            CPU::STY_AB => {
                let address = self.fetch_addr_param();
                self.sty_ab(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn sty_zp(&mut self, address: u8) {
        self.memory.zp_write(address, self.register_y);
    }

    #[inline]
    fn sty_zp_x(&mut self, address: u8) {
        self.memory.zp_x_write(address, self.register_x, self.register_y);
    }

    #[inline]
    fn sty_ab(&mut self, address: u16) {
        self.memory.ab_write(address, self.register_y);
    }

    fn sax(&mut self, opcode: u8) {
        match opcode {
            CPU::SAX_ZP => {
                let address = self.fetch_param();
                self.sax_zp(address);
            },
            CPU::SAX_ZP_Y => {
                let address = self.fetch_param();
                self.sax_zp_y(address);
            },
            CPU::SAX_AB => {
                let address = self.fetch_addr_param();
                self.sax_ab(address);
            },
            CPU::SAX_IN_X => {
                let address = self.fetch_param();
                self.sax_in_x(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn sax_zp(&mut self, address: u8) {
        self.memory.zp_write(address, self.register_x & self.register_a);
    }

    #[inline]
    fn sax_zp_y(&mut self, address: u8) {
        self.memory.zp_y_write(address, self.register_y, self.register_x & self.register_a);
    }

    #[inline]
    fn sax_ab(&mut self, address: u16) {
        self.memory.ab_write(address, self.register_x & self.register_a);
    }

    #[inline]
    fn sax_in_x(&mut self, address: u8) {
        self.memory.in_x_write(address, self.register_x, self.register_x & self.register_a);
    }

    fn dec(&mut self, opcode: u8) {
        match opcode {
            CPU::DEC_ZP => {
                let address = self.fetch_param();
                self.dec_zp(address);
            },
            CPU::DEC_ZP_X => {
                let address = self.fetch_param();
                self.dec_zp_x(address);
            },
            CPU::DEC_AB => {
                let address = self.fetch_addr_param();
                self.dec_ab(address);
            },
            CPU::DEC_AB_X => {
                let address = self.fetch_addr_param();
                self.dec_ab_x(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn dec_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        value = value.wrapping_sub(1);
        self.memory.zp_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn dec_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        value = value.wrapping_sub(1);
        self.memory.zp_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn dec_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        value = value.wrapping_sub(1);
        self.memory.ab_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn dec_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        value = value.wrapping_sub(1);
        self.memory.ab_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    fn dcp(&mut self, opcode: u8) {
        match opcode {
            CPU::DCP_ZP => {
                let address = self.fetch_param();
                self.dcp_zp(address);
            },
            CPU::DCP_ZP_X => {
                let address = self.fetch_param();
                self.dcp_zp_x(address);
            },
            CPU::DCP_AB => {
                let address = self.fetch_addr_param();
                self.dcp_ab(address);
            },
            CPU::DCP_AB_X => {
                let address = self.fetch_addr_param();
                self.dcp_ab_x(address);
            },
            CPU::DCP_AB_Y => {
                let address = self.fetch_addr_param();
                self.dcp_ab_y(address);
            },
            CPU::DCP_IN_X => {
                let address = self.fetch_param();
                self.dcp_in_x(address);
            },
            CPU::DCP_IN_Y => {
                let address = self.fetch_param();
                self.dcp_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn dcp_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        value = value.wrapping_sub(1);
        self.memory.zp_write(address, value);
        self.cmp_zp(address);
    }

    #[inline]
    fn dcp_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        value = value.wrapping_sub(1);
        self.memory.zp_x_write(address, self.register_x, value);
        self.cmp_zp_x(address);
    }

    #[inline]
    fn dcp_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        value = value.wrapping_sub(1);
        self.memory.ab_write(address, value);
        self.cmp_ab(address);
    }

    #[inline]
    fn dcp_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        value = value.wrapping_sub(1);
        self.memory.ab_x_write(address, self.register_x, value);
        self.cmp_ab_x(address);
    }

    #[inline]
    fn dcp_ab_y(&mut self, address: u16) {
        let mut value = self.memory.ab_y_read(address, self.register_y);
        value = value.wrapping_sub(1);
        self.memory.ab_y_write(address, self.register_y, value);
        self.cmp_ab_y(address);
    }

    #[inline]
    fn dcp_in_x(&mut self, address: u8) {
        let mut value = self.memory.in_x_read(address, self.register_x);
        value = value.wrapping_sub(1);
        self.memory.in_x_write(address, self.register_x, value);
        self.cmp_in_x(address);
    }

    #[inline]
    fn dcp_in_y(&mut self, address: u8) {
        let mut value = self.memory.in_y_read(address, self.register_y);
        value = value.wrapping_sub(1);
        self.memory.in_y_write(address, self.register_y, value);
        self.cmp_in_y(address);
    }

    fn inc(&mut self, opcode: u8) {
        match opcode {
            CPU::INC_ZP => {
                let address = self.fetch_param();
                self.inc_zp(address);
            },
            CPU::INC_ZP_X => {
                let address = self.fetch_param();
                self.inc_zp_x(address);
            },
            CPU::INC_AB => {
                let address = self.fetch_addr_param();
                self.inc_ab(address);
            },
            CPU::INC_AB_X => {
                let address = self.fetch_addr_param();
                self.inc_ab_x(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn inc_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        value = value.wrapping_add(1);
        self.memory.zp_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn inc_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        value = value.wrapping_add(1);
        self.memory.zp_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn inc_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        value = value.wrapping_add(1);
        self.memory.ab_write(address, value);
        self.update_zero_and_negative_flag(value);
    }

    #[inline]
    fn inc_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        value = value.wrapping_add(1);
        self.memory.ab_x_write(address, self.register_x, value);
        self.update_zero_and_negative_flag(value);
    }

    fn isb(&mut self, opcode: u8) {
        match opcode {
            CPU::ISB_ZP => {
                let address = self.fetch_param();
                self.isb_zp(address);
            },
            CPU::ISB_ZP_X => {
                let address = self.fetch_param();
                self.isb_zp_x(address);
            },
            CPU::ISB_AB => {
                let address = self.fetch_addr_param();
                self.isb_ab(address);
            },
            CPU::ISB_AB_X => {
                let address = self.fetch_addr_param();
                self.isb_ab_x(address);
            },
            CPU::ISB_AB_Y => {
                let address = self.fetch_addr_param();
                self.isb_ab_y(address);
            },
            CPU::ISB_IN_X => {
                let address = self.fetch_param();
                self.isb_in_x(address);
            },
            CPU::ISB_IN_Y => {
                let address = self.fetch_param();
                self.isb_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn isb_zp(&mut self, address: u8) {
        let mut value = self.memory.zp_read(address);
        value = value.wrapping_add(1);
        self.memory.zp_write(address, value);
        self.sbc_zp(address);
    }

    #[inline]
    fn isb_zp_x(&mut self, address: u8) {
        let mut value = self.memory.zp_x_read(address, self.register_x);
        value = value.wrapping_add(1);
        self.memory.zp_x_write(address, self.register_x, value);
        self.sbc_zp_x(address);
    }

    #[inline]
    fn isb_ab(&mut self, address: u16) {
        let mut value = self.memory.ab_read(address);
        value = value.wrapping_add(1);
        self.memory.ab_write(address, value);
        self.sbc_ab(address);
    }

    #[inline]
    fn isb_ab_x(&mut self, address: u16) {
        let mut value = self.memory.ab_x_read(address, self.register_x);
        value = value.wrapping_add(1);
        self.memory.ab_x_write(address, self.register_x, value);
        self.sbc_ab_x(address);
    }

    #[inline]
    fn isb_ab_y(&mut self, address: u16) {
        let mut value = self.memory.ab_y_read(address, self.register_y);
        value = value.wrapping_add(1);
        self.memory.ab_y_write(address, self.register_y, value);
        self.sbc_ab_y(address);
    }

    #[inline]
    fn isb_in_x(&mut self, address: u8) {
        let mut value = self.memory.in_x_read(address, self.register_x);
        value = value.wrapping_add(1);
        self.memory.in_x_write(address, self.register_x, value);
        self.sbc_in_x(address);
    }

    #[inline]
    fn isb_in_y(&mut self, address: u8) {
        let mut value = self.memory.in_y_read(address, self.register_y);
        value = value.wrapping_add(1);
        self.memory.in_y_write(address, self.register_y, value);
        self.sbc_in_y(address);
    }

    fn _cmp(&mut self, opcode: u8) {
        match opcode {
            CPU::CMP_IM => {
                let immediate = self.fetch_param();
                self.cmp_im(immediate);
            },
            CPU::CMP_ZP => {
                let address = self.fetch_param();
                self.cmp_zp(address);
            },
            CPU::CMP_ZP_X => {
                let address = self.fetch_param();
                self.cmp_zp_x(address);
            },
            CPU::CMP_AB => {
                let address = self.fetch_addr_param();
                self.cmp_ab(address);
            },
            CPU::CMP_AB_X => {
                let address = self.fetch_addr_param();
                self.cmp_ab_x(address);
            },
            CPU::CMP_AB_Y => {
                let address = self.fetch_addr_param();
                self.cmp_ab_y(address);
            },
            CPU::CMP_IN_X => {
                let address = self.fetch_param();
                self.cmp_in_x(address);
            },
            CPU::CMP_IN_Y => {
                let address = self.fetch_param();
                self.cmp_in_y(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn cmp_im(&mut self, immediate: u8) {
        let cmp = self.register_a.wrapping_sub(immediate);
        self.update_status_flag(CARRY_FLAG, self.register_a >= immediate);
        self.update_zero_and_negative_flag(cmp);
    }

    #[inline]
    fn cmp_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.cmp_im(value);
    }

    #[inline]
    fn cmp_zp_x(&mut self, address: u8) {
        let value = self.memory.zp_x_read(address, self.register_x);
        self.cmp_im(value);
    }

    #[inline]
    fn cmp_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.cmp_im(value);
    }

    #[inline]
    fn cmp_ab_x(&mut self, address: u16) {
        let value = self.memory.ab_x_read(address, self.register_x);
        self.cmp_im(value);
    }

    #[inline]
    fn cmp_ab_y(&mut self, address: u16) {
        let value = self.memory.ab_y_read(address, self.register_y);
        self.cmp_im(value);
    }

    #[inline]
    fn cmp_in_x(&mut self, address: u8) {
        let value = self.memory.in_x_read(address, self.register_x);
        self.cmp_im(value);
    }

    #[inline]
    fn cmp_in_y(&mut self, address: u8) {
        let value = self.memory.in_y_read(address, self.register_y);
        self.cmp_im(value);
    }

    fn cpx(&mut self, opcode: u8) {
        match opcode {
            CPU::CPX_IM => {
                let immediate = self.fetch_param();
                self.cpx_im(immediate);
            },
            CPU::CPX_ZP => {
                let address = self.fetch_param();
                self.cpx_zp(address);
            },
            CPU::CPX_AB => {
                let address = self.fetch_addr_param();
                self.cpx_ab(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn cpx_im(&mut self, immediate: u8) {
        let cmp = self.register_x.wrapping_sub(immediate);
        self.update_status_flag(CARRY_FLAG, self.register_x >= immediate);
        self.update_zero_and_negative_flag(cmp);
    }

    #[inline]
    fn cpx_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.cpx_im(value);
    }

    #[inline]
    fn cpx_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.cpx_im(value);
    }

    fn cpy(&mut self, opcode: u8) {
        match opcode {
            CPU::CPY_IM => {
                let immediate = self.fetch_param();
                self.cpy_im(immediate);
            },
            CPU::CPY_ZP => {
                let address = self.fetch_param();
                self.cpy_zp(address);
            },
            CPU::CPY_AB => {
                let address = self.fetch_addr_param();
                self.cpy_ab(address);
            },
            _ => panic!("invalid opcode: {:x}", opcode)
        }
        self.increment_program_counter()
    }

    #[inline]
    fn cpy_im(&mut self, immediate: u8) {
        let cmp = self.register_y.wrapping_sub(immediate);
        self.update_status_flag(CARRY_FLAG, self.register_y >= immediate);
        self.update_zero_and_negative_flag(cmp);
    }

    #[inline]
    fn cpy_zp(&mut self, address: u8) {
        let value = self.memory.zp_read(address);
        self.cpy_im(value);
    }

    #[inline]
    fn cpy_ab(&mut self, address: u16) {
        let value = self.memory.ab_read(address);
        self.cpy_im(value);
    }

    #[inline]
    fn get_status_flag(&self, flag: u8) -> bool {
        self.status.view_bits::<Lsb0>()[flag as usize]
    }

    #[inline]
    fn set_status_flag(&mut self, flag: u8) {
        self.update_status_flag(flag, true);
    }

    #[inline]
    fn clear_status_flag(&mut self, flag: u8) {
        self.update_status_flag(flag, false);
    }

    #[inline]
    fn update_status_flag(&mut self, flag: u8, value: bool) {
        self.status.view_bits_mut::<Lsb0>().set(flag as usize, value);
    }

    #[inline]
    fn update_zero_and_negative_flag(&mut self, value: u8) {
        self.update_status_flag(ZERO_FLAG, value == 0);
        self.update_status_flag(NEGATIVE_FLAG, value & 0x80 > 0);
    }

    #[inline]
    fn update_bit_flags(&mut self, value: u8) {
        let test = value & self.register_a;
        self.update_status_flag(ZERO_FLAG, test == 0);
        self.update_status_flag(NEGATIVE_FLAG, value & 0x80 > 0);
        self.update_status_flag(OVERFLOW_FLAG, (value << 1) & 0x80 > 0);
    }

    #[inline]
    fn increment_program_counter(&mut self) {
        self.program_counter = self.program_counter.wrapping_add(1);
    }

    #[inline]
    fn fetch_param(&mut self) -> u8 {
        self.increment_program_counter();
        self.memory.read_byte(self.program_counter)
    }

    #[inline]
    fn fetch_addr_param(&mut self) -> u16 {
        self.increment_program_counter();
        self.increment_program_counter();
        self.memory.read_addr(self.program_counter.wrapping_sub(1))
    }

    #[inline]
    fn push_byte(&mut self, value: u8) {
        self.memory.write_byte(0x0100 + self.stack as u16, value);
        self.stack = self.stack.wrapping_sub(1);
    }

    #[inline]
    fn pop_byte(&mut self) -> u8 {
        self.stack = self.stack.wrapping_add(1);
        self.memory.read_byte(0x0100 + self.stack as u16)
    }

    #[inline]
    fn push_addr(&mut self, value: u16) {
        self.memory.write_addr(0x0100 + self.stack.wrapping_sub(1) as u16, value);
        self.stack = self.stack.wrapping_sub(2);
    }

    #[inline]
    fn pop_addr(&mut self) -> u16 {
        self.stack = self.stack.wrapping_add(2);
        self.memory.read_addr(0x0100 + self.stack.wrapping_sub(1) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTE_A: u8 = 0x0a;
    const BYTE_B: u8 = 0x0b;

    #[test]
    fn test_init() {
        let cpu = CPU::new();
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.register_x, 0);
        assert_eq!(cpu.register_y, 0);
        assert_eq!(cpu.program_counter, 0);
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.get_status_flag(UNUSED_FLAG), true);
        assert_eq!(cpu.get_status_flag(BREAK_COMMAND), true);
    }

    #[test]
    fn test_reset() {
        let mut cpu = CPU::new();
        cpu.reset();
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.register_x, 0);
        assert_eq!(cpu.register_y, 0);
        assert_eq!(cpu.program_counter, 0);
        assert_eq!(cpu.stack, 0xfd);
        assert_eq!(cpu.get_status_flag(UNUSED_FLAG), true);
        assert_eq!(cpu.get_status_flag(BREAK_COMMAND), false);
        assert_eq!(cpu.get_status_flag(INTERRUPT_DISABLE), true);
    }

    /* BRK and JAM */

    #[test]
    fn test_step_brk() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0, CPU::BRK);
        cpu.step().unwrap_or_default();
        assert_eq!(cpu.program_counter, 1);
    }

    #[test]
    fn test_step_jam() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0, CPU::JAM_1);
        cpu.step().unwrap();
        cpu.step().unwrap();
        cpu.step().unwrap();
        assert_eq!(cpu.program_counter, 0);
    }
    
    /* Set & Clear Flags */

    #[test]
    fn test_sec() {
        let mut cpu = CPU::new();
        cpu.sec();
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_clc() {
        let mut cpu = CPU::new();
        cpu.status = 0b1111_1111;
        cpu.clc();
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), false);
    }

    #[test]
    fn test_sed() {
        let mut cpu = CPU::new();
        cpu.sed();
        assert_eq!(cpu.get_status_flag(DECIMAL_MODE_FLAG), true);
    }

    #[test]
    fn test_cld() {
        let mut cpu = CPU::new();
        cpu.status = 0b1111_1111;
        cpu.cld();
        assert_eq!(cpu.get_status_flag(DECIMAL_MODE_FLAG), false);
    }

    #[test]
    fn test_sei() {
        let mut cpu = CPU::new();
        cpu.sei();
        assert_eq!(cpu.get_status_flag(INTERRUPT_DISABLE), true);
    }

    #[test]
    fn test_cli() {
        let mut cpu = CPU::new();
        cpu.status = 0b1111_1111;
        cpu.cli();
        assert_eq!(cpu.get_status_flag(INTERRUPT_DISABLE), false);
    }

    #[test]
    fn test_clv() {
        let mut cpu = CPU::new();
        cpu.status = 0b1111_1111;
        cpu.clv();
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), false);
    }

    /* Stack */

    #[test]
    fn test_pha() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.pha();
        assert_eq!(cpu.stack, 0xfe);
        assert_eq!(cpu.memory.read_byte(0x01ff), BYTE_A);
    }

    #[test]
    fn test_pla() {
        let mut cpu = CPU::new();
        cpu.status = 0b0111_1010;
        cpu.php();
        cpu.pla();
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.register_a, 0b0111_1010);
        assert_eq!(cpu.memory.read_byte(0x01ff), 0b0111_1010);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
    }

    #[test]
    fn test_pla_zero() {
        let mut cpu = CPU::new();
        cpu.pha();
        cpu.pla();
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.memory.read_byte(0x01ff), 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
    }

    #[test]
    fn test_pla_negative() {
        let mut cpu = CPU::new();
        cpu.status = 0b1011_1010;
        cpu.php();
        cpu.pla();
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.register_a, 0b1011_1010);
        assert_eq!(cpu.memory.read_byte(0x01ff), 0b1011_1010);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    #[test]
    fn test_php() {
        let mut cpu = CPU::new();
        cpu.status = 0b1011_1010;
        cpu.php();
        assert_eq!(cpu.stack, 0xfe);
        assert_eq!(cpu.memory.read_byte(0x01ff), 0b1011_1010);
    }

    #[test]
    fn test_php_set_b_flag() {
        let mut cpu = CPU::new();
        cpu.php();
        assert_eq!(cpu.stack, 0xfe);
        assert_eq!(cpu.memory.read_byte(0x01ff), 0b0011_0000);
    }

    #[test]
    fn test_plp() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1010_1010;
        cpu.pha();
        cpu.plp();
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.status, 0b1010_1010);
        assert_eq!(cpu.memory.read_byte(0x01ff), 0b1010_1010);
    }

    #[test]
    fn test_plp_set_b_flag() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x04;
        cpu.pha();
        cpu.plp();
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.status, 0x24);
        assert_eq!(cpu.memory.read_byte(0x01ff), 0x04);
    }

    #[test]
    fn test_plp_clear_b_flag() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        cpu.pha();
        cpu.plp();
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.status, 0xef);
        assert_eq!(cpu.memory.read_byte(0x01ff), 0xff);
    }

    /* Bit Test */

    #[test]
    fn test_bit_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0b0011_1111);
        cpu.register_a = 0b0110_0011;
        cpu.bit_zp(0x10);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), false);
    }

    #[test]
    fn test_bit_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 0b0011_1111);
        cpu.register_a = 0b0110_0011;
        cpu.bit_ab(0x1400);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), false);
    }

    #[test]
    fn test_bit_zero() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0b0011_1100);
        cpu.register_a = 0b1100_0011;
        cpu.bit_zp(0x10);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), false);
    }

    #[test]
    fn test_bit_negative() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0b1000_0000);
        cpu.register_a = 0b1111_1111;
        cpu.bit_zp(0x10);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), false);
    }

    #[test]
    fn test_bit_overflow() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0b0100_0000);
        cpu.register_a = 0b1111_1111;
        cpu.bit_zp(0x10);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }
    
    /* Add */

    #[test]
    fn test_adc_im() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.adc_im(BYTE_A);
        assert_eq!(cpu.register_a, BYTE_B);
    }

    #[test]
    fn test_adc_zp() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.adc_zp(0x10);
        assert_eq!(cpu.register_a, BYTE_B);
    }

    #[test]
    fn test_adc_zp_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.register_x = 0x08;
        cpu.adc_zp_x(0x08);
        assert_eq!(cpu.register_a, BYTE_B);
    }

    #[test]
    fn test_adc_ab() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.adc_ab(0x1400);
        assert_eq!(cpu.register_a, BYTE_B);
    }

    #[test]
    fn test_adc_ab_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_x = 0x10;
        cpu.adc_ab_x(0x1400);
        assert_eq!(cpu.register_a, BYTE_B);
    }

    #[test]
    fn test_adc_ab_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_y = 0x10;
        cpu.adc_ab_y(0x1400);
        assert_eq!(cpu.register_a, BYTE_B);
    }

    #[test]
    fn test_adc_in_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_x = 0x08;
        cpu.adc_in_x(0x08);
        assert_eq!(cpu.register_a, BYTE_B);
    }

    #[test]
    fn test_adc_in_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_y = 0x10;
        cpu.adc_in_y(0x10);
        assert_eq!(cpu.register_a, BYTE_B);
    }

    #[test]
    fn test_adc_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        cpu.adc_im(0x01);
        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
    }

    #[test]
    fn test_adc_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xfe;
        cpu.adc_im(0x01);
        assert_eq!(cpu.register_a, 0xff);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    #[test]
    fn test_adc_carry() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        cpu.adc_im(0xff);
        assert_eq!(cpu.register_a, 0xfe);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_adc_add_positives_overflow() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x64;
        cpu.adc_im(0x64);
        assert_eq!(cpu.register_a, 0xc8);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_adc_add_positives_overflow_with_carry() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x64;
        cpu.adc_im(0x64);
        assert_eq!(cpu.register_a, 0xc9);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_adc_add_negatives_overflow() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x9C;
        cpu.adc_im(0x9C);
        assert_eq!(cpu.register_a, 0x38);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_adc_add_negatives_overflow_with_carry() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x9C;
        cpu.adc_im(0x9C);
        assert_eq!(cpu.register_a, 0x39);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_adc_carry_overflow() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x0f;
        cpu.adc_im(0x70);
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_adc_add_zero_carry_overflow() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x7f;
        cpu.adc_im(0x00);
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }
    
    #[test]
    fn test_adc_carry_wraparound() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x0f;
        cpu.adc_im(0xf0);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), false);
    }

    /* Subtract */

    #[test]
    fn test_sbc_im() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.sbc_im(0x01);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_sbc_zp() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.memory.write_byte(0x10, 0x01);
        cpu.sbc_zp(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_sbc_zp_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.memory.write_byte(0x10, 0x01);
        cpu.register_x = 0x08;
        cpu.sbc_zp_x(0x08);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_sbc_ab() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.memory.write_byte(0x1400, 0x01);
        cpu.sbc_ab(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_sbc_ab_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.memory.write_byte(0x1410, 0x01);
        cpu.register_x = 0x10;
        cpu.sbc_ab_x(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_sbc_ab_y() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.memory.write_byte(0x1410, 0x01);
        cpu.register_y = 0x10;
        cpu.sbc_ab_y(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_sbc_in_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.memory.write_byte(0x1400, 0x01);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_x = 0x08;
        cpu.sbc_in_x(0x08);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_sbc_in_y() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.memory.write_byte(0x1410, 0x01);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_y = 0x10;
        cpu.sbc_in_y(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_sbc_zero() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x01;
        cpu.sbc_im(0x01);
        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
    }

    #[test]
    fn test_sbc_negative() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0xff;
        cpu.sbc_im(0x01);
        assert_eq!(cpu.register_a, 0xfe);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    #[test]
    fn test_sbc_carry() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x10;
        cpu.sbc_im(0x01);
        assert_eq!(cpu.register_a, 0x0f);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_sbc_sub_negatives_carry() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0xff;
        cpu.sbc_im(0xff);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_sbc_sub_negatives_carry_with_borrow() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        cpu.sbc_im(0xff);
        assert_eq!(cpu.register_a, 0xff);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), false);
    }

    #[test]
    fn test_sbc_sub_positive_overflow() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x9C;
        cpu.sbc_im(0x64);
        assert_eq!(cpu.register_a, 0x38);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_sbc_sub_positive_overflow_with_borrow() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x9C;
        cpu.sbc_im(0x64);
        assert_eq!(cpu.register_a, 0x37);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_sbc_sub_negative_overflow() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0x64;
        cpu.sbc_im(0x9C);
        assert_eq!(cpu.register_a, 0xc8);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_sbc_sub_negative_overflow_with_borrow() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x64;
        cpu.sbc_im(0x9C);
        assert_eq!(cpu.register_a, 0xc7);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_sbc_borrow_overflow() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x80;
        cpu.sbc_im(0x0f);
        assert_eq!(cpu.register_a, 0x70);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_sbc_add_zero_borrow_overflow() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x80;
        cpu.sbc_im(0x00);
        assert_eq!(cpu.register_a, 0x7f);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_sbc_borrow_wraparound() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x00;
        cpu.sbc_im(0x00);
        assert_eq!(cpu.register_a, 0xff);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), false);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), false);
    }

    /* Bitwise */

    #[test]
    fn test_eor_im() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0101_1010;
        cpu.eor_im(0b1010_1010);
        assert_eq!(cpu.register_a, 0b1111_0000);
    }

    #[test]
    fn test_eor_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0b1010_1010);
        cpu.register_a = 0b0101_1010;
        cpu.eor_zp(0x10);
        assert_eq!(cpu.register_a, 0b1111_0000);
    }

    #[test]
    fn test_eor_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, 0b1010_1010);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.eor_zp_x(0x10);
        assert_eq!(cpu.register_a, 0b1111_0000);
    }

    #[test]
    fn test_eor_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 0b1010_1010);
        cpu.register_a = 0b0101_1010;
        cpu.eor_ab(0x1400);
        assert_eq!(cpu.register_a, 0b1111_0000);
    }

    #[test]
    fn test_eor_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0b1010_1010);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.eor_ab_x(0x1400);
        assert_eq!(cpu.register_a, 0b1111_0000);
    }

    #[test]
    fn test_eor_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0b1010_1010);
        cpu.register_a = 0b0101_1010;
        cpu.register_y = 0x10;
        cpu.eor_ab_y(0x1400);
        assert_eq!(cpu.register_a, 0b1111_0000);
    }

    #[test]
    fn test_eor_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x20, 0x1400);
        cpu.memory.write_byte(0x1400, 0b1010_1010);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.eor_in_x(0x10);
        assert_eq!(cpu.register_a, 0b1111_0000);
    }

    #[test]
    fn test_eor_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.memory.write_byte(0x1410, 0b1010_1010);
        cpu.register_a = 0b0101_1010;
        cpu.register_y = 0x10;
        cpu.eor_in_y(0x10);
        assert_eq!(cpu.register_a, 0b1111_0000);
    }

    #[test]
    fn test_eor_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0101_1010;
        cpu.eor_im(0b0101_1010);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
    }

    #[test]
    fn test_eor_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0101_1010;
        cpu.eor_im(0b1101_1010);
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    #[test]
    fn test_double_eor_cancels_out() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0101_1010;
        cpu.eor_im(0b1101_1011);
        cpu.eor_im(0b1101_1011);
        assert_eq!(cpu.register_a, 0b0101_1010);
    }

    #[test]
    fn test_and_im() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0101_1010;
        cpu.and_im(0b0110_0110);
        assert_eq!(cpu.register_a, 0b0100_0010);
    }

    #[test]
    fn test_and_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.and_zp(0x10);
        assert_eq!(cpu.register_a, 0b0100_0010);
    }

    #[test]
    fn test_and_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.and_zp_x(0x10);
        assert_eq!(cpu.register_a, 0b0100_0010);
    }

    #[test]
    fn test_and_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.and_ab(0x1400);
        assert_eq!(cpu.register_a, 0b0100_0010);
    }

    #[test]
    fn test_and_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.and_ab_x(0x1400);
        assert_eq!(cpu.register_a, 0b0100_0010);
    }

    #[test]
    fn test_and_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_y = 0x10;
        cpu.and_ab_y(0x1400);
        assert_eq!(cpu.register_a, 0b0100_0010);
    }

    #[test]
    fn test_and_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x20, 0x1400);
        cpu.memory.write_byte(0x1400, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.and_in_x(0x10);
        assert_eq!(cpu.register_a, 0b0100_0010);
    }

    #[test]
    fn test_and_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.memory.write_byte(0x1410, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_y = 0x10;
        cpu.and_in_y(0x10);
        assert_eq!(cpu.register_a, 0b0100_0010);
    }

    #[test]
    fn test_and_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0101_1010;
        cpu.and_im(0b1010_0101);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
    }

    #[test]
    fn test_and_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1101_1010;
        cpu.and_im(0b1010_0101);
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    #[test]
    fn test_anc() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x9b;
        cpu.anc(0xf1);
        assert_eq!(cpu.register_a, 0x91);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
    }

    #[test]
    fn test_arr() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0b1110_0000;
        cpu.arr(0b1110_1010);
        assert_eq!(cpu.register_a, 0b1111_0000);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), false);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_arr_overflow() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0b1011_0000;
        cpu.arr(0b1110_1010);
        assert_eq!(cpu.register_a, 0b1101_0000);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
        assert_eq!(cpu.get_status_flag(OVERFLOW_FLAG), true);
    }

    #[test]
    fn test_alr() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0b1110_0001;
        cpu.alr(0b1110_1011);
        assert_eq!(cpu.register_a, 0b0111_0000);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_lxa() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0b1110_0001;
        cpu.lxa(0b1110_1011);
        assert_eq!(cpu.register_a, 0b1110_0001);
        assert_eq!(cpu.register_x, 0b1110_0001);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    #[test]
    fn test_las() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x140a, 0b1010_1010);
        cpu.stack = 0b1010_0011;
        cpu.register_y = BYTE_A;
        cpu.las(0x1400);
        assert_eq!(cpu.register_a, 0b1010_0010);
        assert_eq!(cpu.register_x, 0b1010_0010);
        assert_eq!(cpu.stack, 0b1010_0010);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    #[test]
    fn test_sha_ab_y() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.register_a = 0b1010_0001;
        cpu.register_x = 0b1110_1101;
        cpu.sha_ab_y(0x1480);
        assert_eq!(cpu.register_a, 0b1010_0001);
        assert_eq!(cpu.register_x, 0b1110_1101);
        assert_eq!(cpu.memory.read_byte(0x148a), 0x01);
    }

    #[test]
    fn test_sha_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x24, 0x1480);
        cpu.register_y = BYTE_A;
        cpu.register_a = 0b1010_0001;
        cpu.register_x = 0b1110_1101;
        cpu.sha_in_y(0x24);
        assert_eq!(cpu.register_a, 0b1010_0001);
        assert_eq!(cpu.register_x, 0b1110_1101);
        assert_eq!(cpu.memory.read_byte(0x148a), 0x21);
    }

    #[test]
    fn test_shx() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.register_x = 0b1110_1101;
        cpu.shx(0x1480);
        assert_eq!(cpu.register_x, 0b1110_1101);
        assert_eq!(cpu.memory.read_byte(0x148a), 0x05);
    }

    #[test]
    fn test_shy() {
        let mut cpu = CPU::new();
        cpu.register_x = BYTE_A;
        cpu.register_y = 0b1110_1101;
        cpu.shy(0x1480);
        assert_eq!(cpu.register_y, 0b1110_1101);
        assert_eq!(cpu.memory.read_byte(0x148a), 0x05);
    }

    #[test]
    fn test_shs() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.register_a = 0b1010_0001;
        cpu.register_x = 0b1110_1101;
        cpu.shs(0x1480);
        assert_eq!(cpu.stack, 0b1010_0001);
        assert_eq!(cpu.memory.read_byte(0x148a), 0x01);
    }

    #[test]
    fn test_sbx() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.register_a = 0b1010_0101;
        cpu.register_x = 0b1110_1101;
        cpu.sbx(0x04);
        assert_eq!(cpu.register_x, 0b1010_0001);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_ane_zero_immediate() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_x = BYTE_B;
        cpu.ane(0);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
    }

    #[test]
    fn test_ane_ff_accumulator() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        cpu.register_x = BYTE_B;
        cpu.ane(BYTE_A);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
    }

    #[test]
    fn test_ane() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x11;
        cpu.register_x = BYTE_B;
        cpu.ane(BYTE_A);
        assert_eq!(cpu.register_a == 0x11, false);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
    }

    #[test]
    fn test_ora_im() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0101_1010;
        cpu.ora_im(0b0110_0110);
        assert_eq!(cpu.register_a, 0b0111_1110);
    }

    #[test]
    fn test_ora_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.ora_zp(0x10);
        assert_eq!(cpu.register_a, 0b0111_1110);
    }

    #[test]
    fn test_ora_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.ora_zp_x(0x10);
        assert_eq!(cpu.register_a, 0b0111_1110);
    }

    #[test]
    fn test_ora_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.ora_ab(0x1400);
        assert_eq!(cpu.register_a, 0b0111_1110);
    }

    #[test]
    fn test_ora_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.ora_ab_x(0x1400);
        assert_eq!(cpu.register_a, 0b0111_1110);
    }

    #[test]
    fn test_ora_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_y = 0x10;
        cpu.ora_ab_y(0x1400);
        assert_eq!(cpu.register_a, 0b0111_1110);
    }

    #[test]
    fn test_ora_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x20, 0x1400);
        cpu.memory.write_byte(0x1400, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_x = 0x10;
        cpu.ora_in_x(0x10);
        assert_eq!(cpu.register_a, 0b0111_1110);
    }

    #[test]
    fn test_ora_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.memory.write_byte(0x1410, 0b0110_0110);
        cpu.register_a = 0b0101_1010;
        cpu.register_y = 0x10;
        cpu.ora_in_y(0x10);
        assert_eq!(cpu.register_a, 0b0111_1110);
    }

    #[test]
    fn test_ora_zero() {
        let mut cpu = CPU::new();
        cpu.ora_im(0);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
    }

    #[test]
    fn test_ora_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0101_1010;
        cpu.ora_im(0b1010_0101);
        assert_eq!(cpu.register_a, 0xff);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    /* Shift */

    #[test]
    fn test_lsr_a() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0000_1111;
        cpu.lsr_a();
        assert_eq!(cpu.register_a, 0b0000_0111);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }
    
    #[test]
    fn test_lsr_zp() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x10, 0b0000_1111);
        cpu.lsr_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), 0b0000_0111);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_lsr_zp_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x20, 0b0000_1111);
        cpu.register_x = 0x10;
        cpu.lsr_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), 0b0000_0111);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_lsr_ab() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x1400, 0b0000_1111);
        cpu.lsr_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), 0b0000_0111);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_lsr_ab_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x1410, 0b0000_1111);
        cpu.register_x = 0x10;
        cpu.lsr_ab_x(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), 0b0000_0111);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_lsr_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;
        cpu.lsr_a();
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_lsr_no_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        cpu.lsr_a();
        assert_eq!(cpu.register_a, 0x7F);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), false);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_lsr_shift_to_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        for _i in 0..8 {
            cpu.lsr_a();
        }
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_sre_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 2);
        cpu.register_a = BYTE_B;
        cpu.sre_zp(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x10), 1);
    }

    #[test]
    fn test_sre_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, 2);
        cpu.register_a = BYTE_B;
        cpu.register_x = 0x10;
        cpu.sre_zp_x(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x20), 1);
    }

    #[test]
    fn test_sre_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 2);
        cpu.register_a = BYTE_B;
        cpu.sre_ab(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1400), 1);
    }

    #[test]
    fn test_sre_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 2);
        cpu.register_a = BYTE_B;
        cpu.register_x = 0x10;
        cpu.sre_ab_x(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1410), 1);
    }

    #[test]
    fn test_sre_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 2);
        cpu.register_a = BYTE_B;
        cpu.register_y = 0x10;
        cpu.sre_ab_y(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1410), 1);
    }

    #[test]
    fn test_sre_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 2);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_B;
        cpu.register_x = 0x08;
        cpu.sre_in_x(0x08);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1400), 1);
    }

    #[test]
    fn test_sre_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 2);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_B;
        cpu.register_y = 0x10;
        cpu.sre_in_y(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1410), 1);
    }

    #[test]
    fn test_asl_a() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1111_0000;
        cpu.asl_a();
        assert_eq!(cpu.register_a, 0b1110_0000);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }
    
    #[test]
    fn test_asl_zp() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x10, 0b1111_0000);
        cpu.asl_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), 0b1110_0000);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_asl_zp_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x20, 0b1111_0000);
        cpu.register_x = 0x10;
        cpu.asl_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), 0b1110_0000);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_asl_ab() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x1400, 0b1111_0000);
        cpu.asl_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), 0b1110_0000);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_asl_ab_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x1410, 0b1111_0000);
        cpu.register_x = 0x10;
        cpu.asl_ab_x(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), 0b1110_0000);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_asl_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x80;
        cpu.asl_a();
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_asl_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x40;
        cpu.asl_a();
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), false);
    }

    #[test]
    fn test_asl_shift_to_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        for _i in 0..8 {
            cpu.asl_a();
        }
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_slo_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0x08);
        cpu.register_a = 0x20;
        cpu.slo_zp(0x10);
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.memory.read_byte(0x10), 0x10);
    }

    #[test]
    fn test_slo_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, 0x08);
        cpu.register_a = 0x20;
        cpu.register_x = 0x10;
        cpu.slo_zp_x(0x10);
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.memory.read_byte(0x20), 0x10);
    }

    #[test]
    fn test_slo_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 0x08);
        cpu.register_a = 0x20;
        cpu.slo_ab(0x1400);
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.memory.read_byte(0x1400), 0x10);
    }

    #[test]
    fn test_slo_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0x08);
        cpu.register_a = 0x20;
        cpu.register_x = 0x10;
        cpu.slo_ab_x(0x1400);
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.memory.read_byte(0x1410), 0x10);
    }

    #[test]
    fn test_slo_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0x08);
        cpu.register_a = 0x20;
        cpu.register_y = 0x10;
        cpu.slo_ab_y(0x1400);
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.memory.read_byte(0x1410), 0x10);
    }

    #[test]
    fn test_slo_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 0x08);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = 0x20;
        cpu.register_x = 0x08;
        cpu.slo_in_x(0x08);
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.memory.read_byte(0x1400), 0x10);
    }

    #[test]
    fn test_slo_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0x08);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = 0x20;
        cpu.register_y = 0x10;
        cpu.slo_in_y(0x10);
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.memory.read_byte(0x1410), 0x10);
    }

    /* Rotate */

    #[test]
    fn test_ror_a() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0b0000_1111;
        cpu.ror_a();
        assert_eq!(cpu.register_a, 0b1000_0111);
    }

    #[test]
    fn test_ror_zp() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x10, 0b0000_1111);
        cpu.ror_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), 0b1000_0111);
    }

    #[test]
    fn test_ror_zp_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x20, 0b0000_1111);
        cpu.register_x = 0x10;
        cpu.ror_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), 0b1000_0111);
    }

    #[test]
    fn test_ror_ab() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x1400, 0b0000_1111);
        cpu.ror_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), 0b1000_0111);
    }

    #[test]
    fn test_ror_ab_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x1410, 0b0000_1111);
        cpu.register_x = 0x10;
        cpu.ror_ab_x(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), 0b1000_0111);
    }

    #[test]
    fn test_ror_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 1;
        cpu.ror_a();
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_ror_negative() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.ror_a();
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), false);
    }

    #[test]
    fn test_ror_wraparound() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0b0000_1111;
        for _i in 0..9 {
            cpu.ror_a();
        }
        assert_eq!(cpu.register_a, 0b0000_1111);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_rra_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 1);
        cpu.register_a = BYTE_A;
        cpu.rra_zp(0x10);
        assert_eq!(cpu.register_a, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x10), 0);
    }

    #[test]
    fn test_rra_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, 1);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.rra_zp_x(0x10);
        assert_eq!(cpu.register_a, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x20), 0);
    }

    #[test]
    fn test_rra_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 1);
        cpu.register_a = BYTE_A;
        cpu.rra_ab(0x1400);
        assert_eq!(cpu.register_a, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x1400), 0);
    }

    #[test]
    fn test_rra_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 1);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.rra_ab_x(0x1400);
        assert_eq!(cpu.register_a, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x1410), 0);
    }

    #[test]
    fn test_rra_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 1);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.rra_ab_y(0x1400);
        assert_eq!(cpu.register_a, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x1410), 0);
    }

    #[test]
    fn test_rra_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 1);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x08;
        cpu.rra_in_x(0x08);
        assert_eq!(cpu.register_a, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x1400), 0);
    }

    #[test]
    fn test_rra_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 1);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.rra_in_y(0x10);
        assert_eq!(cpu.register_a, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x1410), 0);
    }

    #[test]
    fn test_rol_a() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0b0000_1111;
        cpu.rol_a();
        assert_eq!(cpu.register_a, 0b0001_1111);
    }

    #[test]
    fn test_rol_zp() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x10, 0b0000_1111);
        cpu.rol_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), 0b0001_1111);
    }

    #[test]
    fn test_rol_zp_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x20, 0b0000_1111);
        cpu.register_x = 0x10;
        cpu.rol_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), 0b0001_1111);
    }

    #[test]
    fn test_rol_ab() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x1400, 0b0000_1111);
        cpu.rol_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), 0b0001_1111);
    }

    #[test]
    fn test_rol_ab_x() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.memory.write_byte(0x1410, 0b0000_1111);
        cpu.register_x = 0x10;
        cpu.rol_ab_x(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), 0b0001_1111);
    }

    #[test]
    fn test_rol_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x80;
        cpu.rol_a();
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_rol_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x40;
        cpu.rol_a();
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), false);
    }

    #[test]
    fn test_rol_wraparound() {
        let mut cpu = CPU::new();
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = 0b0000_1111;
        for _i in 0..9 {
            cpu.rol_a();
        }
        assert_eq!(cpu.register_a, 0b0000_1111);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_rla_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, 0x05);
        cpu.register_a = BYTE_A;
        cpu.rla_zp(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_A);
    }

    #[test]
    fn test_rla_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, 0x05);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.rla_zp_x(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_A);
    }

    #[test]
    fn test_rla_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 0x05);
        cpu.register_a = BYTE_A;
        cpu.rla_ab(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    #[test]
    fn test_rla_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0x05);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.rla_ab_x(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
    }

    #[test]
    fn test_rla_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0x05);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.rla_ab_y(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
    }

    #[test]
    fn test_rla_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, 0x05);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x08;
        cpu.rla_in_x(0x08);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    #[test]
    fn test_rla_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, 0x05);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.rla_in_y(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
    }

    /* Load */

    #[test]
    fn test_lda_im() {
        let mut cpu = CPU::new();
        cpu.lda_im(BYTE_A);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_lda_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.lda_zp(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_lda_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.register_x = 0x08;
        cpu.lda_zp_x(0x08);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_lda_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.lda_ab(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_lda_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_x = 0x10;
        cpu.lda_ab_x(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_lda_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_y = 0x10;
        cpu.lda_ab_y(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_lda_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_x = 0x08;
        cpu.lda_in_x(0x08);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_lda_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_y = 0x10;
        cpu.lda_in_y(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
    }

    #[test]
    fn test_ldx_im() {
        let mut cpu = CPU::new();
        cpu.ldx_im(BYTE_A);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_ldx_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.ldx_zp(0x10);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_ldx_zp_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, BYTE_A);
        cpu.register_y = 0x10;
        cpu.ldx_zp_y(0x10);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_ldx_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.ldx_ab(0x1400);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_ldx_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_y = 0x10;
        cpu.ldx_ab_y(0x1400);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_ldy_im() {
        let mut cpu = CPU::new();
        cpu.ldy_im(BYTE_A);
        assert_eq!(cpu.register_y, BYTE_A);
    }

    #[test]
    fn test_ldy_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.ldy_zp(0x10);
        assert_eq!(cpu.register_y, BYTE_A);
    }

    #[test]
    fn test_ldy_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, BYTE_A);
        cpu.register_x = 0x10;
        cpu.ldy_zp_x(0x10);
        assert_eq!(cpu.register_y, BYTE_A);
    }

    #[test]
    fn test_ldy_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.ldy_ab(0x1400);
        assert_eq!(cpu.register_y, BYTE_A);
    }

    #[test]
    fn test_ldy_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_x = 0x10;
        cpu.ldy_ab_x(0x1400);
        assert_eq!(cpu.register_y, BYTE_A);
    }

    #[test]
    fn test_lax_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.lax_zp(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_lax_zp_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, BYTE_A);
        cpu.register_y = 0x10;
        cpu.lax_zp_y(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_lax_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.lax_ab(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_lax_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_y = 0x10;
        cpu.lax_ab_y(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_lax_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_x = 0x08;
        cpu.lax_in_x(0x08);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_lax_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_y = 0x10;
        cpu.lax_in_y(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_A);
    }

    #[test]
    fn test_load_zero() {
        let mut cpu = CPU::new();
        cpu.lda_im(0);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true)
    }

    #[test]
    fn test_load_negative() {
        let mut cpu = CPU::new();
        cpu.lda_im(0xff);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true)
    }

    /* Store */

    #[test]
    fn test_sta_zp() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.sta_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_A);
    }

    #[test]
    fn test_sta_zp_x() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.sta_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_A);
    }

    #[test]
    fn test_sta_ab() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.sta_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    #[test]
    fn test_sta_ab_x() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.sta_ab_x(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
    }

    #[test]
    fn test_sta_ab_y() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.sta_ab_y(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
    }

    #[test]
    fn test_sta_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x20, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.sta_in_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    #[test]
    fn test_sta_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.sta_in_y(0x10);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
    }

    #[test]
    fn test_stx_zp() {
        let mut cpu = CPU::new();
        cpu.register_x = BYTE_A;
        cpu.stx_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_A);
    }

    #[test]
    fn test_stx_zp_y() {
        let mut cpu = CPU::new();
        cpu.register_x = BYTE_A;
        cpu.register_y = 0x10;
        cpu.stx_zp_y(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_A);
    }

    #[test]
    fn test_stx_ab() {
        let mut cpu = CPU::new();
        cpu.register_x = BYTE_A;
        cpu.stx_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    #[test]
    fn test_sty_zp() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.sty_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_A);
    }

    #[test]
    fn test_sty_zp_x() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.register_x = 0x10;
        cpu.sty_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_A);
    }

    #[test]
    fn test_sty_ab() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.sty_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    #[test]
    fn test_sax_zp() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_x = BYTE_B;
        cpu.sax_zp(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_A);
    }

    #[test]
    fn test_sax_zp_y() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_x = BYTE_B;
        cpu.register_y = 0x10;
        cpu.sax_zp_y(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_A);
    }

    #[test]
    fn test_sax_ab() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_x = BYTE_B;
        cpu.sax_ab(0x1400);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    #[test]
    fn test_sax_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x1b, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_x = BYTE_B;
        cpu.sax_in_x(0x10);
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, BYTE_B);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    /* Transfer */

    #[test]
    fn test_tax() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_x = BYTE_B;
        cpu.tax();
        assert_eq!(cpu.register_x, BYTE_A);
        assert_eq!(cpu.register_a, cpu.register_x);
    }

    #[test]
    fn test_tay() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.register_y = BYTE_B;
        cpu.tay();
        assert_eq!(cpu.register_y, BYTE_A);
        assert_eq!(cpu.register_a, cpu.register_y);
    }

    #[test]
    fn test_tsx() {
        let mut cpu = CPU::new();
        cpu.stack = BYTE_A;
        cpu.register_x = BYTE_B;
        cpu.tsx();
        assert_eq!(cpu.register_x, BYTE_A);
        assert_eq!(cpu.stack, cpu.register_x);
    }

    #[test]
    fn test_txa() {
        let mut cpu = CPU::new();
        cpu.register_x = BYTE_A;
        cpu.register_a = BYTE_B;
        cpu.txa();
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_x, cpu.register_a);
    }

    #[test]
    fn test_txs() {
        let mut cpu = CPU::new();
        cpu.register_x = BYTE_A;
        cpu.stack = BYTE_B;
        cpu.txs();
        assert_eq!(cpu.stack, BYTE_A);
        assert_eq!(cpu.register_x, cpu.stack);
    }

    #[test]
    fn test_tya() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.register_a = BYTE_B;
        cpu.tya();
        assert_eq!(cpu.register_a, BYTE_A);
        assert_eq!(cpu.register_y, cpu.register_a);
    }

    #[test]
    fn test_transfer_zero() {
        let mut cpu = CPU::new();
        cpu.tax();
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true)
    }

    #[test]
    fn test_transfer_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xff;
        cpu.tax();
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true)
    }

    /* Increment */

    #[test]
    fn test_inc_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.inc_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_B);
    }

    #[test]
    fn test_inc_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, BYTE_A);
        cpu.register_x = 0x10;
        cpu.inc_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_B);
    }

    #[test]
    fn test_inc_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.inc_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_B);
    }

    #[test]
    fn test_inc_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_x = 0x10;
        cpu.inc_ab_x(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_B);
    }

    #[test]
    fn test_inx() {
        let mut cpu = CPU::new();
        cpu.inx();
        assert_eq!(cpu.register_x, 1);
        cpu.inx();
        assert_eq!(cpu.register_x, 2);
    }

    #[test]
    fn test_iny() {
        let mut cpu = CPU::new();
        cpu.iny();
        assert_eq!(cpu.register_y, 1);
        cpu.iny();
        assert_eq!(cpu.register_y, 2);
    }

    #[test]
    fn test_isb_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.isb_zp(0x10);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_B);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_isb_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, BYTE_A);
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.register_x = 0x10;
        cpu.isb_zp_x(0x10);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_B);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_isb_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.isb_ab(0x1400);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_B);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_isb_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.register_x = 0x10;
        cpu.isb_ab_x(0x1400);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_B);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_isb_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.register_y = 0x10;
        cpu.isb_ab_y(0x1400);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_B);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_isb_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.register_x = 0x08;
        cpu.isb_in_x(0x08);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_B);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_isb_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.set_status_flag(CARRY_FLAG);
        cpu.register_a = BYTE_B;
        cpu.register_y = 0x10;
        cpu.isb_in_y(0x10);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_B);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_increment_zero() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.inx();
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true)
    }

    #[test]
    fn test_increment_negative() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xfe;
        cpu.inx();
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true)
    }

    /* Decrement */

    #[test]
    fn test_dec_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_B);
        cpu.dec_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_A);
    }

    #[test]
    fn test_dec_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, BYTE_B);
        cpu.register_x = 0x10;
        cpu.dec_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_A);
    }

    #[test]
    fn test_dec_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_B);
        cpu.dec_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
    }

    #[test]
    fn test_dec_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_B);
        cpu.register_x = 0x10;
        cpu.dec_ab_x(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
    }

    #[test]
    fn test_dex() {
        let mut cpu = CPU::new();
        cpu.register_x = 2;
        cpu.dex();
        assert_eq!(cpu.register_x, 1);
        cpu.dex();
        assert_eq!(cpu.register_x, 0);
    }

    #[test]
    fn test_dey() {
        let mut cpu = CPU::new();
        cpu.register_y = 2;
        cpu.dey();
        assert_eq!(cpu.register_y, 1);
        cpu.dey();
        assert_eq!(cpu.register_y, 0);
    }

    #[test]
    fn test_dcp_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_B);
        cpu.register_a = BYTE_A;
        cpu.dcp_zp(0x10);
        assert_eq!(cpu.memory.read_byte(0x10), BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_dcp_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x20, BYTE_B);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.dcp_zp_x(0x10);
        assert_eq!(cpu.memory.read_byte(0x20), BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_dcp_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_B);
        cpu.register_a = BYTE_A;
        cpu.dcp_ab(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_dcp_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_B);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.dcp_ab_x(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_dcp_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_B);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.dcp_ab_y(0x1400);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_dcp_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_B);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x08;
        cpu.dcp_in_x(0x08);
        assert_eq!(cpu.memory.read_byte(0x1400), BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_dcp_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_B);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.dcp_in_y(0x10);
        assert_eq!(cpu.memory.read_byte(0x1410), BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_decrement_zero() {
        let mut cpu = CPU::new();
        cpu.register_x = 1;
        cpu.dex();
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true)
    }

    #[test]
    fn test_decrement_negative() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.dex();
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true)
    }

    /* Compare */

    #[test]
    fn test_cmp_im() {
        let mut cpu = CPU::new();
        cpu.register_a = BYTE_A;
        cpu.cmp_im(BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cmp_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.register_a = BYTE_A;
        cpu.cmp_zp(0x10);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cmp_zp_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x08;
        cpu.cmp_zp_x(0x08);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cmp_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.register_a = BYTE_A;
        cpu.cmp_ab(0x1400);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cmp_ab_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x10;
        cpu.cmp_ab_x(0x1400);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cmp_ab_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.cmp_ab_y(0x1400);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cmp_in_x() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_x = 0x08;
        cpu.cmp_in_x(0x08);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cmp_in_y() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1410, BYTE_A);
        cpu.memory.write_addr(0x10, 0x1400);
        cpu.register_a = BYTE_A;
        cpu.register_y = 0x10;
        cpu.cmp_in_y(0x10);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cpx_im() {
        let mut cpu = CPU::new();
        cpu.register_x = BYTE_A;
        cpu.cpx_im(BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cpx_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.register_x = BYTE_A;
        cpu.cpx_zp(0x10);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cpx_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.register_x = BYTE_A;
        cpu.cpx_ab(0x1400);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cpy_im() {
        let mut cpu = CPU::new();
        cpu.register_y = BYTE_A;
        cpu.cpy_im(BYTE_A);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cpy_zp() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x10, BYTE_A);
        cpu.register_y = BYTE_A;
        cpu.cpy_zp(0x10);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_cpy_ab() {
        let mut cpu = CPU::new();
        cpu.memory.write_byte(0x1400, BYTE_A);
        cpu.register_y = BYTE_A;
        cpu.cpy_ab(0x1400);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_compare_same() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x20;
        cpu.cmp_im(0x20);
        assert_eq!(cpu.get_status_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_compare_greater() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x20;
        cpu.cmp_im(0x10);
        assert_eq!(cpu.get_status_flag(CARRY_FLAG), true);
    }

    #[test]
    fn test_compare_lesser() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x20;
        cpu.cmp_im(0x30);
        assert_eq!(cpu.get_status_flag(NEGATIVE_FLAG), true);
    }

    /* Jump & Branch */

    #[test]
    fn test_jmp_ab() {
        let mut cpu = CPU::new();
        cpu.jmp_ab(0x1400);
        assert_eq!(cpu.program_counter, 0x1400);
    }

    #[test]
    fn test_jmp_in() {
        let mut cpu = CPU::new();
        cpu.memory.write_addr(0x1400, 0x2000);
        cpu.jmp_in(0x1400);
        assert_eq!(cpu.program_counter, 0x2000);
    }

    #[test]
    fn test_jsr() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x1234;
        cpu.jsr(0x2000);
        assert_eq!(cpu.program_counter, 0x2000);
        assert_eq!(cpu.stack, 0xfd);
        assert_eq!(cpu.memory.read_addr(0x01fe), 0x1234);
    }

    #[test]
    fn test_rts() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x1234;
        cpu.jsr(0x2000);
        cpu.rts();
        assert_eq!(cpu.program_counter, 0x1234 + 1);
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.memory.read_addr(0x01fe), 0x1234);
    }

    #[test]
    fn test_rti() {
        let mut cpu = CPU::new();
        cpu.status = 0b1011_1010;
        cpu.program_counter = 0x1234;
        cpu.push_addr(cpu.program_counter);
        cpu.push_byte(cpu.status);
        cpu.rti();
        assert_eq!(cpu.status, 0b1010_1010);
        assert_eq!(cpu.program_counter, 0x1234);
        assert_eq!(cpu.stack, 0xff);
        assert_eq!(cpu.memory.read_byte(0x01fd), 0b1011_1010);
        assert_eq!(cpu.memory.read_addr(0x01fe), 0x1234);
    }

    #[test]
    fn test_beq() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.set_status_flag(ZERO_FLAG);
        cpu.beq(0x10);
        assert_eq!(cpu.program_counter, 0x90 + 1);
    }

    #[test]
    fn test_bne() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.clear_status_flag(ZERO_FLAG);
        cpu.bne(0x10);
        assert_eq!(cpu.program_counter, 0x90 + 1);
    }

    #[test]
    fn test_bcs() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.set_status_flag(CARRY_FLAG);
        cpu.bcs(0x10);
        assert_eq!(cpu.program_counter, 0x90 + 1);
    }

    #[test]
    fn test_bcc() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.clear_status_flag(CARRY_FLAG);
        cpu.bcc(0x10);
        assert_eq!(cpu.program_counter, 0x90 + 1);
    }

    #[test]
    fn test_bmi() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.set_status_flag(NEGATIVE_FLAG);
        cpu.bmi(0x10);
        assert_eq!(cpu.program_counter, 0x90 + 1);
    }

    #[test]
    fn test_bpl() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.clear_status_flag(NEGATIVE_FLAG);
        cpu.bpl(0x10);
        assert_eq!(cpu.program_counter, 0x90 + 1);
    }

    #[test]
    fn test_bvs() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.set_status_flag(OVERFLOW_FLAG);
        cpu.bvs(0x10);
        assert_eq!(cpu.program_counter, 0x90 + 1);
    }

    #[test]
    fn test_bvc() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.clear_status_flag(OVERFLOW_FLAG);
        cpu.bvc(0x10);
        assert_eq!(cpu.program_counter, 0x90 + 1);
    }

    #[test]
    fn test_branch_zero_offset() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.set_status_flag(ZERO_FLAG);
        cpu.beq(0);
        assert_eq!(cpu.program_counter, 0x80 + 1);
    }

    #[test]
    fn test_branch_negative_offset() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.set_status_flag(ZERO_FLAG);
        cpu.beq(-0x10);
        assert_eq!(cpu.program_counter, 0x70 + 1);
    }

    /* NOP */

    #[test]
    fn test_nop() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.nop();
        assert_eq!(cpu.program_counter, 0x81);
    }

    #[test]
    fn test_dop() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.dop();
        assert_eq!(cpu.program_counter, 0x82);
    }

    #[test]
    fn test_top() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x80;
        cpu.top();
        assert_eq!(cpu.program_counter, 0x83);
    }
}