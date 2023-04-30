use sdl2::Sdl;
use crate::nes::apu::registers::frame_counter::FrameCounterRegister;
use crate::nes::apu::registers::dmc::DMCRegisters;
use crate::nes::apu::registers::noise::NoiseRegisters;
use crate::nes::apu::registers::pulse::PulseRegisters;
use crate::nes::apu::registers::status::StatusFlag::{DmcEnable, FrameInterrupt, NoiseEnable, PulseOneEnable, PulseTwoEnable, TriangleEnable};
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
            guard.noise.silence();
        }
        if self.status.is_clear(DmcEnable) {
            // self.dmc.clear_length_counter();
            guard.dmc.silence();
        }
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
            guard.pulse_one.set_duty(self.pulse_one.get_duty());
            guard.pulse_one.set_duration_enable(self.pulse_one.is_one_shot());
            guard.pulse_one.set_envelope_enable(self.pulse_one.is_envelope_volume());
            if self.pulse_one.is_envelope_volume() {
                guard.pulse_one.set_envelope_frequency(self.pulse_one.get_envelope_frequency());
            } else {
                guard.pulse_one.set_volume(self.pulse_one.get_volume());
            }
        }
        if register_idx == APU::REGISTER_B {
            guard.pulse_one.set_sweep_enable(self.pulse_one.is_sweep_enabled());
            guard.pulse_one.set_sweep_negate(self.pulse_one.is_sweep_negate());
            guard.pulse_one.set_sweep_shift(self.pulse_one.get_sweep_shift());
            guard.pulse_one.set_sweep_frequency(self.pulse_one.get_sweep_frequency());
        }
        if register_idx == APU::REGISTER_C {
            guard.pulse_one.set_frequency_from_timer(self.pulse_one.get_timer());
        }
        if register_idx == APU::REGISTER_D {
            guard.pulse_one.set_frequency_from_timer(self.pulse_one.get_timer());
            guard.pulse_one.set_duration(self.pulse_one.get_duration());
            guard.pulse_one.reset();
        }
        if !guard.mute_pulse_one {
            println!("pulse_one: freq: {}, timer: {}, volume: {}, duty: {}, length_counter: {}, \
              is_loop: {}, is_envelope: {}, is_sweep: {}, sweep_negate: {}, \
              sweep_period: {}, sweep_shift: {}",
                self.pulse_one.get_frequency(), self.pulse_one.get_timer(), self.pulse_one.get_volume(),
                self.pulse_one.get_duty(), self.pulse_one.get_length_counter(), self.pulse_one.is_loop(),
                self.pulse_one.is_envelope_volume(), self.pulse_one.is_sweep_enabled(),
                self.pulse_one.is_sweep_negate(), self.pulse_one.get_sweep_period(),
                self.pulse_one.get_sweep_shift()
            );
        }
    }

    pub fn write_pulse_two_registers(&mut self, register_idx: u8, data: u8) {
        self.pulse_two.write(register_idx, data);
        let mut guard = self.audio_player.as_mut().unwrap().device.lock();
        if register_idx == APU::REGISTER_A {
            guard.pulse_two.set_duty(self.pulse_two.get_duty());
            guard.pulse_two.set_duration_enable(self.pulse_two.is_one_shot());
            guard.pulse_two.set_envelope_enable(self.pulse_two.is_envelope_volume());
            if self.pulse_two.is_envelope_volume() {
                guard.pulse_two.set_envelope_frequency(self.pulse_two.get_envelope_frequency());
            } else {
                guard.pulse_two.set_volume(self.pulse_two.get_volume());
            }
        }
        if register_idx == APU::REGISTER_B {
            guard.pulse_two.set_sweep_enable(self.pulse_two.is_sweep_enabled());
            guard.pulse_two.set_sweep_negate(self.pulse_two.is_sweep_negate());
            guard.pulse_two.set_sweep_shift(self.pulse_two.get_sweep_shift());
            guard.pulse_two.set_sweep_frequency(self.pulse_two.get_sweep_frequency());
        }
        if register_idx == APU::REGISTER_C {
            guard.pulse_two.set_frequency_from_timer(self.pulse_two.get_timer());
        }
        if register_idx == APU::REGISTER_D {
            guard.pulse_two.set_frequency_from_timer(self.pulse_two.get_timer());
            guard.pulse_two.set_duration(self.pulse_two.get_duration());
            guard.pulse_two.reset();
        }
        if !guard.mute_pulse_two {
            println!("pulse_two: freq: {}, timer: {}, volume: {}, duty: {}, length_counter: {}, \
              is_loop: {}, is_envelope: {}, is_sweep: {}, sweep_negate: {}, \
              sweep_period: {}, sweep_shift: {}",
                self.pulse_two.get_frequency(), self.pulse_two.get_timer(), self.pulse_two.get_volume(),
                self.pulse_two.get_duty(), self.pulse_two.get_length_counter(), self.pulse_two.is_loop(),
                self.pulse_two.is_envelope_volume(), self.pulse_two.is_sweep_enabled(),
                self.pulse_two.is_sweep_negate(), self.pulse_two.get_sweep_period(),
                self.pulse_two.get_sweep_shift()
            );
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
                guard.triangle.set_duration(rate * self.triangle.get_linear_counter() as f32);
            }
        }
        if register_idx == APU::REGISTER_C || register_idx == APU::REGISTER_D {
            if self.triangle.get_length_counter() == 0 || self.triangle.get_timer() < 2 {
                guard.triangle.silence();
            } else {
                guard.triangle.set_frequency(self.triangle.get_frequency());
            }
        }
        if !guard.mute_triangle {
            println!("triangle: freq: {}, timer: {}, length_counter: {}, linear_counter: {}",
                self.triangle.get_frequency(), self.triangle.get_timer(),
                self.triangle.get_length_counter(), self.triangle.get_linear_counter());
        }
    }

    pub fn write_noise_registers(&mut self, register_idx: u8, data: u8) {
        self.noise.write(register_idx, data);
        let mut guard = self.audio_player.as_mut().unwrap().device.lock();
        if register_idx == APU::REGISTER_A {
            guard.noise.set_volume(self.noise.get_volume());
        }
        if register_idx == APU::REGISTER_C {
            guard.noise.set_is_tone_mode(self.noise.is_tone_mode());
            guard.noise.set_frequency(self.noise.get_frequency());
        }
        if register_idx == APU::REGISTER_D {
            if self.noise.get_length_counter() == 0 {
                guard.noise.silence();
            } else {
                let rate = AudioPlayer::FREQ as f32 / 120.0;
                guard.noise.set_duration(rate * self.noise.get_length_counter() as f32);
            }
        }
        if !guard.mute_noise {
            println!("noise: freq: {}, period: {}, volume: {}, length_counter: {}, tone-mode: {}, constant-volume: {}, one-shot: {}",
                self.noise.get_frequency(), self.noise.get_period(), self.noise.get_volume(),
                self.noise.get_length_counter(), self.noise.is_tone_mode(),
                self.noise.is_constant_volume(), self.noise.is_one_shot_play());
        }
    }

    pub fn write_dmc_registers(&mut self, register_idx: u8, data: u8) {
        self.dmc.write(register_idx, data);
        let mut guard = self.audio_player.as_mut().unwrap().device.lock();
        if register_idx == APU::REGISTER_A {
            guard.dmc.set_frequency(self.dmc.get_frequency());
        }
        if register_idx == APU::REGISTER_B {
            guard.dmc.set_volume(self.dmc.get_volume());
        }
        if !guard.mute_dmc {
            println!("dmc: volume: {}, rate: {}, sample_address: 0x{:x}, sample_length: {}",
                self.dmc.get_volume(), self.dmc.get_rate_idx(), self.dmc.get_sample_address(),
                self.dmc.get_sample_length());
        }
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