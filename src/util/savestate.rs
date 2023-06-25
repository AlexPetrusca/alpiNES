use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::ExactSizeIterator;
use std::path::Path;
use std::convert::TryFrom;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use serde_cbor::Value;
use crate::nes::NES;
use crate::nes::cpu::CPU;
use crate::nes::ppu::PPU;
use crate::nes::rom::{Mirroring, ROM};
use crate::nes::rom::mappers::mapper0::Mapper0;
use crate::nes::rom::mappers::mapper1::Mapper1;
use crate::nes::rom::mappers::mapper2::Mapper2;
use crate::nes::rom::mappers::mapper3::Mapper3;
use crate::nes::rom::mappers::mapper4::Mapper4;
use crate::nes::rom::mappers::mapper66::Mapper66;
use crate::{custom_ram_range, palletes_ram_range, prg_ram_range, ram_range, vram_range};

#[derive(Serialize, Deserialize, Debug)]
pub struct CPUState {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub stack: u8,
    pub status: u8,
    pub program_counter: u16,

    pub ram: Vec<u8>,
    pub custom_ram: Vec<u8>,
    pub prg_ram: Vec<u8>,

    pub cycles: usize,
}

impl CPUState {
    pub fn new(cpu: &CPU) -> Self {
        CPUState {
            register_a: cpu.register_a,
            register_x: cpu.register_x,
            register_y: cpu.register_y,
            stack: cpu.stack,
            status: cpu.status.value,
            program_counter: cpu.program_counter,
            ram: cpu.memory.memory[ram_range!()].to_vec(),
            custom_ram: cpu.memory.memory[custom_ram_range!()].to_vec(),
            prg_ram: cpu.memory.memory[prg_ram_range!()].to_vec(),
            cycles: cpu.cycles
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PPUState {
    pub addr: u16,
    pub addr_latch: bool,
    pub data: u8,
    pub ctrl: u8,
    pub status: u8,
    pub mask: u8,
    pub scroll: u16,
    pub scroll_latch: bool,
    pub oam_addr: u8,
    pub oam_data: u8,

    pub vram: Vec<u8>,
    pub palletes_ram: Vec<u8>,
    pub oam: Vec<u8>,
    pub scroll_ctx_v: u16,
    pub scroll_ctx_t: u16,
    pub scroll_ctx_x: u8,
    pub scroll_ctx_w: bool,
    pub data_buffer: u8,

    pub cycles: usize,
    pub scanline: isize,
    pub nmi_flag: bool,
}

impl PPUState {
    pub fn new(ppu: &PPU) -> Self {
        PPUState {
            addr: ppu.addr.get(),
            addr_latch: ppu.addr.latch,
            data: ppu.data,
            ctrl: ppu.ctrl.value,
            status: ppu.status.value,
            mask: ppu.mask.value,
            scroll: ppu.scroll.get(),
            scroll_latch: ppu.scroll.latch,
            oam_addr: ppu.oam_addr,
            oam_data: ppu.oam_data,

            vram: ppu.memory.memory[vram_range!()].to_vec(),
            palletes_ram: ppu.memory.memory[palletes_ram_range!()].to_vec(),
            oam: ppu.oam.memory.to_vec(),
            scroll_ctx_v: ppu.scroll_ctx.v,
            scroll_ctx_t: ppu.scroll_ctx.t,
            scroll_ctx_x: ppu.scroll_ctx.x,
            scroll_ctx_w: ppu.scroll_ctx.w,
            data_buffer: ppu.data_buffer,

            cycles: ppu.cycles,
            scanline: ppu.scanline,
            nmi_flag: ppu.nmi_flag,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ROMState {
    pub chr_ram: Option<Vec<u8>>,
    pub mapper1: Mapper1State,
    pub mapper2: Mapper2State,
    pub mapper3: Mapper3State,
    pub mapper4: Mapper4State,
    pub mapper66: Mapper66State,
}

impl ROMState {
    pub fn new(cpu_rom: &ROM, ppu_rom: &ROM) -> Self {
        ROMState {
            chr_ram: if ppu_rom.is_chr_ram { Some(ppu_rom.chr_rom.to_vec()) } else { None },
            mapper1: Mapper1State::new(&cpu_rom.mapper1),
            mapper2: Mapper2State::new(&cpu_rom.mapper2),
            mapper3: Mapper3State::new(&cpu_rom.mapper3),
            mapper4: Mapper4State::new(&cpu_rom.mapper4),
            mapper66: Mapper66State::new(&cpu_rom.mapper66),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper1State {
    pub shift_reg_value: u8,
    pub shift_reg_shift: u8,
    pub prg_bank_select_mode: u8,
    pub chr_bank_select_mode: u8,
    pub prg_bank_select: u8,
    pub chr_bank_select: u8,
    pub chr_bank0_select: u8,
    pub chr_bank1_select: u8,
    pub screen_mirroring: Mirroring,
}

impl Mapper1State {
    pub fn new(mapper1: &Mapper1) -> Self {
        Mapper1State {
            shift_reg_value: mapper1.shift_register.value,
            shift_reg_shift: mapper1.shift_register.shift,
            prg_bank_select_mode: mapper1.prg_bank_select_mode,
            chr_bank_select_mode: mapper1.chr_bank_select_mode,
            prg_bank_select: mapper1.prg_bank_select,
            chr_bank_select: mapper1.chr_bank_select,
            chr_bank0_select: mapper1.chr_bank0_select,
            chr_bank1_select: mapper1.chr_bank1_select,
            screen_mirroring: mapper1.screen_mirroring.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper2State {
    pub prg_bank_select: u8,
}

impl Mapper2State {
    pub fn new(mapper2: &Mapper2) -> Self {
        Mapper2State {
            prg_bank_select: mapper2.prg_bank_select,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper3State {
    pub chr_bank_select: u8,
}

impl Mapper3State {
    pub fn new(mapper3: &Mapper3) -> Self {
        Mapper3State {
            chr_bank_select: mapper3.chr_bank_select,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper4State {
    pub bank_select: u8,
    pub prg_bank_select_mode: u8,
    pub chr_bank_select_mode: u8,
    pub prg_bank0_select:u8,
    pub prg_bank1_select:u8,
    pub chr_bank0_select: u8,
    pub chr_bank1_select: u8,
    pub chr_bank0_1kb_select: u8,
    pub chr_bank1_1kb_select: u8,
    pub chr_bank2_1kb_select: u8,
    pub chr_bank3_1kb_select: u8,
    pub chr_bank0_2kb_select: u8,
    pub chr_bank1_2kb_select: u8,
    pub screen_mirroring: Mirroring,
    pub irq_counter: Option<u8>,
    pub irq_latch: Option<u8>,
    pub irq_reload: Option<bool>,
    pub irq_enable: Option<bool>,
    pub irq_flag: Option<bool>,
}

impl Mapper4State {
    pub fn new(mapper4: &Mapper4) -> Self {
        Mapper4State {
            bank_select: mapper4.bank_select,
            prg_bank_select_mode: mapper4.prg_bank_select_mode,
            chr_bank_select_mode: mapper4.chr_bank_select_mode,
            prg_bank0_select: mapper4.prg_bank0_select,
            prg_bank1_select: mapper4.prg_bank1_select,
            chr_bank0_select: mapper4.chr_bank0_select,
            chr_bank1_select: mapper4.chr_bank1_select,
            chr_bank0_1kb_select: mapper4.chr_bank0_1kb_select,
            chr_bank1_1kb_select: mapper4.chr_bank1_1kb_select,
            chr_bank2_1kb_select: mapper4.chr_bank2_1kb_select,
            chr_bank3_1kb_select: mapper4.chr_bank3_1kb_select,
            chr_bank0_2kb_select: mapper4.chr_bank0_2kb_select,
            chr_bank1_2kb_select: mapper4.chr_bank1_2kb_select,
            screen_mirroring: mapper4.screen_mirroring.clone(),
            irq_counter: Some(mapper4.irq_counter),
            irq_latch: Some(mapper4.irq_latch),
            irq_reload: Some(mapper4.irq_reload),
            irq_enable: Some(mapper4.irq_enable),
            irq_flag: Some(mapper4.irq_flag),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper66State {
    pub prg_bank_select: u8,
    pub chr_bank_select: u8,
}

impl Mapper66State {
    pub fn new(mapper66: &Mapper66) -> Self {
        Mapper66State {
            prg_bank_select: mapper66.prg_bank_select,
            chr_bank_select: mapper66.chr_bank_select,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveState {
    pub cpu_state: CPUState,
    pub ppu_state: PPUState,
    pub rom_state: ROMState,
}

impl SaveState {
    pub fn new(nes: &NES) -> Self {
        SaveState {
            cpu_state: CPUState::new(&nes.cpu),
            ppu_state: PPUState::new(&nes.cpu.memory.ppu),
            rom_state: ROMState::new(&nes.cpu.memory.rom, &nes.cpu.memory.ppu.memory.rom),
        }
    }

    pub fn deserialize(path: &Path) -> Option<SaveState> {
        if path.exists() {
            let mut save_file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let save_state = serde_cbor::from_reader(save_file).expect("unable to load savestate file");
            return Some(save_state);
        }
        return None;
    }

    pub fn serialize(path: &Path, save_state: &SaveState) {
        let prefix_path = path.parent().unwrap();
        fs::create_dir_all(prefix_path).unwrap();

        let mut save_file = File::create(path).expect("unable to create savestate file");
        serde_cbor::to_writer(save_file, save_state).expect("unable to write to savestate file");
    }

    pub fn load_nes_state(nes: &mut NES, save_state: &SaveState) {
        let cpu_state = &save_state.cpu_state;
        Self::load_cpu_state(&mut nes.cpu, cpu_state);

        let ppu_state = &save_state.ppu_state;
        Self::load_ppu_state(&mut nes.cpu.memory.ppu, ppu_state);

        // todo: [FEATURE] add apu restore for savestates

        let rom_state = &save_state.rom_state;
        Self::load_rom_state(&mut nes.cpu.memory.rom, rom_state);
        Self::load_rom_state(&mut nes.cpu.memory.ppu.memory.rom, rom_state);
    }

    fn load_cpu_state(cpu: &mut CPU, cpu_state: &CPUState) {
        cpu.register_a = cpu_state.register_a;
        cpu.register_x = cpu_state.register_x;
        cpu.register_y = cpu_state.register_y;
        cpu.stack = cpu_state.stack;
        cpu.status.value = cpu_state.status;
        cpu.program_counter = cpu_state.program_counter;
        cpu.memory.memory[ram_range!()].copy_from_slice(cpu_state.ram.as_slice());
        cpu.memory.memory[custom_ram_range!()].copy_from_slice(cpu_state.custom_ram.as_slice());
        // todo: [BUG] Need to also restore battery.sav file on load savestate
        cpu.memory.memory[prg_ram_range!()].copy_from_slice(cpu_state.prg_ram.as_slice());
        cpu.cycles = cpu_state.cycles;
    }

    fn load_ppu_state(ppu: &mut PPU, ppu_state: &PPUState) {
        ppu.addr.set(ppu_state.addr);
        ppu.addr.latch = ppu_state.addr_latch;
        ppu.data = ppu_state.data;
        ppu.ctrl.set_value(ppu_state.ctrl);
        ppu.status.set_value(ppu_state.status);
        ppu.mask.set_value(ppu_state.mask);
        ppu.scroll.set(ppu_state.scroll);
        ppu.scroll.latch = ppu_state.scroll_latch;
        ppu.oam_addr = ppu_state.oam_addr;
        ppu.oam_data = ppu_state.oam_data;
        ppu.memory.memory[vram_range!()].copy_from_slice(ppu_state.vram.as_slice());
        ppu.memory.memory[palletes_ram_range!()].copy_from_slice(ppu_state.palletes_ram.as_slice());
        ppu.oam.memory.copy_from_slice(ppu_state.oam.as_slice());
        ppu.scroll_ctx.v = ppu_state.scroll_ctx_v;
        ppu.scroll_ctx.t = ppu_state.scroll_ctx_t;
        ppu.scroll_ctx.x = ppu_state.scroll_ctx_x;
        ppu.scroll_ctx.w = ppu_state.scroll_ctx_w;
        ppu.data_buffer = ppu_state.data_buffer;
        ppu.scanline = ppu_state.scanline;
        ppu.cycles = ppu_state.cycles;
        ppu.nmi_flag = ppu_state.nmi_flag;
    }

    fn load_rom_state(rom: &mut ROM, rom_state: &ROMState) {
        if let Some(chr_ram) = &rom_state.chr_ram {
            rom.chr_rom.copy_from_slice(chr_ram.as_slice());
        }
        match rom.mapper_id {
            1 => {
                rom.mapper1.shift_register.value = rom_state.mapper1.shift_reg_value;
                rom.mapper1.shift_register.shift = rom_state.mapper1.shift_reg_shift;
                rom.mapper1.prg_bank_select_mode = rom_state.mapper1.prg_bank_select_mode;
                rom.mapper1.chr_bank_select_mode = rom_state.mapper1.chr_bank_select_mode;
                rom.mapper1.prg_bank_select = rom_state.mapper1.prg_bank_select;
                rom.mapper1.chr_bank_select = rom_state.mapper1.chr_bank_select;
                rom.mapper1.chr_bank0_select = rom_state.mapper1.chr_bank0_select;
                rom.mapper1.chr_bank1_select = rom_state.mapper1.chr_bank1_select;
                rom.mapper1.screen_mirroring = rom_state.mapper1.screen_mirroring.clone();
                rom.screen_mirroring = rom_state.mapper1.screen_mirroring.clone();
            },
            2 => {
                rom.mapper2.prg_bank_select = rom_state.mapper2.prg_bank_select;
            },
            3 => {
                rom.mapper3.chr_bank_select = rom_state.mapper3.chr_bank_select;
            },
            4 => {
                rom.mapper4.bank_select = rom_state.mapper4.bank_select;
                rom.mapper4.prg_bank_select_mode = rom_state.mapper4.prg_bank_select_mode;
                rom.mapper4.chr_bank_select_mode = rom_state.mapper4.chr_bank_select_mode;
                rom.mapper4.prg_bank0_select = rom_state.mapper4.prg_bank0_select;
                rom.mapper4.prg_bank1_select = rom_state.mapper4.prg_bank1_select;
                rom.mapper4.chr_bank0_select = rom_state.mapper4.chr_bank0_select;
                rom.mapper4.chr_bank1_select = rom_state.mapper4.chr_bank1_select;
                rom.mapper4.chr_bank0_1kb_select = rom_state.mapper4.chr_bank0_1kb_select;
                rom.mapper4.chr_bank1_1kb_select = rom_state.mapper4.chr_bank1_1kb_select;
                rom.mapper4.chr_bank2_1kb_select = rom_state.mapper4.chr_bank2_1kb_select;
                rom.mapper4.chr_bank3_1kb_select = rom_state.mapper4.chr_bank3_1kb_select;
                rom.mapper4.chr_bank0_2kb_select = rom_state.mapper4.chr_bank0_2kb_select;
                rom.mapper4.chr_bank1_2kb_select = rom_state.mapper4.chr_bank1_2kb_select;
                rom.mapper4.screen_mirroring = rom_state.mapper4.screen_mirroring.clone();
                rom.screen_mirroring = rom_state.mapper4.screen_mirroring.clone();
                rom.mapper4.irq_counter = rom_state.mapper4.irq_counter.unwrap_or(0);
                rom.mapper4.irq_latch = rom_state.mapper4.irq_latch.unwrap_or(0);
                rom.mapper4.irq_reload = rom_state.mapper4.irq_reload.unwrap_or(false);
                rom.mapper4.irq_enable = rom_state.mapper4.irq_enable.unwrap_or(false);
                rom.mapper4.irq_flag = rom_state.mapper4.irq_flag.unwrap_or(false);
            },
            66 => {
                rom.mapper66.prg_bank_select = rom_state.mapper66.prg_bank_select;
                rom.mapper66.chr_bank_select = rom_state.mapper66.chr_bank_select;
            },
            _ => panic!("save state for mapper is not supported: mapper {}", rom.mapper_id)
        }
    }
}

