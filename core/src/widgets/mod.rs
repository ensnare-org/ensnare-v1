// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::types::Normal;
use eframe::{
    egui::{self},
    epaint::{pos2, Color32, Rect, Rounding, Stroke},
};

pub mod prelude {
    pub use super::{
        audio, control, controllers, core, generators, pattern, placeholder, timeline, track,
    };
}

/// Contains widgets that help visualize audio.
pub mod audio;

/// Contains widgets related to automation/control.
pub mod control;

/// Contains widgets that support Controller views.
pub mod controllers;

/// Various widgets used throughout the system.
pub mod core;

/// Widgets that help render generators ([Envelope], [Oscillator], etc.)
pub mod generators;

/// Widgets that help render modulators, such as [Dca].
pub mod modulators;

/// Contains widgets related to [Pattern](crate::mini::piano_roll::Pattern)s and
/// [PianoRoll](crate::mini::piano_roll::PianoRoll).
pub mod pattern;

/// Contains widgets that are useful as placeholders during development.
pub mod placeholder;

/// Contains widgets that help draw timeline views.
pub mod timeline;

/// Contains widgets that help draw tracks.
pub mod track;

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

pub fn indicator(value: Normal) -> impl egui::Widget + 'static {
    move |ui: &mut egui::Ui| indicator_ui(ui, value)
}

fn indicator_ui(ui: &mut egui::Ui, value: Normal) -> egui::Response {
    let desired_size = egui::vec2(2.0, 16.0);
    let (rect, response) =
        ui.allocate_exact_size(desired_size, egui::Sense::focusable_noninteractive());

    if ui.is_rect_visible(rect) {
        ui.painter().rect(
            rect,
            Rounding::default(),
            Color32::BLACK,
            Stroke {
                width: 1.0,
                color: Color32::DARK_GRAY,
            },
        );
        let sound_rect = Rect::from_two_pos(
            rect.left_bottom(),
            pos2(
                rect.right(),
                rect.bottom() - rect.height() * value.value_as_f32(),
            ),
        );
        ui.painter().rect(
            sound_rect,
            Rounding::default(),
            Color32::YELLOW,
            Stroke {
                width: 1.0,
                color: Color32::YELLOW,
            },
        );
    }

    response
}
