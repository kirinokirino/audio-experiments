pub mod dissection;
pub mod mess;

pub mod buffer;
pub use buffer::Buffer;

pub mod effects;
pub use effects::Gain;

pub mod sources;
pub use sources::{Sine, Square};

pub const SAMPLE_RATE: u32 = 44100;
pub const SAMPLES_PER_CHANNEL: usize = 513 * 4;

pub struct Pipeline {
    source: Box<dyn Source>,
    effects: Vec<Box<dyn Effect>>,
}

impl Pipeline {
    pub fn new(source: impl Source + 'static) -> Self {
        Self {
            source: Box::new(source),
            effects: Vec::new(),
        }
    }

    pub fn add_effect(&mut self, effect: impl Effect + 'static) {
        self.effects.push(Box::new(effect));
    }
}

impl Source for Pipeline {
    fn next(&mut self) -> f32 {
        let mut sample = self.source.next();
        for effect in &mut self.effects {
            sample = effect.process(sample);
        }
        sample
    }
}

/// A single-sample processor (e.g., gain, delay).
pub trait Effect {
    fn process(&mut self, input: f32) -> f32;
}

/// A sample generator (e.g., oscillator, noise).
pub trait Source {
    /// Generate the next sample.
    fn next(&mut self) -> f32;
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn peak_scalar(samples: &[f32]) -> f32 {
    let mut max = 0.0;
    for &sample in samples {
        if sample.abs() > max {
            max = sample.abs();
        }
    }
    max
}

// Maybe find proper simd capabilities or benchmark/look at compilation output.
const CHUNK_SIZE: usize = 8;

pub fn peak(samples: &[f32]) -> f32 {
    let mut tmp = [0.0; CHUNK_SIZE];

    for chunk in samples.chunks_exact(CHUNK_SIZE) {
        for i in 0..CHUNK_SIZE {
            let abs = chunk[i].abs();
            if abs > tmp[i] {
                tmp[i] = abs;
            }
        }
    }

    // Process the chunk maxes and the remainder using sample_peak
    let remainder = &samples[(samples.len() / CHUNK_SIZE) * CHUNK_SIZE..];
    peak_scalar(&tmp).max(peak_scalar(remainder))
}
