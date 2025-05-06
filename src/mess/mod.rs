mod conversions;
mod format;
pub mod fileio;

use core::f64;
use std::ops::Sub;

#[derive(Clone)]
struct NoteDuration {
    beats: i32,
    sixty_fourths: i32,
    rest: f32,
    full: f64,
}

impl NoteDuration {
    fn new(duration: f64) -> Self {
        let beats = duration as i32;
        let sixty_fourths: i32 = ((duration - beats as f64) * 64.0) as i32;
        let rest = duration as f32 - (beats as f32 + sixty_fourths as f32 / 64.0);
        Self {
            beats,
            sixty_fourths,
            rest,
            full: duration,
        }
    }
}

#[derive(Debug, Clone)]
struct Velocity(f32);

impl Velocity {
    fn new(value: f32) -> Self {
        Self(value)
    }
    fn is_default(&self) -> bool {
        self.0 - 0.5 < 0.01
    }
}

#[derive(Debug, Clone)]
struct Note {
    name: PitchClass,
    octave: i8,
    velocity: Velocity,
    duration: NoteDuration,
    tuning_offset: f32,
}

pub type Semitone = i32;

impl Note {
    fn new(name: PitchClass, octave: i8) -> Self {
        Self {
            name,
            octave,
            velocity: Velocity::new(0.5),
            duration: NoteDuration::new(1.0),
            tuning_offset: 0.0,
        }
    }

    fn semitone(&self) -> Semitone {
        let octave = self.octave * 12;
        let note: i8 = self.name.into();
        i32::from(note + octave)
    }
}

impl Sub for Note {
    type Output = Semitone;

    fn sub(self, other: Self) -> Self::Output {
        let self_semitone = self.semitone();
        let other_semitone = other.semitone();
        self_semitone - other_semitone
    }
}

pub fn semitone_to_frequency(semitone: Semitone) -> f32 {
    let semitone_ratio = 2.0f64.powf(1.0 / 12.0);
    let c5 = semitone_ratio.powf(3.0) * 220.0;
    let c0 = c5 * 0.5f64.powf(5.0);
    (c0 * semitone_ratio.powf(semitone as f64)) as f32
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum PitchClass {
    C = 0,
    Db = 1,
    D = 2,
    Eb = 3,
    E = 4,
    F = 5,
    Gb = 6,
    G = 7,
    Ab = 8,
    A = 9,
    Bb = 10,
    B = 11,
}
