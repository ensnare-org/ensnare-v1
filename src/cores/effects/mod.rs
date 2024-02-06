// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Effects transform audio through the
//! [TransformsAudio](crate::traits::TransformsAudio) trait. Examples are
//! [Reverb] and filters.

pub use {
    bitcrusher::Bitcrusher,
    chorus::Chorus,
    compressor::Compressor,
    delay::{Delay, RecirculatingDelayLine},
    filter::{
        BiQuadFilterAllPass, BiQuadFilterBandPass, BiQuadFilterBandStop, BiQuadFilterHighPass,
        BiQuadFilterLowPass24db,
    },
    gain::Gain,
    limiter::Limiter,
    reverb::Reverb,
    test::*,
};

mod bitcrusher;
mod chorus;
mod compressor;
mod delay;
mod filter;
mod gain;
mod limiter;
mod reverb;
mod test;
