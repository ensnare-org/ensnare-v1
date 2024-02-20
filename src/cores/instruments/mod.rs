// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Instruments produce digital audio, usually in response to MIDI messages. All
//! synthesizers and samplers are examples of instruments.

pub use {
    drumkit::DrumkitCore,
    fm::{FmSynthCore, FmSynthCoreBuilder},
    sampler::{SamplerCore, SamplerVoice},
    subtractive::{
        LfoRouting, SubtractiveSynthCore, SubtractiveSynthCoreBuilder, SubtractiveSynthVoice,
    },
    test::{
        TestAudioSourceCore, TestAudioSourceCoreBuilder, TestControllerAlwaysSendsMidiMessageCore,
    },
};

mod drumkit;
mod fm;
mod sampler;
mod subtractive;
mod test;
