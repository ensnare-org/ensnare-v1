// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::{
    controllers::{Arpeggiator, LfoController, SignalPassthroughController, Timer, Trigger},
    effects::{
        filter::BiQuadFilterLowPass24db, Bitcrusher, Chorus, Compressor, Gain, Limiter, Reverb,
    },
    instruments::{Drumkit, FmSynth, Sampler, WelshSynth},
};
use crate::{
    cores::{effects, instruments},
    prelude::*,
    util::Paths,
};
use std::path::PathBuf;

pub struct BuiltInEntities {}
impl BuiltInEntities {
    /// Registers all the entities in this collection.
    pub fn register(
        mut factory: EntityFactory<dyn EntityBounds>,
    ) -> EntityFactory<dyn EntityBounds> {
        // Controllers
        factory.register_entity_with_str_key(Arpeggiator::ENTITY_KEY, |uid| {
            Box::new(Arpeggiator::new_with(uid))
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
                crate::automation::Timer::new_with(MusicalTime::DURATION_QUARTER),
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
                instruments::LfoRouting::FilterCutoff,
                Normal::from(0.5),
                effects::BiQuadFilterLowPass24db::new_with(FrequencyHz(250.0), 1.0),
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
    use crate::{entities::Bitcrusher, prelude::*};
    use std::collections::HashSet;

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

    // TODO: this is copied from crate::core::entities::factory
    pub fn check_entity_factory(factory: EntityFactory<dyn EntityBounds>) {
        assert!(factory
            .new_entity(&EntityKey::from(".9-#$%)@#)"), Uid::default())
            .is_none());

        for (uid, key) in factory.keys().iter().enumerate() {
            let uid = Uid(uid + 1000);
            let e = factory.new_entity(key, uid);
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

    // This could be a test specific to the Control proc macro, but we'd like to
    // run it over all the entities we know about in case someone implements the
    // Controls trait manually.
    fn validate_controllable(entity: &mut dyn EntityBounds) {
        let mut param_names: HashSet<String> = HashSet::default();

        for index in 0..entity.control_index_count() {
            let index = ControlIndex(index);
            let param_name = entity.control_name_for_index(index).unwrap();
            assert!(
                param_names.insert(param_name.clone()),
                "Duplicate param name {} at index {index}",
                &param_name
            );
            assert_eq!(
                entity.control_index_for_name(&param_name).unwrap(),
                index,
                "Couldn't recover expected index {index} from control_index_for_name({})",
                &param_name
            );
        }
        assert_eq!(
            param_names.len(),
            entity.control_index_count(),
            "control_index_count() agrees with number of params"
        );

        // The Controls trait doesn't support getting values, only setting them.
        // So we can't actually verify that our sets are doing anything. If this
        // becomes an issue, then we have two options: (1) extend the Controls
        // trait to allow getting, and then worry that any errors are tested by
        // the same generated code that has the error, or (2) come up with a
        // wacky proc macro that converts param_name into a getter invocation. I
        // don't think regular macros can do that because of hygiene rules.
        for index in 0..entity.control_index_count() {
            let index = ControlIndex(index);
            let param_name = entity.control_name_for_index(index).unwrap();
            entity.control_set_param_by_index(index, 0.0.into());
            entity.control_set_param_by_index(index, 1.0.into());
            entity.control_set_param_by_name(&param_name, 0.0.into());
            entity.control_set_param_by_name(&param_name, 1.0.into());
        }
    }

    fn validate_configurable(entity: &mut dyn EntityBounds) {
        let sample_rate = entity.sample_rate();
        let new_sample_rate = (sample_rate.0 + 100).into();
        entity.update_sample_rate(new_sample_rate);
        assert_eq!(entity.sample_rate(), new_sample_rate);

        let tempo = entity.tempo();
        let new_tempo = (tempo.0 + 10.0).into();
        entity.update_tempo(new_tempo);
        assert_eq!(entity.tempo(), new_tempo);

        let new_time_signature = TimeSignature::CUT_TIME;
        assert_ne!(entity.time_signature(), new_time_signature);
        entity.update_time_signature(new_time_signature);
        assert_eq!(entity.time_signature(), new_time_signature);
    }

    #[test]
    fn entity_passes() {
        let factory = BuiltInEntities::register(EntityFactory::default());
        let uid_factory = EntityUidFactory::default();
        for entity_key in factory.keys() {
            let mut entity = factory
                .new_entity(entity_key, uid_factory.mint_next())
                .unwrap();
            validate_controllable(entity.as_mut());
            validate_configurable(entity.as_mut());
        }

        // TODO: move this somewhere that does testing for all entities.
    }
}
