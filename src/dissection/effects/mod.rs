/// Attenuation effect.
#[derive(Debug, Clone, PartialEq)]
pub struct Attenuate {
    gain: f32,
}

impl Default for Attenuate {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl Attenuate {
    /// Creates new attenuation effect.
    pub fn new(gain: f32) -> Self {
        Self {
            gain: gain.max(0.0),
        }
    }
}

impl EffectRenderTrait for Attenuate {
    fn render(&mut self, input: &[(f32, f32)], output: &mut [(f32, f32)]) {
        for ((input_left, input_right), (output_left, output_right)) in
            input.iter().zip(output.iter_mut())
        {
            *output_left = *input_left * self.gain;
            *output_right = *input_right * self.gain;
        }
    }
}

/// Effects is a digital signal processing (DSP) unit that transforms input signal in a specific way.
/// For example, [`LowPassFilterEffect`] could be used to muffle audio sources; to create "underwater"
/// effect.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// See [`Attenuate`] docs for more info.
    Attenuate(Attenuate),
}

impl Default for Effect {
    fn default() -> Self {
        Effect::Attenuate(Default::default())
    }
}

pub(crate) trait EffectRenderTrait {
    fn render(&mut self, input: &[(f32, f32)], output: &mut [(f32, f32)]);
}

macro_rules! static_dispatch {
    ($self:ident, $func:ident, $($args:expr),*) => {
        match $self {
            Effect::Attenuate(v) => v.$func($($args),*),
        }
    };
}

impl EffectRenderTrait for Effect {
    fn render(&mut self, input: &[(f32, f32)], output: &mut [(f32, f32)]) {
        static_dispatch!(self, render, input, output)
    }
}
