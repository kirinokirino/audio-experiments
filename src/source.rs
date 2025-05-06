use glam::Vec3;

use std::{fmt::Debug, time::Duration};

use crate::{buffer::Buffer, SAMPLE_RATE};

/// Status (state) of sound source.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(u32)]
pub enum Status {
    /// Sound is stopped - it won't produces any sample and won't load mixer. This is default
    /// state of all sound sources.
    Stopped = 0,
    Playing = 1,

    /// Sound is paused, it can stay in this state any amount if time. Playback can be continued by
    /// setting `Playing` status.
    Paused = 2,
}

/// See module info.
#[derive(Clone)]
pub struct SoundSource {
    pub name: String,
    pub buffer: Option<Buffer>,
    // Read position in the buffer in samples. Differs from `playback_pos` if buffer is streaming.
    // In case of streaming buffer its maximum value will be some fixed value which is
    // implementation defined. It can be less than zero, this happens when we are in the process
    // of reading next block in streaming buffer (see also prev_buffer_sample).
    buf_read_pos: f64,
    // Real playback position in samples.
    playback_pos: f64,
    pitch: f64,
    pub gain: f32,
    pub looping: bool,
    pub spatial_blend: f32,
    // Important coefficient for runtime resampling. It is used to modify playback speed
    // of a source in order to match output device sampling rate. PCM data can be stored
    // in various sampling rates (22050 Hz, 44100 Hz, 88200 Hz, etc.) but output device
    // is running at fixed sampling rate (usually 44100 Hz). For example if we we'll feed
    // data to device with rate of 22050 Hz but device is running at 44100 Hz then we'll
    // hear that sound will have high pitch (2.0), to fix that we'll just pre-multiply
    // playback speed by 0.5.
    // However such auto-resampling has poor quality, but it is fast.
    resampling_multiplier: f64,
    pub status: Status,
    pub(crate) bus: String,
    pub play_once: bool,
    // Here we use Option because when source is just created it has no info about it
    // previous left and right channel gains. We can't set it to 1.0 for example
    // because it would give incorrect results: a sound would just start as loud as it
    // can be with no respect to real distance attenuation (or what else affects channel
    // gain). So if these are None engine will set correct values first and only then it
    // will start interpolation of gain.
    pub(crate) last_left_gain: Option<f32>,
    pub(crate) last_right_gain: Option<f32>,
    pub(crate) frame_samples: Vec<(f32, f32)>,
    // This sample is used when doing linear interpolation between two blocks of streaming buffer.
    prev_buffer_sample: (f32, f32),
    radius: f32,
    position: Vec3,
    max_distance: f32,
    rolloff_factor: f32,
}

impl Debug for SoundSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundSource")
            .field("name", &self.name)
            .field("buffer", &self.buffer)
            .field("buf_read_pos", &self.buf_read_pos)
            .field("playback_pos", &self.playback_pos)
            .field("pitch", &self.pitch)
            .field("gain", &self.gain)
            .field("looping", &self.looping)
            .field("spatial_blend", &self.spatial_blend)
            .field("resampling_multiplier", &self.resampling_multiplier)
            .field("status", &self.status)
            .field("bus", &self.bus)
            .field("play_once", &self.play_once)
            .field("last_left_gain", &self.last_left_gain)
            .field("last_right_gain", &self.last_right_gain)
            .field("frame_samples", &format!("[..{} frame_samples]", &self.frame_samples.len()))
            .field("prev_buffer_sample", &self.prev_buffer_sample)
            .field("radius", &self.radius)
            .field("position", &self.position)
            .field("max_distance", &self.max_distance)
            .field("rolloff_factor", &self.rolloff_factor)
            .finish()
    }
}

impl SoundSource {
    /// Sets new gain (volume) of sound. Value should be in 0..1 range, but it is not clamped
    /// and larger values can be used to "overdrive" sound.
    ///
    /// # Notes
    ///
    /// Physical volume has non-linear scale (logarithmic) so perception of sound at 0.25 gain
    /// will be different if logarithmic scale was used.
    pub fn set_gain(&mut self, gain: f32) -> &mut Self {
        self.gain = gain;
        self
    }

    /// Sets sound pitch. Defines "tone" of sounds. Default value is 1.0
    pub fn set_pitch(&mut self, pitch: f64) -> &mut Self {
        self.pitch = pitch.abs();
        self
    }

