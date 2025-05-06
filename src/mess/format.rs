
use std::fmt::{Debug, Display};

use super::{NoteDuration, Velocity, Note, PitchClass};

impl Debug for NoteDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}s", self.full)
    }
}

impl Display for NoteDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.beats, self.sixty_fourths)
    }
}

impl Display for Velocity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}", self.0 * 127.0)
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let velocity = if !self.velocity.is_default() {
            format!(" {}", self.velocity)
        } else {
            String::new()
        };
        write!(f, "{}{}{velocity}", self.name, self.octave)
    }
}

impl Display for PitchClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::C => write!(f, "C"),
            Self::Db => write!(f, "C#"),
            Self::D => write!(f, "D"),
            Self::Eb => write!(f, "D#"),
            Self::E => write!(f, "E"),
            Self::F => write!(f, "F"),
            Self::Gb => write!(f, "F#"),
            Self::G => write!(f, "G"),
            Self::Ab => write!(f, "G#"),
            Self::A => write!(f, "A"),
            Self::Bb => write!(f, "A#"),
            Self::B => write!(f, "B"),
        }
    }
}

impl Debug for PitchClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::C => write!(f, "C"),
            Self::Db => write!(f, "C#"),
            Self::D => write!(f, "D"),
            Self::Eb => write!(f, "D#"),
            Self::E => write!(f, "E"),
            Self::F => write!(f, "F"),
            Self::Gb => write!(f, "F#"),
            Self::G => write!(f, "G"),
            Self::Ab => write!(f, "G#"),
            Self::A => write!(f, "A"),
            Self::Bb => write!(f, "A#"),
            Self::B => write!(f, "B"),
        }
    }
}
