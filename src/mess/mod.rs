use crate::dissection::buffer::Buffer;

mod conversions;
pub mod delay;
pub mod fileio;
mod format;
pub mod melody;

pub fn amplitude_to_db(amplitude: f32) -> f32 {
    20.0 * amplitude.log10()
}
pub fn db_to_amplitude(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
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
