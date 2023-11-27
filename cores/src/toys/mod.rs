// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use controllers::{ToyController, ToyControllerAlwaysSendsMidiMessage, ToyControllerParams};
pub use effects::{ToyEffect, ToyEffectParams};
pub use instruments::{ToyInstrument, ToyInstrumentParams};
pub use synth::{ToySynth, ToySynthParams};

mod controllers;
mod effects;
mod instruments;
mod synth;

#[cfg(test)]
pub mod tests {
    use super::*;
    use ensnare_core::{
        rng::Rng,
        traits::{Generates, Ticks},
    };

    // TODO: restore tests that test basic trait behavior, then figure out how
    // to run everyone implementing those traits through that behavior. For now,
    // this one just tests that a generic instrument doesn't panic when accessed
    // for non-consecutive time slices.
    #[test]
    fn sources_audio_random_access() {
        let mut instrument = ToyInstrument::default();
        let mut rng = Rng::default();

        for _ in 0..100 {
            instrument.tick(rng.rand_range(1..10) as usize);
            let _ = instrument.value();
        }
    }
}
