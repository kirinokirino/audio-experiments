use hound::{self, read_wave_header};
use std::f32::consts::PI;
use std::i16;

mod consts {
    pub const DEVICES: &[&str] = &["default\0", "pipewire\0"];
    pub const SAMPLE_RATE: u32 = 44100;
    pub const CHANNELS: u16 = 2;
    pub const PCM_BUFFER_SIZE: ::std::os::raw::c_ulong = 4096;
}

fn main() {
    let mut audio = AudioContext::new();
    let mut random = std::time::Instant::now().elapsed().as_nanos();

    for _ in (0..440) {
        audio.senders[0].send(SynthEvent::pitch((random % 660 + 220) as f32));
        std::thread::sleep(std::time::Duration::from_millis(10));
        (random, _) = random.overflowing_mul(283457);
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    /*
    let mut reader = hound::WavReader::open("sine.wav").unwrap();
    dbg!(reader.spec(), reader.duration());

    let spec = hound::WavSpec {
        channels: consts::CHANNELS,
        sample_rate: consts::SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create("sine.wav", spec).unwrap();

    let buffer: Vec<f32> = (0..11)
        .map(|i| fill_buffer(i * consts::PCM_BUFFER_SIZE as usize))
        .flatten()
        .collect();

    for (i, sample) in buffer.into_iter().enumerate() {
        let channel = i % consts::CHANNELS as usize;
        match channel {
            // LEFT
            0 => writer.write_sample(sample * 1.0).unwrap(),
            // RIGHT
            1 => writer.write_sample(sample * 1.0).unwrap(),
            _ => unreachable!(),
        }
    }
    writer.finalize().unwrap();
    */
}

// roughly based on http://equalarea.com/paul/alsa-audio.html
use alsa_sys as sys;
use std::sync::mpsc;

struct Synthesizer {
    amplitude: f32,
    pitch: f32,
    pub buffer: [f32; (consts::PCM_BUFFER_SIZE as usize * consts::CHANNELS as usize)],
    events: mpsc::Receiver<SynthEvent>,
}

impl Synthesizer {
    pub fn new(reciever: mpsc::Receiver<SynthEvent>) -> Self {
        Synthesizer {
            amplitude: 0.01,
            pitch: 440.0,
            buffer: [0.0; consts::PCM_BUFFER_SIZE as usize * consts::CHANNELS as usize],
            events: reciever,
        }
    }

    pub fn handle_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                SynthEvent::amplitude(amplitude) => self.amplitude = amplitude,
                SynthEvent::pitch(pitch) => self.pitch = pitch,
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

unsafe fn audio_thread(mut synth: Synthesizer) {
    synth.fill_buffer(0);
    let mut time = 0u64;

    let pcm_handle = setup_pcm_device();

    loop {
        synth.handle_events();
        // Wait for PCM to be ready for next write (no timeout)
        if sys::snd_pcm_wait(pcm_handle, -1) < 0 {
            panic!("PCM device is not ready");
        }

        // // find out how much space is available for playback data
        // teoretically it should reduce latency - we will fill a minimum amount of
        // frames just to keep alsa busy and will be able to mix some fresh sounds
        // it does, but also randmly panics sometimes

        // let frames_to_deliver = sys::snd_pcm_avail_update(pcm_handle);
        // println!("{}", frames_to_deliver);
        // let frames_to_deliver = if frames_to_deliver > consts::PCM_BUFFER_SIZE as _ {
        //     consts::PCM_BUFFER_SIZE as i64
        // } else {
        //     frames_to_deliver
        // };

        let frames_to_deliver = consts::PCM_BUFFER_SIZE as i64;

        // ask mixer to fill the buffer
        // TODO: mixer.fill_audio_buffer(&mut buffer, frames_to_deliver as usize);
        synth.fill_buffer(time as usize);

        // send filled buffer back to alsa
        let frames_writen = sys::snd_pcm_writei(
            pcm_handle,
            synth.buffer.as_ptr() as *const _,
            frames_to_deliver as _,
        );
        if frames_writen == -libc::EPIPE as ::std::os::raw::c_long {
            println!("Underrun occured: -EPIPE, attempting recover");

            sys::snd_pcm_recover(pcm_handle, frames_writen as _, 0);
        }

        if frames_writen > 0 && frames_writen != frames_to_deliver as _ {
            println!("Underrun occured: frames_writen != frames_to_deliver, attempting recover");

            sys::snd_pcm_recover(pcm_handle, frames_writen as _, 0);
        }
        time += consts::PCM_BUFFER_SIZE;
    }
}

unsafe fn setup_pcm_device() -> *mut sys::snd_pcm_t {
    let mut pcm_handle = std::ptr::null_mut();

    // Open the PCM device in playback mode
    if !consts::DEVICES.iter().any(|device| {
        sys::snd_pcm_open(
            &mut pcm_handle,
            device.as_ptr() as _,
            sys::SND_PCM_STREAM_PLAYBACK,
            0,
        ) >= 0
    }) {
        panic!("Can't open PCM device.");
    }

    let mut hw_params: *mut sys::snd_pcm_hw_params_t = std::ptr::null_mut();
    sys::snd_pcm_hw_params_malloc(&mut hw_params);
    sys::snd_pcm_hw_params_any(pcm_handle, hw_params);

    if sys::snd_pcm_hw_params_set_access(pcm_handle, hw_params, sys::SND_PCM_ACCESS_RW_INTERLEAVED)
        < 0
    {
        panic!("Can't set interleaved mode");
    }

    if sys::snd_pcm_hw_params_set_format(pcm_handle, hw_params, sys::SND_PCM_FORMAT_FLOAT_LE) < 0 {
        panic!("Can't set SND_PCM_FORMAT_FLOAT_LE format");
    }
    if sys::snd_pcm_hw_params_set_buffer_size(pcm_handle, hw_params, consts::PCM_BUFFER_SIZE) < 0 {
        panic!("Cant's set buffer size");
    }
    if sys::snd_pcm_hw_params_set_channels(pcm_handle, hw_params, consts::CHANNELS.into()) < 0 {
        panic!("Can't set channels number.");
    }

    let mut rate = consts::SAMPLE_RATE;
    if sys::snd_pcm_hw_params_set_rate_near(pcm_handle, hw_params, &mut rate, std::ptr::null_mut())
        < 0
    {
        panic!("Can't set rate.");
    }

    // Write parameters
    if sys::snd_pcm_hw_params(pcm_handle, hw_params) < 0 {
        panic!("Can't set harware parameters.");
    }
    sys::snd_pcm_hw_params_free(hw_params);

    // tell ALSA to wake us up whenever AudioContext::PCM_BUFFER_SIZE or more frames
    //   of playback data can be delivered. Also, tell
    //   ALSA that we'll start the device ourselves.
    let mut sw_params: *mut sys::snd_pcm_sw_params_t = std::ptr::null_mut();

    if sys::snd_pcm_sw_params_malloc(&mut sw_params) < 0 {
        panic!("cannot allocate software parameters structure");
    }
    if sys::snd_pcm_sw_params_current(pcm_handle, sw_params) < 0 {
        panic!("cannot initialize software parameters structure");
    }

    // if sys::snd_pcm_sw_params_set_avail_min(
    //     pcm_handle,
    //     sw_params,
    //     AudioContext::PCM_BUFFER_SIZE,
    // ) < 0
    // {
    //     panic!("cannot set minimum available count");
    // }
    if sys::snd_pcm_sw_params_set_start_threshold(pcm_handle, sw_params, 0) < 0 {
        panic!("cannot set start mode");
    }
    if sys::snd_pcm_sw_params(pcm_handle, sw_params) < 0 {
        panic!("cannot set software parameters");
    }
    sys::snd_pcm_sw_params_free(sw_params);

    if sys::snd_pcm_prepare(pcm_handle) < 0 {
        panic!("cannot prepare audio interface for use");
    }

    pcm_handle
}

pub struct AudioContext {
    pub senders: Vec<mpsc::Sender<SynthEvent>>,
}

impl AudioContext {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<SynthEvent>();
        std::thread::spawn(move || unsafe {
            audio_thread(Synthesizer::new(rx));
        });

        let mut senders = Vec::new();
        senders.push(tx);
        Self { senders }
    }
}

pub enum SynthEvent {
    amplitude(f32),
    pitch(f32),
}
