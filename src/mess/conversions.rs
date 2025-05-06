use super::{NoteDuration, Velocity, Note, PitchClass};

impl From<u8> for Velocity {
    fn from(value: u8) -> Self {
        Self(value as f32 / 127.0)
    }
}

impl From<Velocity> for u8 {
    fn from(value: Velocity) -> Self {
        (value.0 * 127.0) as u8
    }
}

impl From<PitchClass> for u8 {
    fn from(value: PitchClass) -> Self {
        value as u8
    }
}

impl From<PitchClass> for i8 {
    fn from(value: PitchClass) -> Self {
        value as i8
    }
}

impl From<u8> for PitchClass {
    fn from(value: u8) -> Self {
        let value = value % 12;
        match value {
            0 => Self::C,
            1 => Self::Db,
            2 => Self::D,
            3 => Self::Eb,
            4 => Self::E,
            5 => Self::F,
            6 => Self::Gb,
            7 => Self::G,
            8 => Self::Ab,
            9 => Self::A,
            10 => Self::Bb,
            11 => Self::B,
            _ => unreachable!(),
        }
    }
}

impl From<i8> for PitchClass {
    fn from(mut value: i8) -> Self {
        while value < 0 {
            value += 12;
        }
        let value = value % 12;
        match value {
            0 => Self::C,
            1 => Self::Db,
            2 => Self::D,
            3 => Self::Eb,
            4 => Self::E,
            5 => Self::F,
            6 => Self::Gb,
            7 => Self::G,
            8 => Self::Ab,
            9 => Self::A,
            10 => Self::Bb,
            11 => Self::B,
            _ => unreachable!(),
        }
    }
}
