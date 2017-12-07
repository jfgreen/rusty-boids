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
        self.samples.record(sample as u32);
    }

    pub fn current(&self) -> u32 {
        self.samples.average()
    }

}


struct SampleBuffer {
    samples: Vec<u32>,
    current: usize,
}

impl SampleBuffer {
    fn new() -> SampleBuffer {
        SampleBuffer {
            samples: vec![0; NUM_SAMPLES],
            current: 0,
        }
    }

    fn record(&mut self, sample: u32) {
        self.samples[self.current] = sample;
        self.current += 1;
        self.current %= self.samples.len();
    }

    fn average(&self) -> u32 {
        //TODO: Better rolling average - look in DSP book
        let avg_ms_frame = self.samples.iter().sum::<u32>() / self.samples.len() as u32;
        (1. / (avg_ms_frame as f64 * 1e-9)) as u32
    }

}

