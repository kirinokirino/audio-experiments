mod audio;
use audio::AudioContext;
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
    use speedy2d::{
        dimen::UVec2,
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

#[cfg(not(feature = "visualizer"))]
fn main() {
    let audio = AudioContext::new();

    for _ in 0..100 {
        if let Err(err) = audio.senders[0].send(SynthEvent::Pitch(440.0)) {
            panic!("{err:?}");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    std::thread::sleep(std::time::Duration::from_secs(1));
}
