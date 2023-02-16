use crate::consts;

use std::f32::consts::PI;
use std::sync::mpsc;

pub enum SynthEvent {
    Amplitude(f32),
    Pitch(f32),
}

pub struct Synth {
    amplitude: f32,
    pitch: f32,
    pub buffer: [f32; (consts::PCM_BUFFER_SIZE as usize * consts::CHANNELS as usize)],
    events: mpsc::Receiver<SynthEvent>,
}

impl Synth {
    pub fn new(reciever: mpsc::Receiver<SynthEvent>) -> Self {
        Synth {
            amplitude: 0.01,
            pitch: 440.0,
            buffer: [0.0; consts::PCM_BUFFER_SIZE as usize * consts::CHANNELS as usize],
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
                match i {
                    i if i % 2 == 0 => {
                        // left channel
                        let t = (time + (i / 2)) as f32 / consts::SAMPLE_RATE as f32;
                        let sample = (t * self.pitch * 2.0 * PI).sin();
                        let amplitude = self.amplitude;
                        (sample * amplitude).min(0.1).max(-0.1)
                    }
                    i if i % 2 == 1 => {
                        // right channel
                        let t = (time + ((i - 1) / 2)) as f32 / consts::SAMPLE_RATE as f32;
                        let sample = (t * self.pitch * 2.0 * PI).cos();
                        let amplitude = self.amplitude;
                        (sample * amplitude).min(0.1).max(-0.1)
                    }
                    _ => unreachable!(),
                }
            })
            .collect::<Vec<f32>>()
            .try_into()
            .unwrap()
    }
}
