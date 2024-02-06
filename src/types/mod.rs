// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Common data types used throughout the system.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        BipolarNormal, FrequencyHz, FrequencyRange, MusicalTime, Normal, ParameterType, Ratio,
        Sample, SampleRate, SampleType, Seconds, SignalType, StereoSample, Tempo, TimeRange,
        TimeSignature, ViewRange,
    };
    pub use super::{IsUid, Uid, UidFactory};
}

pub use colors::ColorScheme;
pub use numbers::{
    FrequencyHz, FrequencyRange, ParameterType, Ratio, Sample, SampleType, SignalType, StereoSample,
};
pub use queues::{AudioQueue, VisualizationQueue};
pub use ranges::{BipolarNormal, Normal};
pub use time::{MusicalTime, SampleRate, Seconds, Tempo, TimeRange, TimeSignature, ViewRange};
pub use uid::{IsUid, Uid, UidFactory};

mod colors;
mod numbers;
mod queues;
mod ranges;
mod time;
mod uid;
