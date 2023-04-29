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
}

impl APUMixer {
    pub fn new() -> Self {
        Self {
            pulse_one: PulseWave::new(),
            pulse_two: PulseWave::new(),
            triangle: TriangleWave::new(),
            noise: NoiseWave::new(),
            dmc: DMCWave::new(),

            volume: 1.0,
            mute: false,
        }
    }
}

impl AudioCallback for APUMixer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            let pulse_one = self.pulse_one.sample() as f32;
            // let pulse_one = 0.0;
            let pulse_two = self.pulse_two.sample() as f32;
            // let pulse_two = 0.0;
            let pulse_out = 95.88 / (8128.0 / (pulse_one + pulse_two) + 100.0);

            let triangle = self.triangle.sample() as f32;
            // let triangle = 0.0;
            let noise = self.noise.sample() as f32;
            // let noise = 0.0;
            let dmc = self.dmc.sample() as f32;
            // let dmc = 0.0;
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
    duration: f32,
    duration_counter: f32,
    volume: u8,
    duty: u8,
    is_loop: bool,
}

impl PulseWave {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            duration: 0.0,
            duration_counter: 0.0,
            volume: 0,
            duty: 0,
            is_loop: false,
        }
    }

    #[inline]
    pub fn sample(&mut self) -> u8 {
        let sample = match self.duty {
            0 => if self.phase >= 0.125 && self.phase <= 0.250 { self.volume } else { 0 },
            1 => if self.phase >= 0.125 && self.phase <= 0.375 { self.volume } else { 0 },
            2 => if self.phase >= 0.125 && self.phase <= 0.625 { self.volume } else { 0 },
            3 => if self.phase >= 0.125 && self.phase <= 0.375 { 0 } else { self.volume },
            _ => panic!("can't be")
        };
        if self.is_loop {
            self.phase = (self.phase + self.phase_inc) % 1.0;
            return sample;
        } else if self.duration_counter < self.duration {
            self.phase = (self.phase + self.phase_inc) % 1.0;
            self.duration_counter += 1.0;
            return sample;
        }
        return 0;
    }

    #[inline]
    pub fn silence(&mut self) {
        self.phase = 0.0;
        self.volume = 0;
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
        self.duration_counter = 0.0;
    }

    pub fn set_is_loop(&mut self, is_loop: bool) {
        self.is_loop = is_loop;
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_inc = freq / AudioPlayer::FREQ as f32;
        self.phase = 0.0;
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