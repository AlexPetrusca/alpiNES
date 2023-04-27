use sdl2::audio::{AudioCallback, AudioDevice, AudioQueue, AudioSpecDesired};
use sdl2::AudioSubsystem;

pub struct APUMixer {
    pub pulse_one: PulseWave,
    pub pulse_two: PulseWave,
}

impl APUMixer {
    pub fn new() -> Self {
        Self {
            pulse_one: PulseWave::new(),
            pulse_two: PulseWave::new(),
        }
    }
}

impl AudioCallback for APUMixer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            *sample = self.pulse_one.callback() + self.pulse_two.callback();
        }
    }
}

pub struct PulseWave {
    pub phase: f32,
    pub phase_inc: f32,
    pub volume: f32
}

impl PulseWave {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            volume: 0.0
        }
    }

    #[inline]
    pub fn callback(&mut self) -> f32 {
        let sample = if self.phase <= 0.125 || self.phase > 0.625 {
            self.volume
        } else {
            -self.volume
        };
        self.phase = (self.phase + self.phase_inc) % 1.0;
        sample
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