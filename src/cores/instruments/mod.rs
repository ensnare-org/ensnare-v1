// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Instruments produce digital audio, usually in response to MIDI messages. All
//! synthesizers and samplers are examples of instruments.

pub use {
    drumkit::Drumkit,
    fm::FmSynth,
    sampler::{Sampler, SamplerVoice},
    test::{TestAudioSource, TestControllerAlwaysSendsMidiMessage},
    welsh::{LfoRouting, WelshSynth, WelshVoice},
};

mod drumkit;
mod fm;
mod sampler;
mod test;
mod welsh;
