use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use crate::SAMPLE_RATE;
use crate::{Pipeline, Source};

#[derive(Default, Clone)]
pub struct Buffer {
    pub samples: Vec<f32>,
}

impl Buffer {
    pub fn new(samples: Vec<f32>) -> Self {
        Self { samples }
    }

    pub fn from_source(chain: &mut Pipeline, duration: f32) -> Self {
        let samples_total = (SAMPLE_RATE as f32 * duration) as usize;
        let samples = (0..samples_total).map(|_| chain.next()).collect();
        Self { samples }
    }

    pub fn normalize(&mut self, max_amplitude: f32) {
        let current_peak = crate::peak(&self);
        if current_peak > 0.00001 {
            let scale = max_amplitude / current_peak;
            self.apply(|sample| sample * scale);
        }
    }

    /// Applies a function to every sample in-place
    pub fn apply<F>(&mut self, mut f: F)
    where
        F: FnMut(f32) -> f32,
    {
        for s in &mut self.samples {
            *s = f(*s);
        }
    }

    pub fn channel_duration_in_samples(&self) -> usize {
        self.samples.len()
    }

    /// Returns exact time length of the buffer.
    #[inline]
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(
            (self.channel_duration_in_samples() as u64 * 1_000_000_000u64) / SAMPLE_RATE as u64,
        )
    }
}

impl Deref for Buffer {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        &self.samples
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.samples
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(format!("Buffer").as_str())
            .field("samples", &format!("[..{} samples]", &self.samples.len()))
            .finish()
    }
}
