use std::fs::File;
use std::io::Write;

use audio::dissection::bus::AudioBus;
use audio::dissection::effects::{Attenuate, Effect};
use audio::dissection::engine::{SharedSoundContext, SharedSoundEngine};
use audio::dissection::source::{self, SoundSource};

use audio::mess::{amplitude_to_db, db_to_amplitude};
use audio::mess::delay::Delay;
use audio::mess::melody::semitone_to_frequency;
use audio::{lerp, SAMPLE_RATE};

use audio::{Gain, Pipeline, Square};

fn main() {
    use audio::Buffer;
    let mut chain = Pipeline::new(Square::new(440.0));
    chain.add_effect(Gain::new(0.05));

    let mut buffer = Buffer::from_source(&mut chain, 3.0);
    for sample in buffer.iter().take((SAMPLE_RATE / 440) as usize * 2) {
        print!("{sample:0.02} ");
    }
    println!();

    let mut peak = audio::peak(&buffer);
    println!("Peak: {}, {}db", peak, amplitude_to_db(peak));

    buffer.normalize(db_to_amplitude(-40.0));
    peak = audio::peak(&buffer);
    println!("Peak: {}, {}db", peak, amplitude_to_db(peak));
}

fn mess_test() {
    for note in 45..70 {
        let freq = semitone_to_frequency(note);
        println!("{note:01} Frequency: {}", freq);
    }

    let mut sine_wave_buffer = sin_buffer(false);

    let mut max_amplitude = audio::peak(&sine_wave_buffer);
    let wanted_change_in_amplitude = 0.005 / max_amplitude;
    println!(
        "Gain: {}, {}db",
        wanted_change_in_amplitude,
        amplitude_to_db(wanted_change_in_amplitude)
    );

    sine_wave_buffer.apply(|s| s * wanted_change_in_amplitude);
    max_amplitude = audio::peak(&sine_wave_buffer);
    println!(
        "Gain: {}, {}db",
        max_amplitude,
        amplitude_to_db(max_amplitude)
    );

    let mut delay = Delay::new(100);
    sine_wave_buffer.apply(|s| delay.process(s));

    let mut file = File::create("sine_wave.wav").unwrap();
    let header = audio::mess::fileio::make_wav_header(
        2,
        audio::SAMPLE_RATE,
        sine_wave_buffer.channel_duration_in_samples() as u32,
    );
    file.write_all(&header).unwrap();
    sine_wave_buffer.write_pcm(file).unwrap();
}

fn sound_engine_test() {
    let engine = SharedSoundEngine::new().unwrap();
    let context = SharedSoundContext::new();
    engine.lock().context = context.clone();

    let sine_wave_buffer = sin_buffer(false);

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

use audio::dissection::buffer::Buffer;
pub fn sin_buffer(mono: bool) -> Buffer {
    let sample_rate = 44100u32;
    let seconds = 2.0;
    let total_samples = (seconds * sample_rate as f32) as usize;
    let mut samples: Vec<f32> = Vec::with_capacity(total_samples);
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
    Buffer::new(samples, mono)
}
