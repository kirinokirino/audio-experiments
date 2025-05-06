

pub mod pool;

pub mod source;
pub mod buffer;
pub mod bus;
pub mod effects;
pub mod engine;

pub const SAMPLE_RATE: u32 = 44100;
pub const SAMPLES_PER_CHANNEL: usize = 513 * 4;


pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
