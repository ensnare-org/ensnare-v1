// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Structs that hold configuration information about various parts of the
//! system. Intended to be serialized.

use crate::prelude::*;
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// Contains persistent audio settings.
#[derive(Debug, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct AudioSettings {
    sample_rate: SampleRate,
    #[derivative(Default(value = "2"))]
    channel_count: u16,

    #[serde(skip)]
    has_been_saved: bool,
}
impl HasSettings for AudioSettings {
    fn has_been_saved(&self) -> bool {
        self.has_been_saved
    }

    fn needs_save(&mut self) {
        self.has_been_saved = false;
    }

    fn mark_clean(&mut self) {
        self.has_been_saved = true;
    }
}
impl AudioSettings {
    /// Returns the currently selected audio sample rate, in Hertz (samples per
    /// second).
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Returns the currently selected number of audio channels. In most cases,
    /// this will be two (left channel and right channel).
    pub fn channel_count(&self) -> u16 {
        self.channel_count
    }
}
