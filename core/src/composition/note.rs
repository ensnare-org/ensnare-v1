// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Add;

/// A [Note] is a single played note. It knows which key it's playing (which
/// is more or less assumed to be a MIDI key value), and when (start/end) it's
/// supposed to play, relative to time zero.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Note {
    /// The MIDI key code for the note. 69 is (usually) A4.
    pub key: u8,
    /// The range of time when this note should play.
    pub range: TimeRange,
}
impl Note {
    /// Creates a [Note] from a u8 and a start/end (inclusive start, exclusive end).
    pub const fn new_with_start_and_end(key: u8, start: MusicalTime, end: MusicalTime) -> Self {
        Self {
            key,
            range: TimeRange(start..end),
        }
    }

    /// Creates a [Note] from a u8 and start/duration.
    pub const fn new_with(key: u8, start: MusicalTime, duration: MusicalTime) -> Self {
        let end = MusicalTime::new_with_units(start.total_units() + duration.total_units());
        Self::new_with_start_and_end(key, start, end)
    }

    /// Creates a [Note] from a [MidiNote].
    pub fn new_with_midi_note(key: MidiNote, start: MusicalTime, duration: MusicalTime) -> Self {
        Self::new_with(key as u8, start, duration)
    }
}
impl Add<MusicalTime> for Note {
    type Output = Self;

    fn add(self, rhs: MusicalTime) -> Self::Output {
        Self::new_with_start_and_end(self.key, self.range.0.start + rhs, self.range.0.end + rhs)
    }
}
// TODO: I don't think this is the best choice to expose this idea. If there's a
// way to do it as an iterator, so that we don't always have to create a Vec,
// that would probably be better.
impl Into<Vec<MidiEvent>> for Note {
    fn into(self) -> Vec<MidiEvent> {
        vec![
            MidiEvent {
                message: MidiMessage::NoteOn {
                    key: u7::from(self.key),
                    vel: u7::from(127),
                },
                time: self.range.0.start,
            },
            MidiEvent {
                message: MidiMessage::NoteOff {
                    key: u7::from(self.key),
                    vel: u7::from(127),
                },
                time: self.range.0.end,
            },
        ]
    }
}
