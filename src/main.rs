use audio::{
    bus::AudioBus, effects::{Attenuate, Effect}, lerp, source::{self, SoundSource}, SharedSoundContext, SharedSoundEngine
};

fn main() {
    let engine = SharedSoundEngine::new().unwrap();
    let context = SharedSoundContext::new();
    engine.lock().context = context.clone();

    // Create sine wave.
    let sample_rate = 44100u32;
    let seconds = 10.0;
    let total_samples = (seconds * sample_rate as f32) as usize;
    let mut samples: Vec<f32> = Vec::with_capacity(total_samples as usize);
    {
        let frequency = 440.0;
        let amplitude = 0.05;
        for i in 0..total_samples {
            let t1 = (i as f32 / (sample_rate as f32 * 0.25)).sin() + 1.0;
            let t2 = ((i as f32 + 10.0) / (sample_rate as f32 * 0.25)).sin() + 1.0;
            let sine1 = amplitude
                * ((std::f32::consts::TAU * i as f32 * frequency) / sample_rate as f32).sin();
            let sine2 = amplitude
                * ((std::f32::consts::TAU * i as f32 * frequency) / sample_rate as f32).sin();
            let left_sample = lerp(sine1, sine2, t1);
            let right_sample = lerp(sine1, sine2, t1);

            samples.push(left_sample);
            samples.push(right_sample);
        }
    }

    let sine_wave_buffer = audio::buffer::Buffer::new(sample_rate, 2, &samples).unwrap();
    
    {
        let mut effects_bus = AudioBus::new("Effects".to_string());
        let effect = Effect::Attenuate(Attenuate::new(0.25));
        effects_bus.add_effect(effect);
        let mut context = context.lock();
        let bus_graph = context.bus_graph_mut();
        let master_bus = bus_graph.primary_bus_handle();
        bus_graph.add_bus(effects_bus, master_bus);
    }

    // Create generic source (without spatial effects) using that buffer.
    let mut source = SoundSource::default();
    source.buffer = Some(sine_wave_buffer);
    source.looping = true;
    source.status = source::Status::Playing;
    source.set_bus("Effects");
    //dbg!(&source);
    let source_handle = context.lock().add_source(source);

    {
        let sound_state = context.lock();
        let source = sound_state.sources().try_borrow(source_handle).unwrap();
        //println!("source  {:?}", source);
        println!(
            "full_render_duration {:?}",
            sound_state.full_render_duration()
        );
        println!("bus_graph {:#?}", sound_state.bus_graph_ref());
        println!("is_paused {:?}", sound_state.paused);
    }

    std::thread::sleep(std::time::Duration::from_secs(3));

    context.lock().source_mut(source_handle).set_pitch(0.5);
    println!(
        "source  {:?}",
        context.lock().sources().try_borrow(source_handle).unwrap()
    );
    std::thread::sleep(std::time::Duration::from_secs(3));
    context.lock().source_mut(source_handle).set_pitch(1.5);
    println!(
        "source  {:?}",
        context.lock().sources().try_borrow(source_handle).unwrap()
    );
    std::thread::sleep(std::time::Duration::from_secs(3));
    context.lock().source_mut(source_handle).set_pitch(0.25);
    println!(
        "source  {:?}",
        context.lock().sources().try_borrow(source_handle).unwrap()
    );
    std::thread::sleep(std::time::Duration::from_secs(3));
    context.lock().source_mut(source_handle).set_pitch(0.75);
    std::thread::sleep(std::time::Duration::from_secs(3));
}
