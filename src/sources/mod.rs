use crate::Source;

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
