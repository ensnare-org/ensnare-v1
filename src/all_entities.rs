// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! This module assembles all the available entities so that an application can
//! use them.

use crate::{
    entities::{
        Arpeggiator, BiQuadFilterAllPass, BiQuadFilterBandPass, BiQuadFilterBandStop,
        BiQuadFilterHighPass, BiQuadFilterLowPass24db, Bitcrusher, Chorus, Compressor, Drumkit,
        FmSynth, Gain, LfoController, Limiter, Reverb, Sampler, SignalPassthroughController, Timer,
        Trigger, WelshSynth,
    },
    prelude::*,
};
use std::path::PathBuf;

/// A wrapper that contains all the entities we know about.
pub struct EnsnareEntities {}
impl EnsnareEntities {
    /// Registers all the entities in this collection.
    pub fn register(
        mut factory: EntityFactory<dyn EntityBounds>,
    ) -> EntityFactory<dyn EntityBounds> {
        // Effects
        factory.register_entity_with_str_key(Bitcrusher::ENTITY_KEY, |uid| {
            Box::new(Bitcrusher::new_with(uid, 8))
        });
        factory.register_entity_with_str_key(Chorus::ENTITY_KEY, |_uid| Box::<Chorus>::default());
        factory.register_entity_with_str_key(Compressor::ENTITY_KEY, |_uid| {
            Box::<Compressor>::default()
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
        factory.register_entity_with_str_key(BiQuadFilterAllPass::ENTITY_KEY, |uid| {
            Box::new(BiQuadFilterAllPass::new_with(uid, FrequencyHz(500.0), 1.0))
        });
        factory.register_entity_with_str_key(BiQuadFilterHighPass::ENTITY_KEY, |uid| {
            Box::new(BiQuadFilterHighPass::new_with(uid, FrequencyHz(500.0), 1.0))
        });
        factory.register_entity_with_str_key(BiQuadFilterBandPass::ENTITY_KEY, |uid| {
            Box::new(BiQuadFilterBandPass::new_with(uid, FrequencyHz(500.0), 5.0))
        });
        factory.register_entity_with_str_key(BiQuadFilterBandStop::ENTITY_KEY, |uid| {
            Box::new(BiQuadFilterBandStop::new_with(uid, FrequencyHz(500.0), 5.0))
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
                Envelope::new_with(0.0001.into(), 0.0005.into(), 0.6.into(), 0.25.into()),
                Oscillator::new_with_waveform(Waveform::Sine),
                Envelope::new_with(0.0001.into(), 0.0005.into(), 0.3.into(), 0.25.into()),
                0.35.into(),
                4.5.into(),
                40.0.into(),
                Dca::default(),
            ))
        });
        factory.register_entity_with_str_key(Sampler::ENTITY_KEY, |uid| {
            let mut sampler = Sampler::new_with(uid, PathBuf::from("stereo-pluck.wav"), None);
            let _ = sampler.load(&Paths::default()); // TODO: we're ignoring the error
            Box::new(sampler)
        });
        factory.register_entity_with_str_key(WelshSynth::ENTITY_KEY, |uid| {
            Box::new(WelshSynth::new_with_factory_patch(uid))
        });

        if false {
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
                    crate::automation::Timer::new_with(MusicalTime::DURATION_QUARTER),
                    ControlValue(1.0),
                ))
            });
        }

        factory
    }
}
