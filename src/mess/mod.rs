use crate::buffer::Buffer;

mod conversions;
pub mod fileio;
mod format;
pub mod melody;

pub fn amplitude_to_db(amplitude: f32) -> f32 {
    20.0 * amplitude.log10()
}
pub fn db_to_amplitude(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub fn amplitude_over_limit(buffer: &Buffer, limit: f32) -> f32 {
    let mut max = 0.0;
    for sample in &buffer.samples {
        if sample.abs() > max {
            max = sample.abs();
        }
    }
    if max > limit {
        return max - limit;
    }
    0.0
}