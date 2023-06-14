use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::ExactSizeIterator;
use std::path::Path;
use std::convert::TryFrom;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::nes::NES;
use crate::nes::cpu::CPU;
use crate::nes::ppu::PPU;
use crate::{custom_ram_range, palletes_ram_range, prg_ram_range, ram_range, vram_range};
use crate::nes::ppu::registers::addr::AddressRegister;
use crate::nes::ppu::registers::ctrl::ControlRegister;
use crate::nes::ppu::registers::mask::MaskRegister;
use crate::nes::ppu::registers::scroll::ScrollRegister;
use crate::nes::ppu::registers::status::StatusRegister;

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
            status: cpu.status,
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
    pub data_buffer: u8,

    pub cycles: usize,
    pub scanline: u16,
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
            data_buffer: ppu.data_buffer,

            cycles: ppu.cycles,
            scanline: ppu.scanline,
            nmi_flag: ppu.nmi_flag,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveState {
    pub cpu_state: CPUState,
    pub ppu_state: PPUState,
}

impl SaveState {
    pub fn new(nes: &NES) -> Self {
        SaveState {
            cpu_state: CPUState::new(&nes.cpu),
            ppu_state: PPUState::new(&nes.cpu.memory.ppu)
        }
    }

    pub fn deserialize(path: &Path) -> Option<SaveState> {
        if path.exists() {
            let mut save_file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let save_state = serde_json::from_reader(save_file).expect("unable to load savestate file");
            return Some(save_state);
        }
        return None;
    }

    pub fn serialize(path: &Path, save_state: &SaveState) {
        let json_string = serde_json::to_string(&save_state).expect("Couldn't stringify json");

        let prefix_path = path.parent().unwrap();
        fs::create_dir_all(prefix_path).unwrap();

        let mut save_file = File::create(path).expect("unable to create savestate file");
        save_file.write(json_string.as_bytes()).expect("unable to write to savestate file");
    }
}

