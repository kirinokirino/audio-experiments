use glam::{Mat3, Vec3};
use strum_macros::{AsRefStr, EnumString, VariantNames};

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub mod pool;
use pool::handle::Handle;
use pool::Pool;
use pool::Ticket;

pub mod source;
use source::{SoundSource, Status};
pub mod buffer;
pub mod bus;
use bus::AudioBusGraph;

pub mod effects;

pub const SAMPLE_RATE: u32 = 44100;
pub const SAMPLES_PER_CHANNEL: usize = 513 * 4;

/// Sound engine manages contexts, feeds output device with data. Sound engine instance can be cloned,
/// however this is always a "shallow" clone, because actual sound engine data is wrapped in Arc.
#[derive(Clone)]
pub struct SoundEngine(Arc<Mutex<SoundEngineState>>);

/// Internal state of the sound engine.
pub struct SoundEngineState {
    contexts: Vec<SoundContext>,
    output_device: Option<tinyaudio::OutputDevice>,
}

impl SoundEngine {
    /// Creates new instance of the sound engine. It is possible to have multiple engines running at
    /// the same time, but you shouldn't do this because you can create multiple contexts which
    /// should cover 99% of use cases.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let engine = Self::without_device();
        engine.initialize_audio_output_device()?;
        Ok(engine)
    }

    /// Creates new instance of a sound engine without OS audio output device (so called headless mode).
    /// The user should periodically run [`State::render`] if they want to implement their own sample sending
    /// method to an output device (or a file, etc.).
    pub fn without_device() -> Self {
        Self(Arc::new(Mutex::new(SoundEngineState {
            contexts: Default::default(),
            output_device: None,
        })))
    }

    /// Tries to initialize default audio output device.
    pub fn initialize_audio_output_device(&self) -> Result<(), Box<dyn Error>> {
        let state = self.clone();

        let device = tinyaudio::run_output_device(
            tinyaudio::OutputDeviceParameters {
                sample_rate: SAMPLE_RATE as usize,
                channels_count: 2,
                channel_sample_count: SAMPLES_PER_CHANNEL,
            },
            {
                move |buf| {
                    // SAFETY: This is safe as long as channels count above is 2.
                    let data = unsafe {
                        std::slice::from_raw_parts_mut(
                            buf.as_mut_ptr() as *mut (f32, f32),
                            buf.len() / 2,
                        )
                    };

                    state.state().render(data);
                }
            },
        )?;

        self.state().output_device = Some(device);

        Ok(())
    }

    /// Destroys current audio output device (if any).
    pub fn destroy_audio_output_device(&self) {
        self.state().output_device = None;
    }

    /// Provides direct access to actual engine data.
    pub fn state(&self) -> MutexGuard<SoundEngineState> {
        self.0.lock().unwrap()
    }
}

impl SoundEngineState {
    /// Adds new context to the engine. Each context must be added to the engine to emit
    /// sounds.
    pub fn add_context(&mut self, context: SoundContext) {
        self.contexts.push(context);
    }

    /// Removes a context from the engine. Removed context will no longer produce any sound.
    pub fn remove_context(&mut self, context: SoundContext) {
        if let Some(position) = self.contexts.iter().position(|c| c == &context) {
            self.contexts.remove(position);
        }
    }

    /// Removes all contexts from the engine.
    pub fn remove_all_contexts(&mut self) {
        self.contexts.clear()
    }

    /// Checks if a context is registered in the engine.
    pub fn has_context(&self, context: &SoundContext) -> bool {
        self.contexts
            .iter()
            .any(|c| Arc::ptr_eq(c.state.as_ref().unwrap(), context.state.as_ref().unwrap()))
    }

    /// Returns a reference to context container.
    pub fn contexts(&self) -> &[SoundContext] {
        &self.contexts
    }

    /// Returns the length of buf to be passed to [`Self::render()`].
    pub fn render_buffer_len() -> usize {
        SAMPLES_PER_CHANNEL
    }

    /// Renders the sound into buf. The buf must have at least [`Self::render_buffer_len()`]
    /// elements. This method must be used if and only if the engine was created via
    /// [`SoundEngine::without_device`].
    ///
    /// ## Deadlocks
    ///
    /// This method internally locks added sound contexts so it must be called when all the contexts
    /// are unlocked or you'll get a deadlock.
    pub fn render(&mut self, buf: &mut [(f32, f32)]) {
        buf.fill((0.0, 0.0));
        self.render_inner(buf);
    }

    fn render_inner(&mut self, buf: &mut [(f32, f32)]) {
        for context in self.contexts.iter_mut() {
            context.state().render(buf);
        }
    }
}

/// See module docs.
#[derive(Clone, Default, Debug)]
pub struct SoundContext {
    pub(crate) state: Option<Arc<Mutex<SoundContextState>>>,
}

impl PartialEq for SoundContext {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self.state.as_ref().unwrap(), other.state.as_ref().unwrap())
    }
}

/// Internal state of context.
#[derive(Default, Debug, Clone)]
pub struct SoundContextState {
    sources: Pool<SoundSource>,
    listener: Listener,
    render_duration: Duration,
    renderer: Renderer,
    bus_graph: AudioBusGraph,
    paused: bool,
}

impl SoundContextState {
    /// Extracts a source from the context and reserves its handle. It is used to temporarily take
    /// ownership over source, and then put node back using given ticket.
    pub fn take_reserve(
        &mut self,
        handle: Handle<SoundSource>,
    ) -> (Ticket<SoundSource>, SoundSource) {
        self.sources.take_reserve(handle)
    }

