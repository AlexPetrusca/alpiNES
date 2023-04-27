use crate::nes::apu::registers::frame_counter::FrameCounterRegister;
use crate::nes::apu::registers::dmc::DMCRegisters;
use crate::nes::apu::registers::noise::NoiseRegisters;
use crate::nes::apu::registers::pulse::PulseRegisters;
use crate::nes::apu::registers::status::StatusFlag::{FrameInterrupt, PulseOneEnable, PulseTwoEnable};
use crate::nes::apu::registers::status::StatusRegister;
use crate::nes::apu::registers::triangle::TriangleRegisters;
use crate::util::bitvec::BitVector;

pub mod registers;

pub struct APU {
    pub pulse_one: PulseRegisters,
    pub pulse_two: PulseRegisters,
    pub triangle: TriangleRegisters,
    pub noise: NoiseRegisters,
    pub dmc: DMCRegisters,

    pub status: StatusRegister,
    pub frame_counter: FrameCounterRegister,

    pub cycles: usize,
}

impl APU {
    pub fn new() -> Self {
        Self {
            pulse_one: PulseRegisters::new(),
            pulse_two: PulseRegisters::new(),
            triangle: TriangleRegisters::new(),
            noise: NoiseRegisters::new(),
            dmc: DMCRegisters::new(),

            status: StatusRegister::new(),
            frame_counter: FrameCounterRegister::new(),

            cycles: 0,
        }
    }

    pub fn read_status_register(&self) -> u8 {
        // todo: implement side-effects
        self.status.get_value()
    }

    pub fn write_status_register(&mut self, value: u8) {
        let frame_int_mask = (self.status.is_set(FrameInterrupt) as u8) << 6;
        self.status.set_value((value & 0b0001_1111) | frame_int_mask);
        if self.status.is_clear(PulseOneEnable) {
            self.pulse_one.clear_length_counter();
        }
        if self.status.is_clear(PulseTwoEnable) {
            self.pulse_two.clear_length_counter();
        }
        // todo: implement rest
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize / 2;
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        Ok(true)
    }
}