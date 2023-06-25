use std::thread;
use std::time::{Duration, Instant};

pub struct PreciseSleeper {
    estimate: f64,
    mean: f64,
    m2: f64,
    count: u64,
}

impl PreciseSleeper {
    pub fn new() -> Self {
        PreciseSleeper {
            estimate: 5e-3,
            mean: 5e-3,
            m2: 0.0,
            count: 1,
        }
    }

    pub fn precise_sleep(&mut self, seconds: f64) {
        let mut seconds = seconds;

        // sleeping in 1ms chunks
        while seconds > self.estimate {
            let start = Instant::now();
            thread::sleep(Duration::from_millis(1));
            let end = Instant::now();

            let observed = end.duration_since(start).as_secs_f64();
            seconds -= observed;

            self.count += 1;
            let delta = observed - self.mean;
            self.mean += delta / self.count as f64;
            self.m2 += delta * (observed - self.mean);
            let stddev = (self.m2 / (self.count as f64 - 1.0)).sqrt();
            self.estimate = self.mean + stddev;
        }

        // spin lock
        let start = Instant::now();
        while Instant::now().duration_since(start).as_secs_f64() < seconds { /* spin */ };
    }
}

