// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Common data types used throughout the system.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        BipolarNormal, FrequencyHz, FrequencyRange, IsUid, MusicalTime, Normal, ParameterType,
        Ratio, Sample, SampleRate, SampleType, Seconds, SignalType, StereoSample, Tempo, TimeRange,
        TimeSignature, Uid, UidFactory, ViewRange,
    };
}
pub use colors::ColorScheme;
pub use ensnare::types::{
    BipolarNormal, FrequencyHz, IsUid, MusicalTime, Normal, ParameterType, Ratio, Sample,
    SampleRate, Seconds, StereoSample, Tempo, TimeRange, TimeSignature, Uid, UidFactory, ViewRange,
};
pub use numbers::{FrequencyRange, SampleType, SignalType};
pub use queues::VisualizationQueue;

mod colors;
mod numbers;
mod queues;
mod time;
mod uid;
