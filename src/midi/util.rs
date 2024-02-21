// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::{u7, MidiMessage};

pub struct MidiUtils {}
impl MidiUtils {
    /// Convenience function to make a note-on [MidiMessage].
    pub fn new_note_on(note: u8, vel: u8) -> MidiMessage {
        MidiMessage::NoteOn {
            key: u7::from(note),
            vel: u7::from(vel),
        }
    }

    /// Convenience function to make a note-off [MidiMessage].
    pub fn new_note_off(note: u8, vel: u8) -> MidiMessage {
        MidiMessage::NoteOff {
            key: u7::from(note),
            vel: u7::from(vel),
        }
    }
}
