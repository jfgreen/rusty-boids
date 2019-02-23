use std::time::{Duration, Instant};

const NUM_SAMPLES: usize = 20;

pub struct FpsCache {
    cache_interval: Duration,
    last_updated: Instant,
    last_yielded_fps: u32,
}

impl FpsCache {
    pub fn new(cache_ms: u64) -> FpsCache {
        FpsCache {
            cache_interval: Duration::from_millis(cache_ms),
            last_updated: Instant::now(),
            last_yielded_fps: 0,
        }
    }

    pub fn poll<F>(&mut self, counter: &FpsCounter, handler: F)
    where
        F: Fn(u32),
    {
        let since_last_update = self.last_updated.elapsed();
        if since_last_update > self.cache_interval {
            self.last_updated = Instant::now();
            let fps = counter.average_fps();
            if fps != self.last_yielded_fps {
                self.last_yielded_fps = fps;
                handler(fps);
            }
        }
    }
}

pub struct FpsCounter {
    last_sampled: Instant,
    samples: Vec<Duration>,
    current_sample: usize,
    rolling_dt_sum: Duration,
}

impl FpsCounter {
    // TODO: Would be more accurate to store instances
    // See: https://github.com/PistonDevelopers/fps_counter
    // Could apply rolling average (or LPF) over that to smooth

    pub fn new() -> FpsCounter {
        FpsCounter {
            last_sampled: Instant::now(),
            samples: vec![Duration::new(0, 0); NUM_SAMPLES],
            current_sample: 0,
            rolling_dt_sum: Duration::new(0, 0),
        }
    }

    pub fn tick(&mut self) {
        let dt = self.last_sampled.elapsed();
        self.last_sampled = Instant::now();
        self.record(dt);
    }

    pub fn average_delta(&self) -> Duration {
        self.rolling_dt_sum / self.samples.len() as u32
    }

    pub fn average_fps(&self) -> u32 {
        let dt = self.average_delta();
        if dt > Duration::from_secs(0) && dt < Duration::from_secs(1) {
            1_000_000_000 / dt.subsec_nanos()
        } else {
            0
        }
    }

    fn record(&mut self, sample: Duration) {
        self.rolling_dt_sum -= self.samples[self.current_sample];
        self.rolling_dt_sum += sample;
        self.samples[self.current_sample] = sample;
        self.current_sample += 1;
        self.current_sample %= self.samples.len();
    }
}
