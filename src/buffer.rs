use std::{fmt::Debug, time::Duration};

use crate::SAMPLE_RATE;

#[derive(Default, Clone)]
pub struct Buffer {
    /// Interleaved decoded samples (mono sounds: L..., stereo sounds: LR...)
    pub samples: Vec<f32>,
    pub channel_count: u8,
    pub channel_duration_in_samples: usize,
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("samples", &format!("[..{} samples]", &self.samples.len()))
            .field("channel_count", &self.channel_count)
            .field(
                "channel_duration_in_samples",
                &self.channel_duration_in_samples,
            )
            .finish()
    }
}

impl Buffer {
    pub fn new(channel_count: u8, samples: &[f32]) -> anyhow::Result<Self> {
        if channel_count < 1 || channel_count > 2 {
            Err(anyhow::anyhow!(
                "Channel count != 1 or 2, found: {channel_count}"
            ))
        //} else if samples.len() % channel_count != 0 {
        } else if samples.len() % 2 != 0 {
            Err(anyhow::anyhow!(
                "Every channel must have a sample, samples % channel_count = {}",
                samples.len() % usize::from(channel_count)
            ))
        } else {
            Ok(Self {
                channel_duration_in_samples: samples.len() / usize::from(channel_count),
                samples: samples.to_owned(),
                channel_count,
            })
        }
    }

    /// Returns exact time length of the buffer.
    #[inline]
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(
            (self.channel_duration_in_samples as u64 * 1_000_000_000u64) / SAMPLE_RATE as u64,
        )
    }
}
