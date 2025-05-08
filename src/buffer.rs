use std::ops::{Deref, DerefMut};
use std::fmt::Debug;
use std::time::Duration;

use crate::SAMPLE_RATE;

#[derive(Default, Clone)]
pub struct Buffer {
    is_mono: bool,
    /// Interleaved decoded samples (mono sounds: L..., stereo sounds: LR...)
    pub samples: Vec<f32>,
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let channels = if self.is_mono { "Mono" } else { "Stereo" };
        f.debug_struct(format!("Buffer ({channels})").as_str())
            .field("samples", &format!("[..{} samples]", &self.samples.len()))
            .finish()
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

impl Buffer {
    pub fn new(mut samples: Vec<f32>, is_mono: bool) -> Self {
        if samples.len() % 2 != 0 && is_mono {
            samples.push(*samples.last().unwrap());
        }
        Self {
            samples: samples.to_owned(),
            is_mono,
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

    pub fn channel_count(&self) -> usize {
        if self.is_mono {
            1
        } else {
            2
        }
    }

    pub fn channel_duration_in_samples(&self) -> usize {
        if self.is_mono {
            self.samples.len()
        } else {
            self.samples.len() / 2
        }
    }

    /// Returns exact time length of the buffer.
    #[inline]
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(
            (self.channel_duration_in_samples() as u64 * 1_000_000_000u64) / SAMPLE_RATE as u64,
        )
    }
}
