// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use derive_more::Display as DeriveDisplay;
use serde::{Deserialize, Serialize};

/// Newtype for MIDI channel.
#[derive(
    Clone, Copy, Debug, Default, DeriveDisplay, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub struct MidiChannel(pub u8);
#[allow(missing_docs)]
impl MidiChannel {
    pub const MIN_VALUE: u8 = 0;
    pub const MAX_VALUE: u8 = 15; // inclusive
    pub const DRUM_VALUE: u8 = 10;
    pub const DRUM: Self = Self(Self::DRUM_VALUE);

    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}
impl From<u4> for MidiChannel {
    fn from(value: u4) -> Self {
        Self(value.as_int())
    }
}
impl From<u8> for MidiChannel {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MidiChannel> for u8 {
    fn from(value: MidiChannel) -> Self {
        value.0
    }
}

/// Represents a timed [MidiMessage].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MidiEvent {
    #[allow(missing_docs)]
    pub message: MidiMessage,
    #[allow(missing_docs)]
    pub time: MusicalTime,
}
