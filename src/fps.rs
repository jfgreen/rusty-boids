// TODO: Average over n samples?

use std::time::{Duration, Instant};

pub struct FpsCounter {
    last_sampled: Instant,
    sample_interval: Duration,
    frames_this_sample: u32,
}

impl FpsCounter {

    pub fn new() -> FpsCounter {
        FpsCounter{
            last_sampled: Instant::now(),
            sample_interval: Duration::from_secs(1),
            frames_this_sample: 0,
        }

    }

    pub fn tick<F>(&mut self, callback: F)
        where F: Fn(u32) {
        self.frames_this_sample += 1;
        if self.last_sampled.elapsed() > self.sample_interval {
            self.last_sampled = Instant::now();
            callback(self.frames_this_sample);
            self.frames_this_sample = 0;
        }

    }

}
