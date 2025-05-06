use std::{
    fmt::Debug,
    io::{Read, Write},
    time::Duration,
};

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

    pub fn write_pcm<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        let ptr = self.samples.as_ptr();
        let len = self.samples.len() * std::mem::size_of::<f32>();

        // Safety:
        // - ptr is aligned
        // - data is valid and read-only
        // - f32 -> u8 aliasing is safe for read-only
        let bytes: &[u8] = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };

        writer.write_all(bytes)
    }

    pub fn read_pcm<R: Read>(mut reader: R, is_mono: bool) -> anyhow::Result<Self> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;

        if bytes.len() % 4 != 0 {
            return Err(anyhow::anyhow!(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Byte count not divisible by 4"
            )));
        }

        let samples: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|chunk| {
                let array: [u8; 4] = chunk.try_into().unwrap();
                f32::from_le_bytes(array)
            })
            .collect();

        Ok(Buffer::new(samples, is_mono))
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
