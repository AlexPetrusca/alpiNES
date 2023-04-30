use rand::{Rng, thread_rng};
use sdl2::audio::{AudioCallback, AudioDevice, AudioQueue, AudioSpecDesired};
use sdl2::AudioSubsystem;
use crate::nes::cpu::mem::Memory;

pub struct APUMixer {
    pub pulse_one: PulseWave,
    pub pulse_two: PulseWave,
    pub triangle: TriangleWave,
    pub noise: NoiseWave,
    pub dmc: DMCWave,

    pub volume: f32,
    pub mute: bool,
    pub mute_pulse_one: bool,
    pub mute_pulse_two: bool,
    pub mute_triangle: bool,
    pub mute_noise: bool,
    pub mute_dmc: bool,
}

impl APUMixer {
    pub fn new() -> Self {
        Self {
            pulse_one: PulseWave::new(1),
            pulse_two: PulseWave::new(2),
            triangle: TriangleWave::new(),
            noise: NoiseWave::new(),
            dmc: DMCWave::new(),

            volume: 1.0,
            mute: false,
            mute_pulse_one: false,
            mute_pulse_two: false,
            mute_triangle: false,
            mute_noise: false,
            mute_dmc: false,
        }
    }
}

impl AudioCallback for APUMixer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            let pulse_one = if self.mute_pulse_one { 0.0 } else { self.pulse_one.sample() as f32 };
            let pulse_two = if self.mute_pulse_two { 0.0 } else { self.pulse_two.sample() as f32 };
            let pulse_out = 95.88 / (8128.0 / (pulse_one + pulse_two) + 100.0);

            let triangle = if self.mute_triangle { 0.0 } else { self.triangle.sample() as f32 };
            let noise = if self.mute_noise { 0.0 } else { self.noise.sample() as f32 };
            let dmc = if self.mute_dmc { 0.0 } else { self.dmc.sample() as f32 };
            let tnd = 1.0 / (triangle / 8227.0 + noise / 12241.0 + dmc / 22638.0);
            let tnd_out = 159.79 / (tnd + 100.0);

            let sample_out = pulse_out + tnd_out;
            let system_volume = if self.mute { 0.0 } else { 1.0 } * self.volume;
            *sample = system_volume * sample_out;
        }
    }
}

pub struct PulseWave {
    phase: f32,
    phase_inc: f32,
    envelope_enable: bool,
    env_phase: f32,
    env_phase_inc: f32,
    sweep_enable: bool,
    sweep_negate: bool,
    sweep_phase: f32,
    sweep_phase_inc: f32,
    sweep_shift: u8,
    sweep_timer: u16,
    duration_enable: bool,
    duration: f32,
    duration_counter: f32,
    volume: u8,
    duty: u8,
    channel: u8,
}