    /// Puts source back by given ticket.
    pub fn put_back(
        &mut self,
        ticket: Ticket<SoundSource>,
        node: SoundSource,
    ) -> Handle<SoundSource> {
        self.sources.put_back(ticket, node)
    }

    /// Makes source handle vacant again.
    pub fn forget_ticket(&mut self, ticket: Ticket<SoundSource>) {
        self.sources.forget_ticket(ticket)
    }

    /// Pause/unpause the sound context. Paused context won't play any sounds.
    pub fn pause(&mut self, pause: bool) {
        self.paused = pause;
    }

    /// Returns true if the sound context is paused, false - otherwise.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Normalizes given frequency using context's sampling rate. Normalized frequency then can be used
    /// to create filters.
    pub fn normalize_frequency(&self, f: f32) -> f32 {
        f / SAMPLE_RATE as f32
    }

    /// Returns amount of time context spent on rendering all sound sources.
    pub fn full_render_duration(&self) -> Duration {
        self.render_duration
    }

    /// Sets new renderer.
    pub fn set_renderer(&mut self, renderer: Renderer) -> Renderer {
        std::mem::replace(&mut self.renderer, renderer)
    }

    /// Returns shared reference to current renderer.
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Returns mutable reference to current renderer.
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
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

    pub(crate) fn render(&mut self, output_device_buffer: &mut [(f32, f32)]) {
        let last_time = Instant::now();

        if !self.paused {
            self.sources.retain(|source| {
                let done = source.is_play_once() && source.status() == Status::Stopped;
                !done
            });

            self.bus_graph.begin_render(output_device_buffer.len());

            // Render sounds to respective audio buses.
            for source in self
                .sources
                .iter_mut()
                .filter(|s| s.status() == Status::Playing)
            {
                if let Some(bus_input_buffer) = self.bus_graph.try_get_bus_input_buffer(&source.bus)
                {
                    source.render(output_device_buffer.len());

                    match self.renderer {
                        Renderer::Default => {
                            // Simple rendering path. Much faster (4-5 times) than HRTF path.
                            render_source_default(source, &self.listener, bus_input_buffer);
                        }
                    }
                }
            }

            self.bus_graph.end_render(output_device_buffer);
        }

        self.render_duration = Instant::now().duration_since(last_time);
    }
}

impl SoundContext {
    /// Creates new instance of context. Internally context starts new thread which will call render all
    /// sound source and send samples to default output device. This method returns `Arc<Mutex<Context>>`
    /// because separate thread also uses context.
    pub fn new() -> Self {
        Self {
            state: Some(Arc::new(Mutex::new(SoundContextState {
                sources: Pool::new(),
                listener: Listener::new(),
                render_duration: Default::default(),
                renderer: Renderer::Default,
                bus_graph: AudioBusGraph::new(),
                paused: false,
            }))),
        }
    }

    /// Returns internal state of the context.
    ///
    /// ## Deadlocks
    ///
    /// This method internally locks a mutex, so if you'll try to do something like this:
    ///
    /// ```no_run
    /// # use fyrox_sound::context::SoundContext;
    /// # let ctx = SoundContext::new();
    /// let state = ctx.state();
    /// // Do something
    /// // ...
    /// ctx.state(); // This will cause a deadlock.
    /// ```
    ///
    /// You'll get a deadlock, so general rule here is to not store result of this method
    /// anywhere.
    pub fn state(&self) -> MutexGuard<'_, SoundContextState> {
        self.state.as_ref().unwrap().lock().unwrap()
    }

    /// Creates deep copy instead of shallow which is done by clone().
    pub fn deep_clone(&self) -> SoundContext {
        SoundContext {
            state: Some(Arc::new(Mutex::new(self.state().clone()))),
        }
    }

    /// Returns true if context is corrupted.
    pub fn is_invalid(&self) -> bool {
        self.state.is_none()
    }
}

#[derive(Debug, Clone, AsRefStr, EnumString, VariantNames)]
pub enum Renderer {
    /// Stateless default renderer.
    Default,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::Default
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
    pub(crate) fn new() -> Self {
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
        Vec3::new(
            m[3],
            m[4],
            m[5],
        )
    }

    /// Returns look axis from basis. look
    pub fn look_axis(&self) -> Vec3 {
        let m = self.basis.to_cols_array();
        Vec3::new(
            m[6],
            m[7],
            m[8],
        )
    }

    /// Returns ear axis from basis. side
    pub fn ear_axis(&self) -> Vec3 {
        let m = self.basis.to_cols_array();
        Vec3::new(
            m[0],
            m[1],
            m[2],
        )
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

pub(crate) fn render_source_default(
    source: &mut SoundSource,
    listener: &Listener,
    mix_buffer: &mut [(f32, f32)],
) {
    let panning = lerp(
        source.panning(),
        source.calculate_panning(listener),
        source.spatial_blend(),
    );
    let gain = 1.0 * source.gain();
    let left_gain = gain * (1.0 + panning);
    let right_gain = gain * (1.0 - panning);
    render_with_params(source, left_gain, right_gain, mix_buffer);
    source.last_left_gain = Some(left_gain);
    source.last_right_gain = Some(right_gain);
}

pub(crate) fn render_source_2d_only(source: &mut SoundSource, mix_buffer: &mut [(f32, f32)]) {
    let gain = (1.0 - source.spatial_blend()) * source.gain();
    let left_gain = gain * (1.0 + source.panning());
    let right_gain = gain * (1.0 - source.panning());
    render_with_params(source, left_gain, right_gain, mix_buffer);
    source.last_left_gain = Some(left_gain);
    source.last_right_gain = Some(right_gain);
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
