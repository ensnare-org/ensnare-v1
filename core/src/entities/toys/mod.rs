// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::midi::MidiChannel;

use super::factory::{EntityFactory, EntityKey};

pub use controllers::{ToyController, ToyControllerAlwaysSendsMidiMessage, ToyControllerParams};
pub use effects::{ToyEffect, ToyEffectParams};
pub use instruments::{ToyInstrument, ToyInstrumentParams};
pub use synth::{ToySynth, ToySynthParams};

mod controllers;
mod effects;
mod instruments;
mod synth;

/// Registers all [EntityFactory]'s entities. Note that the function returns a
/// EntityFactory, rather than operating on an &mut. This encourages
/// one-and-done creation, after which the factory is immutable:
///
/// ```ignore
/// let factory = register_factory_entities(EntityFactory::default());
/// ```
///
/// TODO: maybe a Builder pattern is better, so that people can compose
/// factories out of any entities they want, and still get the benefits of
/// immutability.
#[must_use]
pub fn register_toy_factory_entities(mut factory: EntityFactory) -> EntityFactory {
    factory.register_entity(EntityKey::from(ToySynth::ENTITY_KEY), |uid| {
        Box::new(ToySynth::new_with(uid, &ToySynthParams::default()))
    });
    factory.register_entity(EntityKey::from(ToyInstrument::ENTITY_KEY), |uid| {
        Box::new(ToyInstrument::new_with(
            uid,
            &ToyInstrumentParams::default(),
        ))
    });
    factory.register_entity(EntityKey::from(ToyController::ENTITY_KEY), |uid| {
        Box::new(ToyController::new_with(
            uid,
            &ToyControllerParams::default(),
            MidiChannel::default(),
        ))
    });
    factory.register_entity(EntityKey::from(ToyEffect::ENTITY_KEY), |uid| {
        Box::new(ToyEffect::new_with(uid, &ToyEffectParams::default()))
    });

    factory.complete_registration();

    factory
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::traits::{Generates, Ticks};

    // TODO: restore tests that test basic trait behavior, then figure out how
    // to run everyone implementing those traits through that behavior. For now,
    // this one just tests that a generic instrument doesn't panic when accessed
    // for non-consecutive time slices.
    #[test]
    fn sources_audio_random_access() {
        let mut instrument = ToyInstrument::default();
        let mut rng = oorandom::Rand32::new(0);

        for _ in 0..100 {
            instrument.tick(rng.rand_range(1..10) as usize);
            let _ = instrument.value();
        }
    }
}
