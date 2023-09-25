// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Built-in musical instruments and supporting components.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::entities::{
        controllers::{
            arpeggiator::{Arpeggiator, ArpeggiatorParams},
            lfo::{LfoController, LfoControllerParams},
            SignalPassthroughController, SignalPassthroughControllerParams, SignalPassthroughType,
        },
        effects::{
            bitcrusher::Bitcrusher,
            chorus::{Chorus, ChorusParams},
            compressor::Compressor,
            delay::{Delay, DelayParams, RecirculatingDelayLine},
            filter::{
                BiQuadFilterAllPass, BiQuadFilterBandPass, BiQuadFilterLowPass24db,
                BiQuadFilterLowPass24dbParams,
            },
            gain::{Gain, GainParams},
            limiter::Limiter,
            mixer::Mixer,
            reverb::{Reverb, ReverbParams},
        },
        factory::{EntityFactory, EntityKey, EntityStore},
        instruments::{
            drumkit::{Drumkit, DrumkitParams},
            fm::{FmSynth, FmSynthParams},
            sampler::{Sampler, SamplerParams},
            welsh::{WelshSynth, WelshSynthParams},
        },
        toys::{
            ToyController, ToyControllerAlwaysSendsMidiMessage, ToyEffect, ToyInstrument, ToySynth,
            ToySynthParams,
        },
    };
}

pub mod factory;

/// Controllers implement the [IsController](ensnare_core::traits::IsController)
/// trait, which means that they control other devices. An example of a
/// controller is a [Sequencer](ensnare_entities::controllers::Sequencer), which
/// produces MIDI messages.
///
/// Generally, controllers produce only control signals, and not audio. But
/// adapters exist that change one kind of signal into another, such as
/// [SignalPassthroughController], which is used in
/// [sidechaining](https://en.wikipedia.org/wiki/Dynamic_range_compression#Side-chaining).
/// In theory, a similar adapter could be used to change a control signal into
/// an audio signal.
pub mod controllers;

/// Effects implement the [IsEffect](ensnare_core::traits::IsEffect) trait, which
/// means that they transform audio. They don't produce their own audio, and
/// while they don't produce control signals, most of them do respond to
/// controls. Examples of effects are [Compressor](crate::effects::Compressor),
/// [BiQuadFilter](crate::effects::filter::BiQuadFilter), and
/// [Reverb](crate::effects::Reverb).
pub mod effects;

/// Instruments play sounds. They implement the
/// [IsInstrument](ensnare_core::traits::IsInstrument) trait, which means that
/// they respond to MIDI and produce [StereoSamples](ensnare_core::StereoSample).
/// Examples of instruments are [Sampler](crate::instruments::Sampler) and
/// [WelshSynth](crate::instruments::WelshSynth).
pub mod instruments;

/// Very simple entities that are good for development and testing.
pub mod toys;
