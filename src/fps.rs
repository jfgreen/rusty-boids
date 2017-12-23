use std::time::{Duration, Instant};

const NUM_SAMPLES: usize = 20;

pub struct CachedFpsCounter {
    counter: FpsCounter,
    last_updated_fps: Instant,
    cache_interval: Duration,
    last_shown_fps: u32,
}

impl CachedFpsCounter {
    pub fn new(cache_ms: u64) -> CachedFpsCounter {
        CachedFpsCounter {
            counter: FpsCounter::new(),
            last_updated_fps: Instant::now(),
            cache_interval: Duration::from_millis(cache_ms),
            last_shown_fps: 0,
        }
    }

    pub fn tick<F>(&mut self, callback: F) where F: Fn(u32) {
        self.counter.tick();
        let since_last_update = self.last_updated_fps.elapsed();
        if since_last_update > self.cache_interval {
            self.last_updated_fps = Instant::now();
            let fps = self.counter.current();
            if fps != self.last_shown_fps {
                self.last_shown_fps = fps;
                callback(fps);
            }
        }
    }
}

pub struct FpsCounter {
    last_sampled: Instant,
    samples: SampleBuffer,
}

impl FpsCounter {

    pub fn new() -> FpsCounter {
        FpsCounter{
            last_sampled: Instant::now(),
            samples: SampleBuffer::new(),
        }
    }

    pub fn tick(&mut self) {
        let sample = self.last_sampled.elapsed().subsec_nanos();
        self.last_sampled = Instant::now();
        self.samples.record(sample);
    }

    pub fn current(&self) -> u32 {
        self.samples.average()
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

