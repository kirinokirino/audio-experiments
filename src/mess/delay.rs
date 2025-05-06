pub struct Delay {
    buffer: Vec<f32>,
    index: usize,
}

impl Delay {
    pub fn new(length: usize) -> Self {
        Self {
            buffer: vec![0.0; length],
            index: 0,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let out = self.buffer[self.index];
        self.buffer[self.index] = input;
        self.index = (self.index + 1) % self.buffer.len();
        out
    }
}