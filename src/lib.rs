use glam::{Mat3, Vec3};

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

mod pool;
use pool::handle::Handle;
use pool::Pool;

pub mod source;
use source::{SoundSource, Status};
pub mod buffer;
mod bus;
use bus::AudioBusGraph;

mod effects;

pub const SAMPLE_RATE: u32 = 44100;
pub const SAMPLES_PER_CHANNEL: usize = 513 * 4;

pub struct SoundEngine {
    pub context: SharedSoundContext,
    output_device: Option<tinyaudio::OutputDevice>,
    internal_buffer: Vec<(f32, f32)>,
    buffer_size: usize,
}

#[derive(Clone)]
pub struct SharedSoundEngine(Arc<Mutex<SoundEngine>>);

impl SharedSoundEngine {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let buffer_size = SAMPLES_PER_CHANNEL;
        let engine = Self(Arc::new(Mutex::new(SoundEngine {
            context: Default::default(),
            output_device: None,
            internal_buffer: vec![(0.0, 0.0); buffer_size],
            buffer_size,
        })));
        let state = engine.clone();

        let device = tinyaudio::run_output_device(
            tinyaudio::OutputDeviceParameters {
                sample_rate: SAMPLE_RATE as usize,
                channels_count: 2,
                channel_sample_count: SAMPLES_PER_CHANNEL,
            },
            move |buf| SharedSoundEngine::render_callback(buf, &state),
        )?;
        engine.lock().output_device = Some(device);
        Ok(engine)
    }
    pub fn lock(&self) -> MutexGuard<SoundEngine> {
        self.0.lock().unwrap()
    }
    fn render_callback(buf: &mut [f32], engine: &SharedSoundEngine) {
        let mut engine = engine.lock();
        // engine.context.clone().lock().mock_render(&mut engine.internal_buffer);
        engine.context.clone().lock().render(&mut engine.internal_buffer);
        
        // Copy to tinyaudio's buffer
        let stereo_samples = buf.len() / 2;
        let output_device_buffer = unsafe {
            std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut (f32, f32), stereo_samples)
        };

        output_device_buffer.copy_from_slice(&engine.internal_buffer[..stereo_samples]);
    }
}

#[derive(Clone, Default, Debug)]
pub struct SharedSoundContext {
    pub state: Option<Arc<Mutex<SoundContext>>>,
}

