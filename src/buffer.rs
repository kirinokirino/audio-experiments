use std::{fmt::Debug, time::Duration};

use crate::SAMPLE_RATE;

#[derive(Default, Clone)]
pub struct Buffer {
    is_mono: bool,
    /// Interleaved decoded samples (mono sounds: L..., stereo sounds: LR...)
    pub samples: Vec<f32>,
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let channels = if self.is_mono {
            "Mono"
        } else {
            "Stereo"
        };
        f.debug_struct(format!("Buffer ({channels})").as_str())
            .field("samples", &format!("[..{} samples]", &self.samples.len()))
            .finish()
    }
}

impl Buffer {
    pub fn new(samples: &[f32], is_mono: bool) -> anyhow::Result<Self> {
        if samples.len() % 2 != 0 && !is_mono {
            panic!("Buffer length is not divisible by 2");
        } else {
            Ok(Self {
                samples: samples.to_owned(),
                is_mono
            })
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
            return self.samples.len()
        } else {
            return self.samples.len() / 2
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
