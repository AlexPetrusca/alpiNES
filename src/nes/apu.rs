use crate::nes::apu::registers::frame_counter::FrameCounterRegister;
use crate::nes::apu::registers::dmc::DMCRegisters;
use crate::nes::apu::registers::noise::NoiseRegisters;
use crate::nes::apu::registers::pulse::PulseRegisters;
use crate::nes::apu::registers::status::StatusFlag::{FrameInterrupt, NoiseEnable, PulseOneEnable, PulseTwoEnable, TriangleEnable};
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

    pub cpu_cycles: usize,
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

            cpu_cycles: 0,
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
        if self.status.is_clear(TriangleEnable) {
            self.triangle.clear_length_counter();
        }
        if self.status.is_clear(NoiseEnable) {
            self.noise.clear_length_counter();
        }
        // todo: implement rest
    }

    pub fn read_frame_counter_register(&self) -> u8 {
        self.frame_counter.read()
    }

    pub fn write_frame_counter_register(&mut self, value: u8) {
        self.frame_counter.write(value);
        self.frame_counter.reset(); // todo: is this good enough? (ie. 3-4 cycle reset issue)
        // todo: implement missing side-effects (https://www.nesdev.org/wiki/APU_Frame_Counter)
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cpu_cycles += cycles as usize;
    }

    pub fn step(&mut self) -> Result<bool, bool> {
        if self.frame_counter.is_four_step_mode() {
            self.step_four_mode();
        } else {
            self.step_five_mode();
        }
        Ok(true)
    }

    fn step_four_mode(&mut self) {
        if self.cpu_cycles > 7457 && self.frame_counter.get_step() == 0 {
            self.frame_counter.increment();
            self.update_quarter_frame();
        }

        if self.cpu_cycles > 14913 && self.frame_counter.get_step() == 1 {
            self.frame_counter.increment();
            self.update_quarter_frame();
            self.update_half_frame();
        }

        if self.cpu_cycles > 22371 && self.frame_counter.get_step() == 2 {
            self.frame_counter.increment();
            self.update_quarter_frame();
        }

        if self.cpu_cycles > 29830 && self.frame_counter.get_step() == 3 {
            self.frame_counter.increment();
            self.update_quarter_frame();
            self.update_half_frame();
            self.set_irq();
            self.cpu_cycles -= 29830;
        }
    }

    fn step_five_mode(&mut self) {
        if self.cpu_cycles > 7457 && self.frame_counter.get_step() == 0 {
            self.frame_counter.increment();
            self.update_quarter_frame();
        }

        if self.cpu_cycles > 14913 && self.frame_counter.get_step() == 1 {
            self.frame_counter.increment();
            self.update_quarter_frame();
            self.update_half_frame();
        }

        if self.cpu_cycles > 22371 && self.frame_counter.get_step() == 2 {
            self.frame_counter.increment();
            self.update_quarter_frame();
        }

        if self.cpu_cycles > 29829 && self.frame_counter.get_step() == 3 {
            self.frame_counter.increment();
        }

        if self.cpu_cycles > 37282 && self.frame_counter.get_step() == 4 {
            self.frame_counter.increment();
            self.update_quarter_frame();
            self.update_half_frame();
            self.cpu_cycles -= 37282;
        }
    }

    fn update_quarter_frame(&mut self) {
        self.triangle.decrement_linear_counter();
        // todo: update envelopes
    }

    fn update_half_frame(&mut self) {
        // todo: update length counters
        self.triangle.decrement_length_counter();
        // todo: update sweep units
    }

    fn set_irq(&mut self) {
        // todo: implement
    }
}