impl PulseWave {
    pub fn new(channel: u8) -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            envelope_enable: false,
            env_phase: 0.0,
            env_phase_inc: 0.0,
            sweep_enable: false,
            sweep_negate: false,
            sweep_phase: 0.0,
            sweep_phase_inc: 0.0,
            sweep_shift: 0,
            sweep_timer: 0,
            duration_enable: false,
            duration: 0.0,
            duration_counter: 0.0,
            volume: 0,
            duty: 0,
            channel: channel
        }
    }

    pub fn sample(&mut self) -> u8 {
        // duty
        let mut sample = match self.duty {
            0 => if self.phase >= 0.125 && self.phase <= 0.250 { self.volume } else { 0 },
            1 => if self.phase >= 0.125 && self.phase <= 0.375 { self.volume } else { 0 },
            2 => if self.phase >= 0.125 && self.phase <= 0.625 { self.volume } else { 0 },
            3 => if self.phase >= 0.125 && self.phase <= 0.375 { 0 } else { self.volume },
            _ => panic!("can't be")
        };

        // waveform
        self.phase = (self.phase + self.phase_inc) % 1.0;

        // envelope
        if self.envelope_enable {
            let old_env_phase = self.env_phase;
            self.env_phase = (self.env_phase + self.env_phase_inc) % 1.0;
            if self.env_phase < old_env_phase && self.volume > 0 {
                self.volume -= 1;
            }
        }

        // sweep
        // todo: sweep has some issues with timing:
        //  - sometimes extra pitch at the end of mario's jump
        //  - sometimes fire balls are noticeably higher pitched
        let target_timer = self.get_sweep_target_timer();
        if self.sweep_enable {
            let old_sweep_phase = self.sweep_phase;
            self.sweep_phase = (self.sweep_phase + self.sweep_phase_inc) % 1.0;
            if self.sweep_phase < old_sweep_phase {
                self.set_frequency_from_timer(target_timer);
            }
        }
        if self.sweep_timer < 8 || target_timer > 0x7FF {
            sample = 0; // mute
        }

        // loop vs one-shot
        if !self.duration_enable {
            return sample;
        } else if self.duration_counter < self.duration {
            self.duration_counter += 1.0;
            return sample;
        }
        return 0;
    }

    pub fn silence(&mut self) {
        self.volume = 0;
        // self.phase = 0.0;
        // self.env_phase = 0.0;
        // self.sweep_phase = 0.0;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.sweep_phase = 0.0; // todo: do I need to reset this?
        if self.envelope_enable {
            self.env_phase = 0.0;
            self.volume = 15;
        }
    }

    fn get_sweep_target_timer(&mut self) -> u16 {
        let mut delta = self.sweep_timer >> self.sweep_shift;
        if self.sweep_negate {
            delta = if self.channel == 1 { !delta } else { delta.wrapping_neg() };
        }
        self.sweep_timer.wrapping_add(delta)
    }

    pub fn set_frequency_from_timer(&mut self, timer: u16) {
        self.sweep_timer = timer;
        self.set_frequency(1_789_773.0 / (16.0 * (timer as f32 + 1.0)));
    }

    fn set_frequency(&mut self, freq: f32) {
        self.phase_inc = freq / AudioPlayer::FREQ as f32;
        self.phase = 0.0;
    }

    pub fn set_envelope_enable(&mut self, envelope_enable: bool) {
        self.envelope_enable = envelope_enable;
        self.volume = 15;
    }

    pub fn set_envelope_frequency(&mut self, env_freq: f32) {
        self.env_phase_inc = env_freq / AudioPlayer::FREQ as f32;
        self.env_phase = 0.0;
    }

    pub fn set_sweep_enable(&mut self, sweep_enable: bool) {
        self.sweep_enable = sweep_enable;
    }

    pub fn set_sweep_negate(&mut self, sweep_negate: bool) {
        self.sweep_negate = sweep_negate;
    }

    pub fn set_sweep_frequency(&mut self, sweep_freq: f32) {
        self.sweep_phase_inc = sweep_freq / AudioPlayer::FREQ as f32;
        self.sweep_phase = 0.0;
    }

    pub fn set_sweep_shift(&mut self, sweep_shift: u8) {
        self.sweep_shift = sweep_shift;
    }

    pub fn set_duration_enable(&mut self, duration_enable: bool) {
        self.duration_enable = duration_enable;
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
        self.duration_counter = 0.0;
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
    }

    pub fn set_duty(&mut self, duty: u8) {
        self.duty = duty;
    }
}

pub struct TriangleWave {
    phase: f32,
    phase_inc: f32,
    duration: f32,
    duration_counter: f32,
}

impl TriangleWave {
    const WAVEFORM: [u8; 32] = [
        15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
         0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
    ];

    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            duration: 0.0,
            duration_counter: 0.0
        }
    }

    #[inline]
    pub fn sample(&mut self) -> u8 {
        if self.duration_counter < self.duration {
            self.phase = (self.phase + self.phase_inc) % 1.0;
            self.duration_counter += 1.0;
        }
        let index = (32.0 * self.phase).floor() as usize;
        TriangleWave::WAVEFORM[index]
    }

    #[inline]
    pub fn silence(&mut self) {
        self.phase = 0.0;
        self.duration = 0.0;
        self.duration_counter = 0.0;
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
        self.duration_counter = 0.0;
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_inc = freq / AudioPlayer::FREQ as f32;
    }
}

