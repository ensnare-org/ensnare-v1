// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::midi::MidiNote;
use core::ops::RangeInclusive;

#[derive(Clone)]
pub struct MidiNoteRange(pub RangeInclusive<MidiNote>);
impl Default for MidiNoteRange {
    fn default() -> Self {
        Self(MidiNote::MIN..=MidiNote::MAX)
    }
}
impl MidiNoteRange {
    pub fn start(&self) -> MidiNote {
        *self.0.start()
    }

    pub fn end(&self) -> MidiNote {
        *self.0.end()
    }
}