impl SharedSoundContext {
    /// Creates new instance of context. Internally context starts new thread which will call render all
    /// sound source and send samples to default output device. This method returns `Arc<Mutex<Context>>`
    /// because separate thread also uses context.
    pub fn new() -> Self {
        Self {
            state: Some(Arc::new(Mutex::new(SoundContext {
                sources: Pool::new(),
                listener: Listener::new(),
                render_duration: Default::default(),
                bus_graph: AudioBusGraph::new(),
                paused: false,
            }))),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, SoundContext> {
        self.state.as_ref().unwrap().lock().unwrap()
    }
}

/// Internal state of context.
#[derive(Default, Debug, Clone)]
pub struct SoundContext {
    sources: Pool<SoundSource>,
    listener: Listener,
    render_duration: Duration,
    bus_graph: AudioBusGraph,
    pub paused: bool,
}

impl SoundContext {
    /// Returns amount of time context spent on rendering all sound sources.
    pub fn full_render_duration(&self) -> Duration {
        self.render_duration
    }
    /// Adds new sound source and returns handle of it by which it can be accessed later on.
    pub fn add_source(&mut self, source: SoundSource) -> Handle<SoundSource> {
        self.sources.spawn(source)
    }

    /// Removes sound source from the context.
    pub fn remove_source(&mut self, source: Handle<SoundSource>) {
        self.sources.free(source);
    }

    /// Returns shared reference to a pool with all sound sources.
    pub fn sources(&self) -> &Pool<SoundSource> {
        &self.sources
    }

    /// Returns mutable reference to a pool with all sound sources.
    pub fn sources_mut(&mut self) -> &mut Pool<SoundSource> {
        &mut self.sources
    }

    /// Returns shared reference to sound source at given handle. If handle is invalid, this method will panic.
    pub fn source(&self, handle: Handle<SoundSource>) -> &SoundSource {
        self.sources.borrow(handle)
    }

    /// Checks whether a handle to a sound source is valid or not.
    pub fn is_valid_handle(&self, handle: Handle<SoundSource>) -> bool {
        self.sources.is_valid_handle(handle)
    }

    /// Returns mutable reference to sound source at given handle. If handle is invalid, this method will panic.
    pub fn source_mut(&mut self, handle: Handle<SoundSource>) -> &mut SoundSource {
        self.sources.borrow_mut(handle)
    }

    /// Returns mutable reference to sound source at given handle. If handle is invalid, this method will panic.
    pub fn try_get_source_mut(&mut self, handle: Handle<SoundSource>) -> Option<&mut SoundSource> {
        self.sources.try_borrow_mut(handle)
    }

    /// Returns shared reference to listener. Engine has only one listener.
    pub fn listener(&self) -> &Listener {
        &self.listener
    }

    /// Returns mutable reference to listener. Engine has only one listener.
    pub fn listener_mut(&mut self) -> &mut Listener {
        &mut self.listener
    }

    /// Returns a reference to the audio bus graph.
    pub fn bus_graph_ref(&self) -> &AudioBusGraph {
        &self.bus_graph
    }

    /// Returns a reference to the audio bus graph.
    pub fn bus_graph_mut(&mut self) -> &mut AudioBusGraph {
        &mut self.bus_graph
    }

    pub fn mock_render(&mut self, output_buffer: &mut [(f32, f32)]) {
        static mut PHASE: f32 = 0.0;
        const FREQ: f32 = 440.0; // A4 note
        const SAMPLE_RATE: f32 = 44100.0;
        
        for (left, right) in output_buffer {
            unsafe {
                let sample = (PHASE * 2.0 * std::f32::consts::PI).sin() * 0.2;
                *left = sample;
                *right = sample;
                PHASE = (PHASE + FREQ / SAMPLE_RATE) % 1.0;
            }
        }
    }
    pub fn render(&mut self, output_device_buffer: &mut [(f32, f32)]) {
        // Clear output first so we can detect if audio is actually being written
        output_device_buffer.fill((0.0, 0.0));
        println!("[Audio] Render started, buffer len: {}", output_device_buffer.len());
    
        let last_time = Instant::now();
    
        if self.paused {
            println!("[Audio] System paused - no processing");
            return;
        }
    
        // Check sources
        let active_sources: usize = self.sources.iter()
            .filter(|s| s.status == Status::Playing)
            .count();
        println!("[Audio] Active sources: {}", active_sources);
    
        self.sources.retain(|source| {
            let done = source.play_once && source.status == Status::Stopped;
            if done {
                println!("[Audio] Removing finished source");
            }
            !done
        });
    
        // Verify bus graph
        println!("[Audio] Beginning bus graph render");
        self.bus_graph.begin_render(output_device_buffer.len());
    
        // Process each active source
        for source in self.sources.iter_mut().filter(|s| s.status == Status::Playing) {
            println!("[Audio] Processing source -> bus '{}'", source.bus);
            
            if let Some(bus_input_buffer) = self.bus_graph.try_get_bus_input_buffer(&source.bus) {
                println!("[Audio]  Found bus buffer (len: {})", bus_input_buffer.len());
                
                source.render(output_device_buffer.len());
                println!("[Audio]  Source rendered {} samples", source.frame_samples().len());
    
                render_source_default(source, &self.listener, bus_input_buffer);
                
                // Debug: Check if bus buffer was written to
                let written_samples = bus_input_buffer.iter()
                    .filter(|&&s| s != (0.0, 0.0))
                    .count();
                println!("[Audio]  Bus buffer modified samples: {}/{}", written_samples, bus_input_buffer.len());
            } else {
                println!("[Audio]  No bus found for '{}'", source.bus);
            }
        }
    
        // Final mix
        println!("[Audio] Final bus graph mix");
        self.bus_graph.end_render(output_device_buffer);
    
        // Verify output
        let silent_output = output_device_buffer.iter()
            .all(|&s| s == (0.0, 0.0));
        println!("[Audio] Output buffer silent: {}", silent_output);
    
        self.render_duration = Instant::now().duration_since(last_time);
        println!("[Audio] Render completed in {:?}", self.render_duration);
    }
}

/// See module docs.
#[derive(Debug, Clone)]
pub struct Listener {
    basis: Mat3,
    position: Vec3,
}

impl Default for Listener {
    fn default() -> Self {
        Self::new()
    }
}

impl Listener {
    pub fn new() -> Self {
        Self {
            basis: Mat3::IDENTITY,
            position: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    /// Sets new basis from given vectors in left-handed coordinate system.
    /// See `set_basis` for more info.
    pub fn set_orientation_lh(&mut self, look: Vec3, up: Vec3) {
        self.basis = Mat3::from_cols(look.cross(up), up, look)
    }

    /// Sets new basis from given vectors in right-handed coordinate system.
    /// See `set_basis` for more info.
    pub fn set_orientation_rh(&mut self, look: Vec3, up: Vec3) {
        self.basis = Mat3::from_cols(up.cross(look), up, look)
    }

    /// Sets arbitrary basis. Basis defines orientation of the listener in space.
    /// In your application you can take basis of camera in world coordinates and
    /// pass it to this method. If you using HRTF, make sure your basis is in
    /// right-handed coordinate system! You can make fake right-handed basis from
    /// left handed, by inverting Z axis. It is fake because it will work only for
    /// positions (engine interested in positions only), but not for rotation, shear
    /// etc.
    ///
    /// # Notes
    ///
    /// Basis must have mutually perpendicular axes.
    ///
    /// ```
    /// use fyrox_sound::listener::Listener;
    /// use fyrox_sound::algebra::{Mat3, UnitQuaternion, Vec3};
    /// use fyrox_sound::math::{Matrix4Ext};
    ///
    /// fn orient_listener(listener: &mut Listener) {
    ///     let basis = UnitQuaternion::from_axis_angle(&Vec3::y_axis(), 45.0f32.to_radians()).to_homogeneous().basis();
    ///     listener.set_basis(basis);
    /// }
    /// ```
    pub fn set_basis(&mut self, matrix: Mat3) {
        self.basis = matrix;
    }

    /// Returns shared reference to current basis.
    pub fn basis(&self) -> &Mat3 {
        &self.basis
    }

    /// Sets current position in world space.
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Returns position of listener.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Returns up axis from basis. up
    pub fn up_axis(&self) -> Vec3 {
        let m = self.basis.to_cols_array();
        Vec3::new(m[3], m[4], m[5])
    }

    /// Returns look axis from basis. look
    pub fn look_axis(&self) -> Vec3 {
        let m = self.basis.to_cols_array();
        Vec3::new(m[6], m[7], m[8])
    }

    /// Returns ear axis from basis. side
    pub fn ear_axis(&self) -> Vec3 {
        let m = self.basis.to_cols_array();
        Vec3::new(m[0], m[1], m[2])
    }
}

fn render_with_params(
    source: &mut SoundSource,
    left_gain: f32,
    right_gain: f32,
    mix_buffer: &mut [(f32, f32)],
) {
    let last_left_gain = *source.last_left_gain.get_or_insert(left_gain);
    let last_right_gain = *source.last_right_gain.get_or_insert(right_gain);

    if last_left_gain != left_gain || last_right_gain != right_gain {
        let step = 1.0 / mix_buffer.len() as f32;
        let mut t = 0.0;
        for ((out_left, out_right), &(raw_left, raw_right)) in
            mix_buffer.iter_mut().zip(source.frame_samples())
        {
            // Interpolation of gain is very important to remove clicks which appears
            // when gain changes by significant value between frames.
            *out_left += lerp(last_left_gain, left_gain, t) * raw_left;
            *out_right += lerp(last_right_gain, right_gain, t) * raw_right;

            t += step;
        }
    } else {
        for ((out_left, out_right), &(raw_left, raw_right)) in
            mix_buffer.iter_mut().zip(source.frame_samples())
        {
            // Optimize the common case when the gain did not change since the last call.
            *out_left += left_gain * raw_left;
            *out_right += right_gain * raw_right;
        }
    }
}

pub fn render_source_default(
    source: &mut SoundSource,
    listener: &Listener,
    mix_buffer: &mut [(f32, f32)],
) {
    let panning = lerp(
        source.panning,
        source.calculate_panning(listener),
        source.spatial_blend,
    );
    let left_gain = source.gain * (1.0 + panning);
    let right_gain = source.gain * (1.0 - panning);
    render_with_params(source, left_gain, right_gain, mix_buffer);
    source.last_left_gain = Some(left_gain);
    source.last_right_gain = Some(right_gain);
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
