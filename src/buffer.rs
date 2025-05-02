use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct Buffer {
    /// Interleaved decoded samples (mono sounds: L..., stereo sounds: LR...)
    pub samples: Vec<f32>,
    pub channel_count: u8,
    pub sample_rate: u32,
    pub channel_duration_in_samples: usize,
}

impl Buffer {
    pub fn new(sample_rate: u32, channel_count: u8, samples: &[f32]) -> anyhow::Result<Self> {
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
                sample_rate,
            })
        }
    }

    /// Returns exact time length of the buffer.
    #[inline]
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(
            (self.channel_duration_in_samples as u64 * 1_000_000_000u64) / self.sample_rate as u64,
        )
    }
}
