use sdl2::audio::{AudioCallback, AudioDevice, AudioQueue, AudioSpecDesired};
use sdl2::AudioSubsystem;

pub struct APUMixer {
    pub pulse_one: PulseWave,
    pub pulse_two: PulseWave,
    pub triangle: TriangleWave,

    pub volume: f32,
    pub mute: bool,
}

impl APUMixer {
    pub fn new() -> Self {
        Self {
            pulse_one: PulseWave::new(),
            pulse_two: PulseWave::new(),
            triangle: TriangleWave::new(),

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
            let pulse_two = self.pulse_two.sample() as f32;
            let pulse_out = 95.88 / (8128.0 / (pulse_one + pulse_two) + 100.0);

            let triangle = self.triangle.sample() as f32;
            let noise = 0 as f32; // todo: implement
            let dmc = 0 as f32; // todo: implement
            let tnd = 1.0 / (triangle / 8227.0 + noise / 12241.0 + dmc / 22638.0);
            let tnd_out = 159.79 / (tnd + 100.0);

            let sample_out = pulse_out + tnd_out;
            let system_volume = if self.mute { 0.0 } else { 1.0 } * self.volume;
            *sample = system_volume * sample_out;
        }
    }
}

pub struct PulseWave {
    pub phase: f32,
    pub phase_inc: f32,
    pub volume: u8,
    pub duty: u8,
}

impl PulseWave {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            volume: 0,
            duty: 0,
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
        self.phase = (self.phase + self.phase_inc) % 1.0;
        sample
    }

    #[inline]
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.phase_inc = 0.0;
        self.volume = 0;
        self.duty = 0;
    }
}

pub struct TriangleWave {
    pub phase: f32,
    pub phase_inc: f32,
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
        }
    }

    #[inline]
    pub fn sample(&mut self) -> u8 {
        let index = (32.0 * self.phase).floor() as usize;
        self.phase = (self.phase + self.phase_inc) % 1.0;
        TriangleWave::WAVEFORM[index]
    }
}

pub struct AudioPlayer {
    pub sdl_audio: AudioSubsystem,
    pub spec: AudioSpecDesired,
    pub device: AudioDevice<APUMixer>,
}

impl AudioPlayer {
    pub const FREQ: i32 = 44100;

    pub fn new(sdl_audio: AudioSubsystem) -> Self {
        let spec = AudioSpecDesired {
            freq: Some(44100),
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