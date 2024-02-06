// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub(crate) use controllers::ToyController;
pub(crate) use effects::ToyEffect;
pub(crate) use instruments::ToyInstrument;
pub(crate) use synth::ToySynth;

mod controllers;
mod effects;
mod instruments;
mod synth;

#[cfg(test)]
pub mod tests {
    use super::*;
    use ensnare::{
        traits::{Generates, Ticks},
        util::Rng,
    };

    // TODO: restore tests that test basic trait behavior, then figure out how
    // to run everyone implementing those traits through that behavior. For now,
    // this one just tests that a generic instrument doesn't panic when accessed
    // for non-consecutive time slices.
    #[test]
    fn sources_audio_random_access() {
        let mut instrument = instruments::ToyInstrument::default();
        let mut rng = Rng::default();

        for _ in 0..100 {
            instrument.tick(rng.rand_range(1..10) as usize);
            let _ = instrument.value();
        }
    }
}
