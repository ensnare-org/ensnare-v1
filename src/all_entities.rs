// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! This module assembles all the available entities so that an application can
//! use them.

use crate::{
    cores::{
        ArpeggiatorParams, BiQuadFilterLowPass24dbParams, DrumkitParams, FmSynthParams, GainParams,
        LfoControllerParams, ReverbParams, SamplerParams, SignalPassthroughControllerParams,
        TimerParams, TriggerParams,
    },
    entities::{
        controllers::{Arpeggiator, LfoController, SignalPassthroughController},
        effects::{
            filter::BiQuadFilterLowPass24db, Bitcrusher, Chorus, Compressor, Gain, Limiter, Mixer,
            Reverb,
        },
        instruments::{Drumkit, FmSynth, Sampler, WelshSynth},
        toys::ToyInstrument,
        EntityFactory,
    },
    generators::EnvelopeParams,
    modulators::DcaParams,
    prelude::*,
};

/// A wrapper that contains all the entities we know about.
pub struct EnsnareEntities2 {}
impl EnsnareEntities2 {
    /// Registers all the entities in this collection.
    pub fn register(
        mut factory: EntityFactory<dyn EntityBounds>,
    ) -> EntityFactory<dyn EntityBounds> {
        // Controllers
        factory.register_entity_with_str_key(Arpeggiator::ENTITY_KEY, |uid| {
            Box::new(Arpeggiator::new_with(uid, &ArpeggiatorParams::default()))
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
            Box::new(SignalPassthroughController::new_with(
                uid,
                &SignalPassthroughControllerParams::default(),
            ))
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
            Box::new(Timer::new_with(
                uid,
                &TimerParams {
                    duration: MusicalTime::DURATION_QUARTER,
                },
            ))
        });
        factory.register_entity_with_str_key(Trigger::ENTITY_KEY, |uid| {
            Box::new(Trigger::new_with(
                uid,
                &TriggerParams {
                    timer: TimerParams {
                        duration: MusicalTime::DURATION_QUARTER,
                    },
                    value: ControlValue(1.0),
                },
            ))
        });

        // Effects
        factory.register_entity_with_str_key(Bitcrusher::ENTITY_KEY, |_uid| {
            Box::<Bitcrusher>::default()
        });
        factory.register_entity_with_str_key(Chorus::ENTITY_KEY, |_uid| Box::<Chorus>::default());
        factory.register_entity_with_str_key(Compressor::ENTITY_KEY, |_uid| {
            Box::<Compressor>::default()
        });
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
        factory.register_entity_with_str_key(BiQuadFilterLowPass24db::ENTITY_KEY, |uid| {
            Box::new(BiQuadFilterLowPass24db::new_with(
                uid,
                &BiQuadFilterLowPass24dbParams {
                    cutoff: FrequencyHz(500.0),
                    passband_ripple: 1.0,
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
            Box::new(WelshSynth::new_with(uid))
        });

        // Toys
        factory.register_entity_with_str_key(ToyInstrument::ENTITY_KEY, |uid| {
            Box::new(ToyInstrument::new_with(uid))
        });

        factory
    }
}
