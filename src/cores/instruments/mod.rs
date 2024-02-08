// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Instruments produce digital audio, usually in response to MIDI messages. All
//! synthesizers and samplers are examples of instruments.

pub use {
    drumkit::DrumkitCore,
    fm::{FmSynthCore, FmSynthCoreBuilder},
    sampler::{SamplerCore, SamplerVoice},
    test::{TestAudioSourceCore, TestControllerAlwaysSendsMidiMessageCore},
    welsh::{LfoRouting, WelshSynthCore, WelshVoice},
};

mod drumkit;
mod fm;
mod sampler;
mod test;
mod welsh;
