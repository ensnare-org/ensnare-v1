// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Effects transform audio through the
//! [TransformsAudio](crate::traits::TransformsAudio) trait. Examples are
//! [Reverb] and filters.

pub use {
    bitcrusher::{BitcrusherCore, BitcrusherCoreBuilder},
    chorus::ChorusCore,
    compressor::CompressorCore,
    delay::{DelayCore, RecirculatingDelayLine},
    filter::{
        BiQuadFilterAllPassCore, BiQuadFilterBandPassCore, BiQuadFilterBandStopCore,
        BiQuadFilterHighPassCore, BiQuadFilterLowPass24dbCore,
    },
    gain::GainCore,
    limiter::{LimiterCore, LimiterCoreBuilder},
    reverb::{ReverbCore, ReverbCoreBuilder},
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
