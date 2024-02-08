// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Effects transform audio through the
//! [TransformsAudio](crate::traits::TransformsAudio) trait. Examples are
//! [Reverb] and filters.

pub use {
    bitcrusher::{BitcrusherCore, BitcrusherCoreBuilder},
    chorus::{ChorusCore, ChorusCoreBuilder},
    compressor::{CompressorCore, CompressorCoreBuilder},
    delay::{DelayCore, DelayCoreBuilder},
    filter::{
        BiQuadFilterAllPassCore, BiQuadFilterAllPassCoreBuilder, BiQuadFilterBandPassCore,
        BiQuadFilterBandPassCoreBuilder, BiQuadFilterBandStopCore, BiQuadFilterBandStopCoreBuilder,
        BiQuadFilterHighPassCore, BiQuadFilterHighPassCoreBuilder,
        BiQuadFilterHighShelfCoreBuilder, BiQuadFilterLowPass24dbCore,
        BiQuadFilterLowPass24dbCoreBuilder, BiQuadFilterLowShelfCore,
        BiQuadFilterLowShelfCoreBuilder, BiQuadFilterNoneCoreBuilder, BiQuadFilterPeakingEqCore,
        BiQuadFilterPeakingEqCoreBuilder,
    },
    gain::{GainCore, GainCoreBuilder},
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
