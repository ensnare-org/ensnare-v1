// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Structs that hold configuration information about various parts of the
//! system. Intended to be serialized.

use crate::prelude::*;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

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

/// Contains persistent MIDI settings.
#[derive(Debug, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
pub struct MidiSettings {
    pub(crate) selected_input: Option<MidiPortDescriptor>,
    pub(crate) selected_output: Option<MidiPortDescriptor>,

    #[serde(default)]
    #[derivative(Default(value = "true"))]
    should_route_externally: bool,

    #[serde(skip)]
    pub(crate) e: MidiSettingsEphemerals,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct MidiSettingsEphemerals {
    has_been_saved: bool,

    #[derivative(Default(value = "Self::create_last_input_instant()"))]
    last_input_instant: Arc<Mutex<Instant>>,
    #[derivative(Default(value = "Instant::now()"))]
    last_output_instant: Instant,
}
impl MidiSettingsEphemerals {
    fn create_last_input_instant() -> Arc<Mutex<Instant>> {
        Arc::new(Mutex::new(Instant::now()))
    }
}

impl HasSettings for MidiSettings {
    fn has_been_saved(&self) -> bool {
        self.e.has_been_saved
    }

    fn needs_save(&mut self) {
        self.e.has_been_saved = false;
    }

    fn mark_clean(&mut self) {
        self.e.has_been_saved = true;
    }
}
impl MidiSettings {
    /// Updates the field and marks the struct eligible to save.
    pub fn set_input(&mut self, input: Option<MidiPortDescriptor>) {
        if input != self.selected_input {
            self.selected_input = input;
            self.needs_save();
        }
    }
    /// Updates the field and marks the struct eligible to save.
    pub fn set_output(&mut self, output: Option<MidiPortDescriptor>) {
        if output != self.selected_output {
            self.selected_output = output;
            self.needs_save();
        }
    }
    pub fn should_route_externally(&self) -> bool {
        self.should_route_externally
    }
    pub fn set_should_route_externally(&mut self, should_route: bool) {
        if should_route != self.should_route_externally {
            self.should_route_externally = should_route;
            self.needs_save();
        }
    }
}
