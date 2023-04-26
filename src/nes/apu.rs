use crate::nes::apu::registers::frame_counter::FrameCounterRegister;
use crate::nes::apu::registers::dmc::DMCRegisters;
use crate::nes::apu::registers::noise::NoiseRegisters;
use crate::nes::apu::registers::pulse::PulseRegisters;
use crate::nes::apu::registers::status::StatusRegister;
use crate::nes::apu::registers::triangle::TriangleRegisters;

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

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize / 2;
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        Ok(true)
    }
}