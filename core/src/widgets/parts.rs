// Copyright (c) 2023 Mike Tsao. All rights reserved.

/// A range that's useful for arranging MIDI notes along an egui axis. Note that
/// this is in reverse order, because vertically-oriented piano rolls show the
/// highest notes at the top of the screen.
pub const MIDI_NOTE_F32_RANGE: std::ops::RangeInclusive<f32> =
    crate::midi::MidiNote::MAX as u8 as f32..=crate::midi::MidiNote::MIN as u8 as f32;

/// A range that covers all MIDI note values in ascending order.
pub const MIDI_NOTE_U8_RANGE: std::ops::RangeInclusive<u8> =
    crate::midi::MidiNote::MIN as u8..=crate::midi::MidiNote::MAX as u8;

#[derive(Copy, Clone, Debug, Default)]
pub enum UiSize {
    #[default]
    Small,
    Medium,
    Large,
}
impl UiSize {
    pub fn from_height(height: f32) -> UiSize {
        if height <= 32.0 {
            UiSize::Small
        } else if height <= 128.0 {
            UiSize::Medium
        } else {
            UiSize::Large
        }
    }
}
