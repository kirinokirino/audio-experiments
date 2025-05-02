use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[derive(Debug)]
pub enum DataSource {
    /// Raw samples in interleaved format with specified sample rate and channel count. Can be used for procedural
    /// sounds.
    /// # Notes
    /// Cannot be used with streaming buffers - it makes no sense to stream data that is already loaded into memory.
    Raw {
        sample_rate: usize,
        /// Total amount of channels.
        channel_count: usize,
        /// Raw samples in interleaved format. Count of samples must be multiple to channel count, otherwise you'll
        /// get error at attempt to use such buffer.
        samples: Vec<f32>,
    },
}

#[derive(Debug, Default, Clone)]
pub struct Samples(pub Vec<f32>);

impl Deref for Samples {
    type Target = Vec<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Samples {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct Buffer {
    /// Interleaved decoded samples (mono sounds: L..., stereo sounds: LR...)
    pub samples: Samples,
    pub channel_count: usize,
    pub sample_rate: usize,
    pub channel_duration_in_samples: usize,
}

impl Buffer {
    /// Data source with raw samples must have sample count multiple of channel count
    pub fn new(source: DataSource) -> anyhow::Result<Self> {
        match source {
            DataSource::Raw {
                sample_rate,
                channel_count,
                samples,
            } => {
                if channel_count < 1 || channel_count > 2 || samples.len() % channel_count != 0 {
                    Err(anyhow::anyhow!(
                        "Channel count != 1 or 2, found: {channel_count}"
                    ))
                } else if samples.len() % channel_count != 0 {
                    Err(anyhow::anyhow!(
                        "Every channel must have a sample, samples % channel_count = {}",
                        samples.len() % channel_count
                    ))
                } else {
                    Ok(Self {
                        channel_duration_in_samples: samples.len() / channel_count,
                        samples: Samples(samples),
                        channel_count,
                        sample_rate,
                    })
                }
            }
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
