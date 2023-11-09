// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    controllers::{Timer, Trigger},
    entities::prelude::{
        BiQuadFilterLowPass24dbParams, DrumkitParams, GainParams, ReverbParams, SamplerParams,
    },
    generators::{EnvelopeParams, Waveform},
    modulators::DcaParams,
    prelude::*,
    stuff::{
        arpeggiator::ArpeggiatorParams,
        fm::FmSynthParams,
        lfo::LfoControllerParams,
        toys::{ToyControllerParams, ToyEffectParams, ToyInstrumentParams, ToySynthParams},
        welsh::WelshSynthParams,
    },
    utils::Paths,
};
use ensnare_egui::prelude::*;

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
pub fn register_factory_entities(mut factory: EntityFactory) -> EntityFactory {
    // Controllers
    factory.register_entity_with_str_key(Arpeggiator::ENTITY_KEY, |uid| {
        Box::new(Arpeggiator::new_with(uid, &ArpeggiatorParams::default()))
    });
    factory
        .register_entity_with_str_key(ensnare_core::controllers::ControlTrip::ENTITY_KEY, |_uid| {
            Box::<ensnare_core::controllers::ControlTrip>::default()
        });
    factory.register_entity_with_str_key(LfoController::ENTITY_KEY, |uid| {
        Box::new(LfoController::new_with(
            uid,
            &LfoControllerParams {
                frequency: FrequencyHz::from(0.2),
                waveform: Waveform::Sawtooth,
            },
        ))
    });
    factory.register_entity_with_str_key(SignalPassthroughController::ENTITY_KEY, |uid| {
        Box::new(SignalPassthroughController::new(uid))
    });
    factory.register_entity_with_str_key("signal-amplitude-passthrough", |uid| {
        Box::new(SignalPassthroughController::new_amplitude_passthrough_type(
            uid,
        ))
    });
    factory.register_entity_with_str_key("signal-amplitude-inverted-passthrough", |uid| {
        Box::new(SignalPassthroughController::new_amplitude_inverted_passthrough_type(uid))
    });
    factory.register_entity_with_str_key(Timer::ENTITY_KEY, |_uid| {
        Box::new(Timer::new_with(MusicalTime::DURATION_QUARTER))
    });
    factory.register_entity_with_str_key(Trigger::ENTITY_KEY, |_uid| {
        Box::new(Trigger::new_with(
            Timer::new_with(MusicalTime::DURATION_QUARTER),
            ControlValue(1.0),
        ))
    });

    // Effects
    factory
        .register_entity_with_str_key(Bitcrusher::ENTITY_KEY, |_uid| Box::<Bitcrusher>::default());
    factory
        .register_entity_with_str_key(Compressor::ENTITY_KEY, |_uid| Box::<Compressor>::default());
    factory.register_entity_with_str_key("filter-low-pass-24db", |uid| {
        Box::new(BiQuadFilterLowPass24db::new_with(
            uid,
            &BiQuadFilterLowPass24dbParams::default(),
        ))
    });
    factory.register_entity_with_str_key(Gain::ENTITY_KEY, |uid| {
        Box::new(Gain::new_with(
            uid,
            &GainParams {
                ceiling: Normal::from(0.5),
            },
        ))
    });
    factory.register_entity_with_str_key(Limiter::ENTITY_KEY, |_uid| Box::<Limiter>::default());
    factory.register_entity_with_str_key(Mixer::ENTITY_KEY, |_uid| Box::<Mixer>::default());
    // TODO: this is lazy. It's too hard right now to adjust parameters within
    // code, so I'm creating a special instrument with the parameters I want.
    factory.register_entity_with_str_key("mute", |uid| {
        Box::new(Gain::new_with(
            uid,
            &GainParams {
                ceiling: Normal::minimum(),
            },
        ))
    });
    factory.register_entity_with_str_key(Reverb::ENTITY_KEY, |uid| {
        Box::new(Reverb::new_with(
            uid,
            &ReverbParams {
                attenuation: Normal::from(0.8),
                seconds: 1.0,
            },
        ))
    });

    // Instruments
    factory.register_entity_with_str_key(Drumkit::ENTITY_KEY, |uid| {
        Box::new(Drumkit::new_with(
            uid,
            &DrumkitParams::default(),
            &Paths::default(),
        ))
    });
    factory.register_entity_with_str_key(FmSynth::ENTITY_KEY, |uid| {
        // A crisp, classic FM sound that brings me back to 1985.
        Box::new(FmSynth::new_with(
            uid,
            &FmSynthParams {
                depth: 1.0.into(),
                ratio: 16.0.into(),
                beta: 10.0.into(),
                carrier_envelope: EnvelopeParams::safe_default(),
                modulator_envelope: EnvelopeParams::default(),
                dca: DcaParams::default(),
            },
        ))
    });
    factory.register_entity_with_str_key(Sampler::ENTITY_KEY, |uid| {
        let mut sampler = Sampler::new_with(
            uid,
            &SamplerParams {
                filename: "stereo-pluck.wav".to_string(),
                root: 0.0.into(),
            },
        );
        let _ = sampler.load(&Paths::default()); // TODO: we're ignoring the error
        Box::new(sampler)
    });
    factory.register_entity_with_str_key(WelshSynth::ENTITY_KEY, |uid| {
        Box::new(WelshSynth::new_with(uid, &WelshSynthParams::default()))
    });

    // Toys
    factory.register_entity_with_str_key(ToyController::ENTITY_KEY, |_uid| {
        Box::<ToyController>::default()
    });
    factory.register_entity_with_str_key("toy-controller-noisy", |_uid| {
        Box::<ToyControllerAlwaysSendsMidiMessage>::default()
    });
    factory.register_entity_with_str_key(ToyInstrument::ENTITY_KEY, |_uid| {
        Box::<ToyInstrument>::default()
    });
    factory.register_entity_with_str_key(ToyEffect::ENTITY_KEY, |_uid| Box::<ToyEffect>::default());
    factory.register_entity_with_str_key(ToySynth::ENTITY_KEY, |uid| {
        Box::new(ToySynth::new_with(
            uid,
            &ToySynthParams {
                voice_count: Default::default(),
                waveform: Default::default(),
                envelope: EnvelopeParams::safe_default(),
                dca: Default::default(),
            },
        ))
    });

    factory.complete_registration();

    factory
}

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
mod tests {
    use super::*;

