use audio::{
    source::{self, SoundSource},
    SharedSoundEngine, SharedSoundContext,
};

fn main() {
    // let engine = SoundEngine::without_device();
    // if let Err(error) = engine.initialize_audio_output_device() {
    //     panic!("Error while initializing audio output device: {error}");
    // }
    let engine = SharedSoundEngine::new().unwrap();
    let context = SharedSoundContext::new();
    engine.lock().context = context.clone();

    // Create sine wave.
    let sample_rate = 44100;
    let samples: Vec<f32> = {
        let frequency = 440.0;
        let amplitude = 0.15;
        (0..44100)
            .map(|i| {
                amplitude
                    * ((2.0 * std::f32::consts::PI * i as f32 * frequency) / sample_rate as f32)
                        .sin()
            })
            .collect()
    };

    let sine_wave_buffer = audio::buffer::Buffer::new(sample_rate, 1, &samples).unwrap();

    // Create generic source (without spatial effects) using that buffer.
    let mut source = SoundSource::default();
    source.buffer = Some(sine_wave_buffer);
    source.looping = true;
    source.status = source::Status::Playing;

    //dbg!(&source);

    context.lock().add_source(source);

    {
        let sound_state = context.lock();
        //println!("sources {:?}", sound_state.sources());
        println!("listener {:?}", sound_state.listener());
        println!(
            "full_render_duration {:?}",
            sound_state.full_render_duration()
        );
        println!("bus_graph {:#?}", sound_state.bus_graph_ref());
        println!("is_paused {:?}", sound_state.paused);
    }
    std::thread::sleep(std::time::Duration::from_secs(3));
}

// sources: Pool<SoundSource>,
// listener: Listener,
// render_duration: Duration,
// renderer: Renderer,
// bus_graph: AudioBusGraph,
// paused: bool,
