// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Built-in instruments and effects.

pub use {
    controllers::{Arpeggiator, LfoController, SignalPassthroughController, Timer, Trigger},
    effects::{
        filter::{
            BiQuadFilterAllPass, BiQuadFilterBandPass, BiQuadFilterBandStop, BiQuadFilterHighPass,
            BiQuadFilterLowPass24db,
        },
        Bitcrusher, Chorus, Compressor, Limiter,
    },
    instruments::{Drumkit, FmSynth, Sampler, SubtractiveSynth},
    //EntityFactory,
};

pub use ensnare::entities::{BuiltInEntities, Delay, Gain, Reverb};

mod controllers;
mod effects;
mod factory;
mod instruments;
