use crate::Source;
use crate::SAMPLE_RATE;

pub struct Square {
    phase: f32,
    frequency: f32,
    phase_increment: f32,
}

impl Square {
    pub fn new(frequency: f32) -> Self {
        let phase_increment = frequency / SAMPLE_RATE as f32;
        Self {
            phase: 0.0,
            frequency,
            phase_increment,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.phase_increment = frequency / SAMPLE_RATE as f32;
    }
}

impl Source for Square {
    fn next(&mut self) -> f32 {
        self.phase = (self.phase + self.phase_increment) % 1.0;
        if self.phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }
}

pub struct Sine {}

impl Sine {
    pub fn new(frequency: f32) -> Self {
        todo!();
        Self {}
    }
}

impl Source for Sine {
    fn next(&mut self) -> f32 {
        todo!()
    }
}
