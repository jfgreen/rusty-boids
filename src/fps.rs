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

    pub fn poll<F>(&mut self, counter: &FpsCounter, handler: F) where F: Fn(u32) {
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
    samples: Vec<u32>,
    current_sample: usize,
    rolling_dt_sum: u32,
}

impl FpsCounter {

    pub fn new() -> FpsCounter {
        FpsCounter {
            last_sampled: Instant::now(),
            samples: vec![0; NUM_SAMPLES],
            current_sample: 0,
            rolling_dt_sum: 0,
        }
    }

    pub fn tick(&mut self) {
        let dt = self.last_sampled.elapsed();
        self.last_sampled = Instant::now();
        //TODO: Combine as_secs with subsec_nanos
        //self.last_delta = dt.as_secs() as f64
        //                + dt.subsec_nanos() as f64 * 1e-9;
        self.record(dt.subsec_nanos());
    }

    pub fn average_delta(&self) -> u32 {
        self.rolling_dt_sum / self.samples.len() as u32
    }

    pub fn average_fps(&self) -> u32 {
        (1. / (self.average_delta() as f64 * 1e-9)) as u32
    }

    fn record(&mut self, sample: u32) {
        self.rolling_dt_sum -= self.samples[self.current_sample];
        self.rolling_dt_sum += sample;
        self.samples[self.current_sample] = sample;
        self.current_sample += 1;
        self.current_sample %= self.samples.len();
    }
}