    // TODO: this is copied from ensnare_core::entities::factory
    fn check_entity_factory(factory: EntityFactory) {
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
                assert!(
                    e.as_controller().is_some()
                        || e.as_instrument().is_some()
                        || e.as_effect().is_some(),
                    "Entity '{}' is missing its entity type",
                    key
                );
            } else {
                panic!("new_entity({key}) failed");
            }
        }
    }

    // TODO: if we want to re-enable this, then we need to change
    // Sampler/Drumkit and anyone else to not load files when instantiated. This
    // might not be practical for those instruments.
    #[ignore = "This test requires Path hives to be set up properly, but they aren't on the CI machine."]
    #[test]
    fn creation_of_production_entities() {
        assert!(
            EntityFactory::default().entities().is_empty(),
            "A new EntityFactory should be empty"
        );

        let factory = register_factory_entities(EntityFactory::default());
        assert!(
            !factory.entities().is_empty(),
            "after registering entities, factory should contain at least one"
        );

        // After registration, rebind as immutable
        let factory = factory;

        check_entity_factory(factory);
    }

    #[test]
    fn creation_of_toy_entities() {
        assert!(
            EntityFactory::default().entities().is_empty(),
            "A new EntityFactory should be empty"
        );

        let factory = register_toy_factory_entities(EntityFactory::default());
        assert!(
            !factory.entities().is_empty(),
            "after registering toy entities, factory should contain at least one"
        );

        // After registration, rebind as immutable
        let factory = factory;

        check_entity_factory(factory);
    }
}
