use crate::Effect;

pub struct Gain {

}

impl Gain {
    pub fn new(amplitude: f32) -> Self {
        todo!();
        Self {}
    }
}

impl Effect for Gain {
    fn process(&mut self, input: f32) -> f32 {
        todo!()
    }
}