pub struct NoiseWave {
    phase: f32,
    phase_inc: f32,
    duration: f32,
    duration_counter: f32,
    volume: u8,
    shift_register: u16,
}

impl NoiseWave {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            duration: 0.0,
            duration_counter: 0.0,
            volume: 0,
            shift_register: 1,
        }
    }

    #[inline]
    pub fn sample(&mut self) -> u8 {
        let old_phase = self.phase;
        if self.duration_counter < self.duration {
            self.phase = (self.phase + self.phase_inc) % 1.0;
            self.duration_counter += 1.0;
        }
        if self.phase < old_phase {
            let feedback = (self.shift_register & 1) ^ ((self.shift_register >> 1) & 1); // todo: mode flag impl
            self.shift_register = self.shift_register >> 1;
            self.shift_register = self.shift_register | (feedback << 14);
        }
        self.volume * (self.shift_register & 1) as u8

        // todo: this is the fceux implementation. Which one is better?
        // if self.phase < old_phase {
        //     self.shift_register = (self.shift_register << 1) + (((self.shift_register >> 13) ^ ( self.shift_register >> 14)) & 1);
        //     // self.shift_register = ( self.shift_register<<1)+(((self.shift_register>>8)^( self.shift_register>>14))&1);
        // }
        // self.volume * ((self.shift_register >> 14) & 1) as u8
    }

    #[inline]
    pub fn silence(&mut self) {
        self.phase = 0.0;
        self.volume = 0;
        self.duration = 0.0;
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_inc = freq / AudioPlayer::FREQ as f32;
        self.phase = 0.0;
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
        self.duration_counter = 0.0;
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
    }
}

// todo: fully implement DMC
pub struct DMCWave {
    phase: f32,
    phase_inc: f32,
    duration: f32,
    duration_counter: f32,
    volume: u8,
    silence: bool,
    dpcm_samples: Vec<u8>,
}

impl DMCWave {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            duration: 0.0,
            duration_counter: 0.0,
            volume: 0,
            silence: false,
            dpcm_samples: Vec::new(),
        }
    }

    #[inline]
    pub fn sample(&mut self) -> u8 {
        if self.duration_counter < self.duration {
            self.phase = (self.phase + self.phase_inc) % 1.0;
            self.duration_counter += 1.0;
        }
        self.volume
    }

    #[inline]
    pub fn silence(&mut self) {
        self.phase = 0.0;
        self.silence = true;
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_inc = freq / AudioPlayer::FREQ as f32;
        self.phase = 0.0;
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
        self.duration_counter = 0.0;
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
    }

    pub fn add_dpcm_sample(&mut self, sample: u8) {
        self.dpcm_samples.push(sample);
    }
}

pub struct AudioPlayer {
    pub sdl_audio: AudioSubsystem,
    pub spec: AudioSpecDesired,
    pub device: AudioDevice<APUMixer>,
}

impl AudioPlayer {
    pub const FREQ: i32 = 16 * 44100;
    pub const LENGTH_LOOKUP: [u16; 32] = [
        10, 254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
        12, 16,  24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
    ];

    pub fn new(sdl_audio: AudioSubsystem) -> Self {
        let spec = AudioSpecDesired {
            freq: Some(AudioPlayer::FREQ),
            channels: Some(1),
            samples: None
        };
        let device = sdl_audio.open_playback(None, &spec, |spec| {
            APUMixer::new()
        }).unwrap();
        device.resume();
        AudioPlayer { sdl_audio, spec, device }
    }

    pub fn play(&self) {
    }
}