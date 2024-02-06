// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Common data types used throughout the system.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        BipolarNormal, FrequencyHz, FrequencyRange, Normal, ParameterType, Ratio, Sample,
        SampleType, SignalType, StereoSample,
    };
    pub use super::{IsUid, Uid, UidFactory};
}

pub use colors::ColorScheme;
pub use numbers::{
    FrequencyHz, FrequencyRange, ParameterType, Ratio, Sample, SampleType, SignalType, StereoSample,
};
pub use queues::{AudioQueue, VisualizationQueue};
pub use ranges::{BipolarNormal, Normal};
pub use uid::{IsUid, Uid, UidFactory};

mod colors;
mod numbers;
mod queues;
mod ranges;
mod uid;
