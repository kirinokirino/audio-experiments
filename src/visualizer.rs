use std::sync::mpsc::{self, Receiver, Sender};

use speedy2d::{
    color::Color,
    dimen::{UVec2, Vec2},
    window::{KeyScancode, VirtualKeyCode, WindowHandler, WindowHelper},
    Graphics2D,
};

use crate::{
    audio::audio_thread,
    consts,
    synth::{OscType, Synth, SynthEvent},
};

const AMPLIFY: f32 = 10.0;
pub struct App {
    viewport: UVec2,
    waves: Vec<Waveform>,
    event_senders: Vec<Sender<SynthEvent>>,
    local_synth: Synth,
    time_callback: Receiver<u64>,
    time: u64,
}

impl App {
    pub fn new(window_size: UVec2) -> Self {
        let (time_tx, time_rx) = mpsc::channel::<u64>();
        let (tx, rx) = mpsc::channel::<SynthEvent>();
        std::thread::spawn(move || unsafe {
            audio_thread(Synth::new(rx), time_tx);
        });

        let (tx2, rx) = mpsc::channel::<SynthEvent>();
        let local_synth = Synth::new(rx);

        let waves = vec![Waveform::new(Vec::new()), Waveform::new(Vec::new())];
        Self {
            viewport: window_size,
            waves,
            event_senders: vec![tx, tx2],
            local_synth,
            time_callback: time_rx,
            time: 0,
        }
    }

    fn update(&mut self) {
        while let Ok(time) = self.time_callback.try_recv() {
            self.time = time
        }
        self.local_synth.handle_events();
        self.local_synth.fill_buffer(self.time.try_into().unwrap());
        for (wave_idx, wave) in self.waves.iter_mut().enumerate() {
            wave.buffer = self
                .local_synth
                .buffer
                .iter()
                .enumerate()
                .filter_map(|(i, sample)| {
                    if i % consts::CHANNELS as usize == wave_idx {
                        Some(*sample)
                    } else {
                        None
                    }
                })
                .collect();
        }
    }
}

impl WindowHandler for App {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        self.update();
        graphics.clear_screen(Color::from_rgb(0.8, 0.8, 0.8));
        let width = self.viewport.x;
        let segment_size = width as f32 / self.waves[0].buffer.len() as f32;

        for wave in &self.waves {
            let wave: Vec<Vec2> = wave
                .buffer
                .iter()
                .enumerate()
                .map(|(i, sample)| {
                    Vec2::new(
                        segment_size * i as f32,
                        AMPLIFY * sample * self.viewport.y as f32 + (self.viewport.y as f32 / 2.0),
                    )
                })
                .collect();
            for pair in wave.as_slice().windows(2) {
                let (from, to) = (pair[0], pair[1]);
                graphics.draw_line(from, to, 2.0, Color::BLACK);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
        helper.request_redraw();
    }

    fn on_resize(&mut self, _helper: &mut WindowHelper<()>, size_pixels: UVec2) {
        self.viewport = size_pixels;
    }

    fn on_key_down(
        &mut self,
        helper: &mut WindowHelper<()>,
        virtual_key_code: Option<VirtualKeyCode>,
        scancode: KeyScancode,
    ) {
        if let Some(key_code) = virtual_key_code {
            match key_code {
                VirtualKeyCode::Escape => helper.terminate_loop(),
                VirtualKeyCode::Key1 => {
                    for sender in &self.event_senders {
                        sender.send(SynthEvent::OscType(OscType::Sine));
                    }
                }
                VirtualKeyCode::Key2 => {
                    for sender in &self.event_senders {
                        sender.send(SynthEvent::OscType(OscType::Triangle));
                    }
                }
                VirtualKeyCode::Key3 => {
                    for sender in &self.event_senders {
                        sender.send(SynthEvent::OscType(OscType::Sawtooth));
                    }
                }
                VirtualKeyCode::Key4 => {
                    for sender in &self.event_senders {
                        sender.send(SynthEvent::OscType(OscType::Square));
                    }
                }
                key => println!("Key: {key:?}, scancode: {scancode}"),
            }
        }
    }
}

struct Waveform {
    buffer: Vec<f32>,
}

impl Waveform {
    fn new(buffer: Vec<f32>) -> Self {
        Self { buffer }
    }
}
