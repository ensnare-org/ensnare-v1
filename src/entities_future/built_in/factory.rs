// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::{
    controllers::{Arpeggiator, LfoController, SignalPassthroughController, Timer, Trigger},
    effects::{
        filter::BiQuadFilterLowPass24db, Bitcrusher, Chorus, Compressor, Gain, Limiter, Reverb,
    },
    instruments::{Drumkit, FmSynth, Sampler, WelshSynth},
};
use ensnare_core::{
    generators::{Envelope, Oscillator, Waveform},
    modulators::Dca,
    prelude::*,
    utils::Paths,
};
use ensnare_entity::{prelude::*, traits::EntityBounds};
use std::path::PathBuf;

pub struct BuiltInEntities {}
impl BuiltInEntities {
    /// Registers all the entities in this collection.
    pub fn register(
        mut factory: EntityFactory<dyn EntityBounds>,
    ) -> EntityFactory<dyn EntityBounds> {
        // Controllers
        factory.register_entity_with_str_key(Arpeggiator::ENTITY_KEY, |uid| {
            Box::new(Arpeggiator::new_with(uid, Tempo::default()))
        });
        factory.register_entity_with_str_key(LfoController::ENTITY_KEY, |uid| {
            Box::new(LfoController::new_with(
                uid,
                Oscillator::new_with_waveform_and_frequency(
                    Waveform::Sawtooth,
                    FrequencyHz::from(0.2),
                ),
            ))
        });
        factory.register_entity_with_str_key(SignalPassthroughController::ENTITY_KEY, |uid| {
            Box::new(SignalPassthroughController::new_with(uid))
        });
        factory.register_entity_with_str_key("signal-amplitude-passthrough", |uid| {
            Box::new(SignalPassthroughController::new_amplitude_passthrough_type(
                uid,
            ))
        });
        factory.register_entity_with_str_key("signal-amplitude-inverted-passthrough", |uid| {
            Box::new(SignalPassthroughController::new_amplitude_inverted_passthrough_type(uid))
        });
        factory.register_entity_with_str_key(Timer::ENTITY_KEY, |uid| {
            Box::new(Timer::new_with(uid, MusicalTime::DURATION_QUARTER))
        });
        factory.register_entity_with_str_key(Trigger::ENTITY_KEY, |uid| {
            Box::new(Trigger::new_with(
                uid,
                ensnare_core::controllers::Timer::new_with(MusicalTime::DURATION_QUARTER),
                ControlValue(1.0),
            ))
        });

        // Effects
        factory.register_entity_with_str_key(Bitcrusher::ENTITY_KEY, |uid| {
            Box::new(Bitcrusher::new_with(uid, 8))
        });
        factory.register_entity_with_str_key(Chorus::ENTITY_KEY, |_uid| Box::<Chorus>::default());
        factory.register_entity_with_str_key(Compressor::ENTITY_KEY, |_uid| {
            Box::<Compressor>::default()
        });
        factory.register_entity_with_str_key("filter-low-pass-24db", |uid| {
            Box::new(BiQuadFilterLowPass24db::new_with(uid, 300.0.into(), 0.85))
        });
        factory.register_entity_with_str_key(Gain::ENTITY_KEY, |uid| {
            Box::new(Gain::new_with(uid, Normal::from(0.5)))
        });
        factory.register_entity_with_str_key(Limiter::ENTITY_KEY, |_uid| Box::<Limiter>::default());
        // TODO: this is lazy. It's too hard right now to adjust parameters within
        // code, so I'm creating a special instrument with the parameters I want.
        factory.register_entity_with_str_key("mute", |uid| {
            Box::new(Gain::new_with(uid, Normal::minimum()))
        });
        factory.register_entity_with_str_key(Reverb::ENTITY_KEY, |uid| {
            Box::new(Reverb::new_with(uid, Normal::from(0.8), 1.0.into()))
        });
        factory.register_entity_with_str_key(BiQuadFilterLowPass24db::ENTITY_KEY, |uid| {
            Box::new(BiQuadFilterLowPass24db::new_with(
                uid,
                FrequencyHz(500.0),
                1.0,
            ))
        });

        // Instruments
        factory.register_entity_with_str_key(Drumkit::ENTITY_KEY, |uid| {
            Box::new(Drumkit::new_with(uid, "feed-me-seymour", &Paths::default()))
        });
        factory.register_entity_with_str_key(FmSynth::ENTITY_KEY, |uid| {
            // A crisp, classic FM sound that brings me back to 1985.
            Box::new(FmSynth::new_with(
                uid,
                Oscillator::new_with_waveform(Waveform::Sine),
                Envelope::safe_default(),
                Oscillator::new_with_waveform(Waveform::Square),
                Envelope::default(),
                1.0.into(),
                16.0.into(),
                10.0.into(),
                Dca::default(),
            ))
        });
        factory.register_entity_with_str_key(Sampler::ENTITY_KEY, |uid| {
            let mut sampler = Sampler::new_with(uid, PathBuf::from("stereo-pluck.wav"), None);
            let _ = sampler.load(&Paths::default()); // TODO: we're ignoring the error
            Box::new(sampler)
        });
        factory.register_entity_with_str_key(WelshSynth::ENTITY_KEY, |uid| {
            Box::new(WelshSynth::new_with(
                uid,
                Oscillator::new_with_waveform(Waveform::Sine),
                Oscillator::new_with_waveform(Waveform::Sawtooth),
                true,
                0.8.into(),
                Envelope::safe_default(),
                Dca::default(),
                Oscillator::new_with_waveform_and_frequency(Waveform::Sine, FrequencyHz::from(0.2)),
                ensnare_cores::LfoRouting::FilterCutoff,
                Normal::from(0.5),
                ensnare_cores::BiQuadFilterLowPass24db::new_with(FrequencyHz(250.0), 1.0),
                Normal::from(0.1),
                Normal::from(0.8),
                Envelope::safe_default(),
            ))
        });

        factory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ensnare_entity::traits::EntityBounds;

    // TODO: if we want to re-enable this, then we need to change
    // Sampler/Drumkit and anyone else to not load files when instantiated. This
    // might not be practical for those instruments.
    #[ignore = "This test requires Path hives to be set up properly, but they aren't on the CI machine."]
    #[test]
    fn creation_of_production_entities() {
        assert!(
            EntityFactory::<dyn EntityBounds>::default()
                .entities()
                .is_empty(),
            "A new EntityFactory should be empty"
        );

        #[allow(unused_mut)]
        let mut factory = EntityFactory::default();
        // TODO re-enable register_factory_entities(&mut factory);
        assert!(
            !factory.entities().is_empty(),
            "after registering entities, factory should contain at least one"
        );

        // After registration, rebind as immutable
        let factory = factory;

        check_entity_factory(factory);
    }

    // TODO: this is copied from ensnare_core::entities::factory
    pub fn check_entity_factory(factory: EntityFactory<dyn EntityBounds>) {
        assert!(factory
            .new_entity(EntityKey::from(".9-#$%)@#)"), Uid::default())
            .is_none());

        for (uid, key) in factory.keys().iter().enumerate() {
            let uid = Uid(uid + 1000);
            let e = factory.new_entity(key.clone(), uid);
            assert!(e.is_some());
            if let Some(e) = e {
                assert!(!e.name().is_empty());
                assert_eq!(
                    e.uid(),
                    uid,
                    "Entity should remember the Uid given at creation"
                );
            } else {
                panic!("new_entity({key}) failed");
            }
        }
    }
}
