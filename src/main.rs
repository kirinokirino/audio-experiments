use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use dasp_ring_buffer::Fixed;
use tinyaudio::prelude::*;

#[cfg(not(feature = "tinyaudio"))]
mod audio;

mod synth;

mod consts {
    pub const DEVICES: &[&str] = &["default\0", "pipewire\0"];
    pub const SAMPLE_RATE: u32 = 44100;
    pub const CHANNELS: u16 = 2;
    pub const PCM_BUFFER_SIZE: ::std::os::raw::c_ulong = 4096 / 8;
}

#[cfg(feature = "visualizer")]
mod visualizer;
#[cfg(feature = "visualizer")]
fn main() {
    use glam::UVec2;
    use speedy2d::{
        window::{WindowCreationOptions, WindowPosition, WindowSize},
        Window,
    };

    use visualizer::App;

    const WINDOW_WIDTH: u32 = 600;
    const WINDOW_HEIGHT: u32 = 480;
    let window_size = UVec2::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let window_pixels = WindowSize::PhysicalPixels(window_size);
    let window = Window::new_with_options(
        "FLOATING",
        WindowCreationOptions::new_windowed(window_pixels, Some(WindowPosition::Center))
            .with_decorations(false)
            .with_transparent(true),
    )
    .expect("Wasn't able to create a window!");
    window.run_loop(App::new(window_size));
}

fn main() {
    let mut engine = AudioEngineInternal::new(
        consts::CHANNELS as usize,
        consts::SAMPLE_RATE as usize,
        consts::PCM_BUFFER_SIZE as usize,
    );

    engine.render();
    dbg!(engine.buffer.lock().unwrap()[1]);
    engine.start();
    dbg!(engine.buffer.lock().unwrap()[1]);

    std::thread::sleep(std::time::Duration::from_secs(5));
}

#[derive(Clone)]
pub struct AudioEngine(Arc<Mutex<AudioEngineInternal>>);

impl AudioEngine {
    pub fn start(&mut self) {
        let channels = self.channels.clone();
        let output_parameters = OutputDeviceParameters {
            channels_count: channels,
            sample_rate: self.sample_rate,
            channel_sample_count: self.sample_count,
        };

        let buffer = self.buffer.clone();

        let device = run_output_device(output_parameters, {
            move |output_buffer| {
                for (output, input) in output_buffer
                    .chunks_mut(channels)
                    .zip(buffer.lock().unwrap().iter())
                {
                    for channel in output {
                        *channel = *input;
                    }
                }
            }
        })
        .unwrap();

        self.device = Some(device);
    }
}

struct AudioEngineInternal {
    clock: f64,

    channels: usize,
    sample_rate: usize,
    sample_count: usize,
    buffer: Arc<Mutex<Fixed<[f32; 512]>>>,

    device: Option<Box<dyn BaseAudioOutputDevice>>,
}

impl AudioEngineInternal {
    fn new(channels: usize, sample_rate: usize, sample_count: usize) -> Self {
        let buffer = Arc::from(Mutex::from(Fixed::from([0.0; 512])));
        Self {
            clock: 0.0,
            channels,
            sample_rate,
            sample_count,
            buffer,
            device: None,
        }
    }

    pub fn render(&mut self) {
        for output in self.buffer.lock().unwrap().iter_mut() {
            self.clock = (self.clock + 1.0) % self.sample_rate as f64;
            let frequency = 440.0;
            let volume = 0.3;
            let value = (self.clock * frequency * std::f64::consts::TAU / self.sample_rate as f64)
                .sin()
                * volume;

            *output = value as f32;
        }
    }
}
