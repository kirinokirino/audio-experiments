use std::io::{Read, Write};

use crate::{buffer::Buffer, SAMPLE_RATE};

pub fn make_wav_header(num_channels: u16, sample_rate: u32, num_frames: u32) -> [u8; 44] {
    let bits_per_sample = 32u16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample / 8u16) as u32;
    let block_align = num_channels * (bits_per_sample / 8);
    let data_chunk_size = num_frames * block_align as u32;
    let file_size = 36 + data_chunk_size;

    let mut header = [0u8; 44];

    // RIFF chunk descriptor
    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&(file_size).to_le_bytes()); // Chunk size
    header[8..12].copy_from_slice(b"WAVE");

    // fmt sub-chunk
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&(16u32).to_le_bytes()); // Subchunk1Size (16 for PCM)
    header[20..22].copy_from_slice(&(3u16).to_le_bytes()); // AudioFormat (3 = IEEE float)
    header[22..24].copy_from_slice(&(num_channels).to_le_bytes()); // NumChannels
    header[24..28].copy_from_slice(&(sample_rate).to_le_bytes()); // SampleRate
    header[28..32].copy_from_slice(&(byte_rate).to_le_bytes()); // ByteRate
    header[32..34].copy_from_slice(&(block_align).to_le_bytes()); // BlockAlign
    header[34..36].copy_from_slice(&(bits_per_sample).to_le_bytes()); // BitsPerSample

    // data sub-chunk
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&(data_chunk_size).to_le_bytes());

    header
}

impl Buffer {
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        match std::path::Path::new(path).extension() {
            Some(ext) if ext == "wav" => self.save_wav(path),
            Some(ext) if ext == "pcm" => self.save_pcm(path),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported file extension",
            )),
        }
    }

    fn save_wav(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        let header = crate::mess::fileio::make_wav_header(
            self.channel_count() as u16,
            SAMPLE_RATE,
            self.channel_duration_in_samples() as u32,
        );
        file.write_all(&header)?;
        self.write_pcm(file)
    }

    fn save_pcm(&self, path: &str) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        self.write_pcm(file)
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
}
