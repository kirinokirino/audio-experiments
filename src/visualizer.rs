use std::sync::mpsc::{self, Receiver, Sender};

use speedy2d::{
    color::Color,
    dimen::{UVec2, Vec2},
    window::{KeyScancode, VirtualKeyCode, WindowHandler, WindowHelper},
    Graphics2D,
};

use crate::{
    audio::audio_thread,
    synth::{Synth, SynthEvent},
};

const AMPLIFY: f32 = 10.0;
pub struct App {
    viewport: UVec2,
    waves: [Waveform; 2],
    event_senders: Vec<Sender<SynthEvent>>,
    local_synth: Synth,
    time_callback: Receiver<u64>,
    time: u64,
}

impl App {
    pub fn new(window_size: UVec2) -> Self {
        let (time_tx, time_rx) = mpsc::channel::<u64>();
        let mut event_senders = Vec::new();
        let (tx, rx) = mpsc::channel::<SynthEvent>();
        std::thread::spawn(move || unsafe {
            audio_thread(Synth::new(rx), time_tx);
        });
        event_senders.push(tx);

        let (tx, rx) = mpsc::channel::<SynthEvent>();
        let local_synth = Synth::new(rx);
        event_senders.push(tx);

        let waves = [Waveform::new(Vec::new()), Waveform::new(Vec::new())];
        Self {
            viewport: window_size,
            waves,
            event_senders,
            local_synth,
            time_callback: time_rx,
            time: 0,
        }
    }

    fn update(&mut self) {
        loop {
            match self.time_callback.try_recv() {
                Ok(time) => {
                    self.time = time;
                }
                Err(_) => break,
            }
        }
        self.local_synth.fill_buffer(self.time.try_into().unwrap());
        self.waves[0].buffer = self
            .local_synth
            .buffer
            .iter()
            .enumerate()
            .filter_map(|(i, sample)| if i % 2 == 0 { Some(*sample) } else { None })
            .collect();
        self.waves[1].buffer = self
            .local_synth
            .buffer
            .iter()
            .enumerate()
            .filter_map(|(i, sample)| if i % 2 == 1 { Some(*sample) } else { None })
            .collect();
    }
}

impl WindowHandler for App {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        self.update();
        graphics.clear_screen(Color::from_rgb(0.8, 0.8, 0.8));
        let width = self.viewport.x;
        let segment_size = width as f32 / self.waves[0].buffer.len() as f32;

        let wave: Vec<Vec2> = self.waves[0]
            .buffer
            .iter()
            .enumerate()
            .map(|(i, sample)| {
                Vec2::new(
                    segment_size as f32 * i as f32,
                    AMPLIFY * sample * self.viewport.y as f32 + (self.viewport.y as f32 / 2.0),
                )
            })
            .collect();

        let wave2: Vec<Vec2> = self.waves[1]
            .buffer
            .iter()
            .enumerate()
            .map(|(i, sample)| {
                Vec2::new(
                    segment_size as f32 * i as f32,
                    AMPLIFY * sample * self.viewport.y as f32 + (self.viewport.y as f32 / 2.0),
                )
            })
            .collect();

        for pair in wave.as_slice().windows(2) {
            let (from, to) = (pair[0], pair[1]);
            graphics.draw_line(from, to, 2.0, Color::BLACK);
        }
        for pair in wave2.as_slice().windows(2) {
            let (from, to) = (pair[0], pair[1]);
            graphics.draw_line(from, to, 2.0, Color::BLACK);
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
