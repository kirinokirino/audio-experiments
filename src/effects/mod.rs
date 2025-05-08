use crate::Effect;

pub struct Gain {
    amplitude: f32,
}

impl Gain {
    pub fn new(amplitude: f32) -> Self {
        Self { amplitude }
    }
}

impl Effect for Gain {
    fn process(&mut self, input: f32) -> f32 {
        input * self.amplitude
    }
}