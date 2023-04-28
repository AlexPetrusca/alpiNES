use sdl2::Sdl;
use crate::nes::apu::registers::frame_counter::FrameCounterRegister;
use crate::nes::apu::registers::dmc::DMCRegisters;
use crate::nes::apu::registers::noise::NoiseRegisters;
use crate::nes::apu::registers::pulse::PulseRegisters;
use crate::nes::apu::registers::status::StatusFlag::{FrameInterrupt, NoiseEnable, PulseOneEnable, PulseTwoEnable, TriangleEnable};
use crate::nes::apu::registers::status::StatusRegister;
use crate::nes::apu::registers::triangle::TriangleRegisters;
use crate::nes::cpu::mem::Memory;
use crate::util::audio::AudioPlayer;
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

    pub audio_player: Option<AudioPlayer>,
    pub cpu_cycles: usize,
}

impl APU {
    const REGISTER_A: u8 = 0;
    const REGISTER_B: u8 = 1;
    const REGISTER_C: u8 = 2;
    const REGISTER_D: u8 = 3;

    pub fn new() -> Self {
        Self {
            pulse_one: PulseRegisters::new(),
            pulse_two: PulseRegisters::new(),
            triangle: TriangleRegisters::new(),
            noise: NoiseRegisters::new(),
            dmc: DMCRegisters::new(),

            status: StatusRegister::new(),
            frame_counter: FrameCounterRegister::new(),

            audio_player: None,
            cpu_cycles: 0,
        }
    }

    pub fn init_audio_player(&mut self, sdl_context: &Sdl) {
        let audio_subsystem = sdl_context.audio().unwrap();
        let audio_player = AudioPlayer::new(audio_subsystem);
        self.audio_player = Some(audio_player)
    }

    pub fn read_status_register(&self) -> u8 {
        // todo: implement side-effects
        self.status.get_value()
    }

    pub fn write_status_register(&mut self, value: u8) {
        let frame_int_mask = (self.status.is_set(FrameInterrupt) as u8) << 6;
        self.status.set_value((value & 0b0001_1111) | frame_int_mask);

        let mut guard = self.audio_player.as_mut().unwrap().device.lock();
        if self.status.is_clear(PulseOneEnable) {
            self.pulse_one.clear_length_counter();
            guard.pulse_one.silence();
        }
        if self.status.is_clear(PulseTwoEnable) {
            self.pulse_two.clear_length_counter();
            guard.pulse_two.silence();
        }
        if self.status.is_clear(TriangleEnable) {
            self.triangle.clear_length_counter();
            guard.triangle.silence();
        }
        if self.status.is_clear(NoiseEnable) {
            self.noise.clear_length_counter();
            // guard.noise.silence();
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

    pub fn write_pulse_one_registers(&mut self, register_idx: u8, data: u8) {
        self.pulse_one.write(register_idx, data);
        let mut guard = self.audio_player.as_mut().unwrap().device.lock();
        if register_idx == APU::REGISTER_A {
            guard.pulse_one.duty = self.pulse_one.get_duty();
            guard.pulse_one.volume = self.pulse_one.get_volume();
        }
        if register_idx == APU::REGISTER_C || register_idx == APU::REGISTER_D {
            if self.pulse_one.get_length_counter() == 0 || self.pulse_one.get_timer() < 8 {
                guard.pulse_one.silence();
            } else {
                guard.pulse_one.phase_inc = self.pulse_one.get_frequency() / AudioPlayer::FREQ as f32;
            }
        }
    }

    pub fn write_pulse_two_registers(&mut self, register_idx: u8, data: u8) {
        self.pulse_two.write(register_idx, data);
        let mut guard = self.audio_player.as_mut().unwrap().device.lock();
        if register_idx == APU::REGISTER_A {
            guard.pulse_two.duty = self.pulse_two.get_duty();
            guard.pulse_two.volume = self.pulse_two.get_volume();
        }
        if register_idx == APU::REGISTER_C || register_idx == APU::REGISTER_D {
            if self.pulse_two.get_length_counter() == 0 || self.pulse_two.get_timer() < 8 {
                guard.pulse_two.silence();
            } else {
                guard.pulse_two.phase_inc = self.pulse_two.get_frequency() / AudioPlayer::FREQ as f32;
            }
        }
    }

    pub fn write_triangle_registers(&mut self, register_idx: u8, data: u8) {
        self.triangle.write(register_idx, data);
        let mut guard = self.audio_player.as_mut().unwrap().device.lock();
        if register_idx == APU::REGISTER_D {
            if self.triangle.get_linear_counter() == 0 {
                guard.triangle.silence();
            } else {
                let rate = AudioPlayer::FREQ as f32 / 240.0;
                guard.triangle.duration = rate * self.triangle.get_linear_counter() as f32;
                guard.triangle.duration_counter = 0.0;
            }
        }
        if register_idx == APU::REGISTER_C || register_idx == APU::REGISTER_D {
            if self.triangle.get_length_counter() == 0 || self.triangle.get_timer() < 2 {
                guard.pulse_two.silence();
            } else {
                guard.triangle.phase_inc = self.triangle.get_frequency() / AudioPlayer::FREQ as f32;
            }
        }
    }

    pub fn write_noise_registers(&mut self, register_idx: u8, data: u8) {
        self.noise.write(register_idx, data);
    }

    pub fn write_dmc_registers(&mut self, register_idx: u8, data: u8) {
        self.dmc.write(register_idx, data);
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
        // self.triangle.decrement_linear_counter();
        // todo: update envelopes
    }

    fn update_half_frame(&mut self) {
        // todo: update length counters
        // self.triangle.decrement_length_counter();
        // todo: update sweep units
    }

    fn set_irq(&mut self) {
        // todo: implement
    }
}