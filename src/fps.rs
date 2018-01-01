use std::time::{Duration, Instant};

const NUM_SAMPLES: usize = 20;

pub struct CachedFpsCounter {
    samples: SampleBuffer,
    cache_interval: Duration,
    last_updated_fps: Instant,
    last_sampled: Instant,
    last_shown_fps: u32,
    last_delta: f64,
}

impl CachedFpsCounter {

    pub fn new(cache_ms: u64) -> CachedFpsCounter {
        CachedFpsCounter {
            samples: SampleBuffer::new(),
            cache_interval: Duration::from_millis(cache_ms),
            last_updated_fps: Instant::now(),
            last_sampled: Instant::now(),
            last_shown_fps: 0,
            last_delta: 0.,
        }
    }

    pub fn tick(&mut self) {
        let dt = self.last_sampled.elapsed();
        self.last_sampled = Instant::now();
        self.samples.record(dt.subsec_nanos());
        self.last_delta = dt.as_secs() as f64
                        + dt.subsec_nanos() as f64 * 1e-9;
    }

    pub fn poll_change<F>(&mut self, callback: F) where F: Fn(u32) {
        let since_last_update = self.last_updated_fps.elapsed();
        if since_last_update > self.cache_interval {
            self.last_updated_fps = Instant::now();
            let fps = self.samples.average();
            if fps != self.last_shown_fps {
                self.last_shown_fps = fps;
                callback(fps);
            }
        }
    }
}


struct SampleBuffer {
    samples: Vec<u32>,
    current: usize,
    rolling_sum: u32,
}

impl SampleBuffer {
    fn new() -> SampleBuffer {
        SampleBuffer {
            samples: vec![0; NUM_SAMPLES],
            current: 0,
            rolling_sum: 0,
        }
    }

    fn record(&mut self, sample: u32) {
        self.rolling_sum -= self.samples[self.current];
        self.rolling_sum += sample;
        self.samples[self.current] = sample;
        self.current += 1;
        self.current %= self.samples.len();
    }

    fn average(&self) -> u32 {
        let avg = self.rolling_sum / self.samples.len() as u32;
        (1. / (avg as f64 * 1e-9)) as u32
    }

}

