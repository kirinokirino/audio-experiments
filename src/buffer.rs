use crate::Pipeline;
use crate::SAMPLE_RATE;

pub struct Buffer {}

impl Buffer {
    pub fn from_source(chain: &mut Pipeline) -> Self {
        Self {}
    }

    pub fn normalize(&mut self, amplitude: f32) {}
}
