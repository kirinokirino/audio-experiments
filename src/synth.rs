use crate::consts;

use dasp_ring_buffer::Fixed;
use std::f32::consts::PI;
use std::sync::mpsc;

pub enum SynthEvent {
    Amplitude(f32),
    Pitch(f32),
}

pub struct Synth {
    amplitude: f32,
    pitch: f32,
    pub buffer: Fixed<Vec<f32>>, //[f32; consts::PCM_BUFFER_SIZE as usize * consts::CHANNELS as usize],
    events: mpsc::Receiver<SynthEvent>,
}

impl Synth {
    pub fn new(reciever: mpsc::Receiver<SynthEvent>) -> Self {
        let buffer = Fixed::from(vec![
            0.0;
            consts::PCM_BUFFER_SIZE as usize
                * consts::CHANNELS as usize
        ]);
        Synth {
            amplitude: 0.01,
            pitch: 440.0,
            buffer,
            events: reciever,
        }
    }

    pub fn handle_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                SynthEvent::Amplitude(amplitude) => self.amplitude = amplitude,
                SynthEvent::Pitch(pitch) => self.pitch = pitch,
            }
        }
    }

    pub fn fill_buffer(&mut self, time: usize) {
        self.buffer = (0..consts::PCM_BUFFER_SIZE as usize * 2)
            .map(|i| {
                let channels = consts::CHANNELS as usize;
                let channel = i % channels;
                let sample_idx = time + (i / channels);
                let time = time_from_sample_idx(sample_idx);
                let sample = sine(time, self.pitch);
                limit(sample * self.amplitude, 0.1)
            })
            .collect::<Vec<f32>>()
            .try_into()
            .unwrap()
    }
}

fn time_from_sample_idx(sample_idx: usize) -> f32 {
    sample_idx as f32 / consts::SAMPLE_RATE as f32
}

fn sine(time: f32, pitch: f32) -> f32 {
    (time * pitch * 2.0 * PI).sin()
}

fn limit(sample: f32, to: f32) -> f32 {
    sample.min(to).max(-to)
}
