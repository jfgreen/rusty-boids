use std::time::Instant;

const NUM_SAMPLES: usize = 20;

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