    /// Stops sound source. Automatically rewinds streaming buffers.
    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.status = Status::Stopped;

        self.buf_read_pos = 0.0;
        self.playback_pos = 0.0;

        Ok(())
    }
    /// Sets position of source in world space.
    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self
    }

    /// Sets radius of imaginable sphere around source in which no distance attenuation is applied.
    pub fn set_radius(&mut self, radius: f32) -> &mut Self {
        self.radius = radius;
        self
    }

    /// Sets rolloff factor. Rolloff factor is used in distance attenuation and has different meaning
    /// in various distance models. It is applicable only for InverseDistance and ExponentDistance
    /// distance models. See DistanceModel docs for formulae.
    pub fn set_rolloff_factor(&mut self, rolloff_factor: f32) -> &mut Self {
        self.rolloff_factor = rolloff_factor;
        self
    }

    /// Sets maximum distance until which distance gain will be applicable. Basically it doing this
    /// min(max(distance, radius), max_distance) which clamps distance in radius..max_distance range.
    /// From listener's perspective this will sound like source has stopped decreasing its volume even
    /// if distance continue to grow.
    pub fn set_max_distance(&mut self, max_distance: f32) -> &mut Self {
        self.max_distance = max_distance;
        self
    }

    /// Sets new name of the target audio bus. The name must be valid, otherwise the sound won't play!
    /// Default is [`AudioBusGraph::PRIMARY_BUS`].
    pub fn set_bus<S: AsRef<str>>(&mut self, bus: S) {
        bus.as_ref().clone_into(&mut self.bus);
    }

    /// Returns playback duration.
    pub fn playback_time(&self) -> Duration {
        if let Some(buffer) = self.buffer.as_ref() {
            return Duration::from_secs_f64(self.playback_pos / (SAMPLE_RATE as f64));
        }

        Duration::from_secs(0)
    }

    /// Sets playback duration.
    pub fn set_playback_time(&mut self, time: Duration) {
        if let Some(buffer) = self.buffer.as_ref() {
            // Set absolute position first.
            self.playback_pos = (time.as_secs_f64() * SAMPLE_RATE as f64)
                .clamp(0.0, buffer.duration().as_secs_f64());
            // Then adjust buffer read position.
            self.buf_read_pos = self.playback_pos;
            assert!(
                self.buf_read_pos * (buffer.channel_count() as f64) < buffer.samples.len() as f64
            );
        }
    }

    pub(crate) fn render(&mut self, amount: usize) {
        if self.frame_samples.capacity() < amount {
            self.frame_samples = Vec::with_capacity(amount);
        }

        self.frame_samples.clear();

        if let Some(mut buffer) = self.buffer.clone() {
            if self.status == Status::Playing && !buffer.samples.is_empty() {
                self.render_playing(&mut buffer, amount);
            }
        }
        // Fill the remaining part of frame_samples.
        self.frame_samples.resize(amount, (0.0, 0.0));
    }

    fn render_playing(&mut self, buffer: &mut Buffer, amount: usize) {
        let mut count = 0;
        loop {
            count += self.render_until_block_end(buffer, amount - count);
            if count == amount {
                break;
            }

            self.buf_read_pos = 0.0;
            self.playback_pos = 0.0;
            if !self.looping {
                self.status = Status::Stopped;
                return;
            }
        }
    }

    // Renders until the end of the block or until amount samples is written and returns
    // the number of written samples.
    fn render_until_block_end(&mut self, buffer: &mut Buffer, mut amount: usize) -> usize {
        let step = self.pitch * self.resampling_multiplier;
        if step == 1.0 {
            if self.buf_read_pos < 0.0 {
                // This can theoretically happen if we change pitch on the fly.
                self.frame_samples.push(self.prev_buffer_sample);
                self.buf_read_pos = 0.0;
                amount -= 1;
            }
            // Fast-path for common case when there is no resampling and no pitch change.
            let from = self.buf_read_pos as usize;
            let buffer_len = buffer.samples.len() / usize::from(buffer.channel_count());
            let rendered = (buffer_len - from).min(amount);
            if buffer.channel_count() == 2 {
                for i in from..from + rendered {
                    self.frame_samples
                        .push((buffer.samples[i * 2], buffer.samples[i * 2 + 1]))
                }
            } else {
                for i in from..from + rendered {
                    self.frame_samples
                        .push((buffer.samples[i], buffer.samples[i]))
                }
            }
            self.buf_read_pos += rendered as f64;
            self.playback_pos += rendered as f64;
            rendered
        } else {
            self.render_until_block_end_resample(buffer, amount, step)
        }
    }

    // Does linear resampling while rendering until the end of the block.
    fn render_until_block_end_resample(
        &mut self,
        buffer: &mut Buffer,
        amount: usize,
        step: f64,
    ) -> usize {
        let mut rendered = 0;

        while self.buf_read_pos < 0.0 {
            // Interpolate between last sample of previous buffer and first sample of current
            // buffer. This is important, otherwise there will be quiet but audible pops
            // in the output.
            let w = (self.buf_read_pos - self.buf_read_pos.floor()) as f32;
            let cur_first_sample = if buffer.channel_count() == 2 {
                (buffer.samples[0], buffer.samples[1])
            } else {
                (buffer.samples[0], buffer.samples[0])
            };
            let l = self.prev_buffer_sample.0 * (1.0 - w) + cur_first_sample.0 * w;
            let r = self.prev_buffer_sample.1 * (1.0 - w) + cur_first_sample.1 * w;
            self.frame_samples.push((l, r));
            self.buf_read_pos += step;
            self.playback_pos += step;
            rendered += 1;
        }

        // We want to keep global positions in f64, but use f32 in inner loops (this improves
        // code generation and performance at least on some systems), so we split the buf_read_pos
        // into integer and f32 part.
        let buffer_base_idx = self.buf_read_pos as usize;
        let mut buffer_rel_pos = (self.buf_read_pos - buffer_base_idx as f64) as f32;
        let start_buffer_rel_pos = buffer_rel_pos;
        let rel_step = step as f32;
        // We skip one last element because the hot loop resampling between current and next
        // element. Last elements are appended after the hot loop.
        let buffer_last = buffer.samples.len() / usize::from(buffer.channel_count()) - 1;
        if buffer.channel_count() == 2 {
            while rendered < amount {
                let (idx, w) = {
                    let idx = buffer_rel_pos as usize;
                    // This looks a bit complicated but fract() is quite a bit slower on x86,
                    // because it turns into a function call on targets < SSE4.1, unlike aarch64)
                    (idx + buffer_base_idx, buffer_rel_pos - idx as f32)
                };
                if idx >= buffer_last {
                    break;
                }
                let l = buffer.samples[idx * 2] * (1.0 - w) + buffer.samples[idx * 2 + 2] * w;
                let r = buffer.samples[idx * 2 + 1] * (1.0 - w) + buffer.samples[idx * 2 + 3] * w;
                self.frame_samples.push((l, r));
                buffer_rel_pos += rel_step;
                rendered += 1;
            }
        } else {
            while rendered < amount {
                let (idx, w) = {
                    let idx = buffer_rel_pos as usize;
                    // See comment above.
                    (idx + buffer_base_idx, buffer_rel_pos - idx as f32)
                };
                if idx >= buffer_last {
                    break;
                }
                let v = buffer.samples[idx] * (1.0 - w) + buffer.samples[idx + 1] * w;
                self.frame_samples.push((v, v));
                buffer_rel_pos += rel_step;
                rendered += 1;
            }
        }

        self.buf_read_pos += (buffer_rel_pos - start_buffer_rel_pos) as f64;
        self.playback_pos += (buffer_rel_pos - start_buffer_rel_pos) as f64;
        rendered
    }

    pub(crate) fn frame_samples(&self) -> &[(f32, f32)] {
        &self.frame_samples
    }
}

impl Default for SoundSource {
    fn default() -> Self {
        Self {
            name: Default::default(),
            buffer: None,
            buf_read_pos: 0.0,
            playback_pos: 0.0,
            pitch: 1.0,
            gain: 1.0,
            spatial_blend: 1.0,
            looping: false,
            resampling_multiplier: 1.0,
            status: Status::Stopped,
            bus: "Master".to_string(),
            play_once: false,
            last_left_gain: None,
            last_right_gain: None,
            frame_samples: Default::default(),
            prev_buffer_sample: (0.0, 0.0),
            radius: 1.0,
            position: Vec3::new(0.0, 0.0, 0.0),
            max_distance: f32::MAX,
            rolloff_factor: 1.0,
        }
    }
}
