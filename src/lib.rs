pub mod dissection;
pub mod mess;

pub mod buffer;
pub use buffer::Buffer;

pub mod effects;
pub use effects::Gain;

pub mod sources;
pub use sources::Sine;

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

pub const SAMPLE_RATE: u32 = 44100;
pub const SAMPLES_PER_CHANNEL: usize = 513 * 4;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
