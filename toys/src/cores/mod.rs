// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub(crate) use controllers::ToyControllerCore;
pub(crate) use effects::ToyEffectCore;
pub(crate) use instruments::ToyInstrumentCore;
pub(crate) use synth::ToySynthCore;

mod controllers;
mod effects;
mod instruments;
mod synth;

#[cfg(test)]
pub mod tests {
    use super::*;
    use ensnare::{traits::Generates, types::StereoSample, util::Rng};

    // TODO: restore tests that test basic trait behavior, then figure out how
    // to run everyone implementing those traits through that behavior.
    #[test]
    fn generates_audio() {
        let mut instrument = instruments::ToyInstrumentCore::default();
        let mut rng = Rng::default();

        let mut buffer = [StereoSample::default(); 7];
        for _ in 0..100 {
            instrument.generate(&mut buffer);
        }
    }
}
