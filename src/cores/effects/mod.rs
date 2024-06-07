// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Effects transform audio through the
//! [TransformsAudio](crate::traits::TransformsAudio) trait. Examples are
//! [Reverb] and filters.

pub use {
    bitcrusher::{BitcrusherCore, BitcrusherCoreBuilder},
    chorus::{ChorusCore, ChorusCoreBuilder},
    compressor::{CompressorCore, CompressorCoreBuilder},
    filter::{
        BiQuadFilterAllPassCore, BiQuadFilterAllPassCoreBuilder, BiQuadFilterBandPassCore,
        BiQuadFilterBandPassCoreBuilder, BiQuadFilterBandStopCore, BiQuadFilterBandStopCoreBuilder,
        BiQuadFilterHighPassCore, BiQuadFilterHighPassCoreBuilder,
        BiQuadFilterHighShelfCoreBuilder, BiQuadFilterLowPass24dbCore,
        BiQuadFilterLowPass24dbCoreBuilder, BiQuadFilterLowShelfCore,
        BiQuadFilterLowShelfCoreBuilder, BiQuadFilterNoneCoreBuilder, BiQuadFilterPeakingEqCore,
        BiQuadFilterPeakingEqCoreBuilder,
    },
    limiter::{LimiterCore, LimiterCoreBuilder},
    test::*,
};

mod bitcrusher;
mod chorus;
mod compressor;
mod filter;
mod limiter;
mod test;
