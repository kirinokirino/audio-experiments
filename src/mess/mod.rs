use crate::buffer::Buffer;

mod conversions;
pub mod fileio;
mod format;
pub mod melody;
pub mod delay;

pub fn amplitude_to_db(amplitude: f32) -> f32 {
    20.0 * amplitude.log10()
}
pub fn db_to_amplitude(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub fn amplitude_over_limit(buffer: &Buffer, limit: f32) -> f32 {
    let mut max = 0.0;
    for sample in &buffer.samples {
        let abs = sample.abs();
        if abs > max {
            max = abs;
        }
    }

    if max > limit {
        limit / max // multiplier to bring `max` down to `limit`
    } else {
        1.0 // no scaling needed
    }
}
