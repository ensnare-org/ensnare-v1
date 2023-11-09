// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Built-in musical instruments and supporting components.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::entities::controllers::{
        sequencers::LivePatternSequencer, SignalPassthroughController, SignalPassthroughType,
    };
    pub use crate::stuff::{
        bitcrusher::Bitcrusher,
        chorus::{Chorus, ChorusParams},
        compressor::Compressor,
        delay::{Delay, DelayParams, RecirculatingDelayLine},
        drumkit::{Drumkit, DrumkitParams},
        filter::{
            BiQuadFilterAllPass, BiQuadFilterBandPass, BiQuadFilterLowPass24db,
            BiQuadFilterLowPass24dbParams,
        },
        fm::{FmSynth, FmSynthParams},
        gain::{Gain, GainParams},
        limiter::Limiter,
        mixer::Mixer,
        reverb::{Reverb, ReverbParams},
        sampler::{Sampler, SamplerParams},
        toys::{
            ToyController, ToyControllerAlwaysSendsMidiMessage, ToyEffect, ToyInstrument, ToySynth,
            ToySynthParams,
        },
        welsh::{WelshSynth, WelshSynthParams},
    };
}

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

// Instruments play sounds. They implement the
// [IsInstrument](ensnare_core::traits::IsInstrument) trait, which means that
// they respond to MIDI and produce [StereoSamples](ensnare_core::StereoSample).
// Examples of instruments are [Sampler](crate::instruments::Sampler) and
// [WelshSynth](crate::instruments::WelshSynth).
//pub mod instruments;
