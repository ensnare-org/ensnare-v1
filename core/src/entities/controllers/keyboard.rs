// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{midi::prelude::*, prelude::*, traits::prelude::*};
use crossbeam_channel::Sender;
use eframe::egui::{Event, Key};
use ensnare_proc_macros::{IsController, Uid};
use serde::{Deserialize, Serialize};

#[derive(Debug, IsController, Uid, Serialize, Deserialize)]
pub struct KeyboardController {
    uid: Uid,

    #[serde(skip)]
    octave: u8,

    #[serde(skip)]
    keyboard_events: ChannelPair<Event>,
}
impl Default for KeyboardController {
    fn default() -> Self {
        Self {
            uid: Default::default(),
            octave: 5,
            keyboard_events: Default::default(),
        }
    }
}
impl Displays for KeyboardController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl HandlesMidi for KeyboardController {}
#[allow(unused_variables)]
impl Controls for KeyboardController {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {}

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        while let Ok(event) = self.keyboard_events.receiver.try_recv() {
            match event {
                Event::Key {
                    key,
                    pressed,
                    repeat,
                    modifiers,
                } => {
                    if let Some((channel, message)) = self.handle_key(&key, pressed) {
                        control_events_fn(self.uid, EntityEvent::Midi(channel, message));
                    }
                }
                _ => {}
            }
        }
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {}

    fn stop(&mut self) {}

    fn skip_to_start(&mut self) {}

    fn is_performing(&self) -> bool {
        false
    }
}
impl Configurable for KeyboardController {}
impl Serializable for KeyboardController {}
impl KeyboardController {
    fn handle_key(&mut self, key: &Key, pressed: bool) -> Option<(MidiChannel, MidiMessage)> {
        match key {
            Key::A => Some(self.midi_note_message(0, pressed)),
            Key::W => Some(self.midi_note_message(1, pressed)),
            Key::S => Some(self.midi_note_message(2, pressed)),
            Key::E => Some(self.midi_note_message(3, pressed)),
            Key::D => Some(self.midi_note_message(4, pressed)),
            Key::F => Some(self.midi_note_message(5, pressed)),
            Key::T => Some(self.midi_note_message(6, pressed)),
            Key::G => Some(self.midi_note_message(7, pressed)),
            Key::Y => Some(self.midi_note_message(8, pressed)),
            Key::H => Some(self.midi_note_message(9, pressed)),
            Key::U => Some(self.midi_note_message(10, pressed)),
            Key::J => Some(self.midi_note_message(11, pressed)),
            Key::K => Some(self.midi_note_message(12, pressed)),
            Key::O => Some(self.midi_note_message(13, pressed)),
            Key::ArrowLeft => {
                if pressed {
                    self.decrease_octave();
                }
                None
            }
            Key::ArrowRight => {
                if pressed {
                    self.increase_octave();
                }
                None
            }
            _ => None,
        }
    }

    fn midi_note_message(&self, midi_note_number: u8, pressed: bool) -> (MidiChannel, MidiMessage) {
        let midi_note_number = (midi_note_number + self.octave * 12).min(127);

        if pressed {
            (
                MidiChannel(0),
                MidiMessage::NoteOn {
                    key: u7::from(midi_note_number),
                    vel: u7::from(127),
                },
            )
        } else {
            (
                MidiChannel(0),
                MidiMessage::NoteOff {
                    key: u7::from(midi_note_number),
                    vel: u7::from(0),
                },
            )
        }
    }

    pub fn sender(&self) -> &Sender<Event> {
        &self.keyboard_events.sender
    }

    fn decrease_octave(&mut self) {
        if self.octave > 0 {
            self.octave -= 1;
        }
    }
    fn increase_octave(&mut self) {
        if self.octave < 10 {
            self.octave += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_messages_for_keystrokes() {
        let mut k = KeyboardController::default();
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C4 as u8),
                vel: u7::from(127)
            }
        );
    }

    #[test]
    fn octaves() {
        let mut k = KeyboardController::default();

        // Play a note at initial octave 4.
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C4 as u8),
                vel: u7::from(127)
            }
        );

        // Increase octave and try again.
        let _ = k.handle_key(&Key::ArrowRight, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C5 as u8),
                vel: u7::from(127)
            }
        );

        // Up to maximum octave 10 (AKA octave 9).
        let _ = k.handle_key(&Key::ArrowRight, true);
        let _ = k.handle_key(&Key::ArrowRight, true);
        let _ = k.handle_key(&Key::ArrowRight, true);
        let _ = k.handle_key(&Key::ArrowRight, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C9 as u8),
                vel: u7::from(127)
            }
        );

        let _ = k.handle_key(&Key::ArrowRight, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C9 as u8),
                vel: u7::from(127)
            },
            "Trying to go higher than max octave shouldn't change anything."
        );

        // Now start over and try again with lower octaves.
        let mut k = KeyboardController::default();
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C3 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C2 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C1 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C0 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::CSub0 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        let (_, _) = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::CSub0 as u8),
                vel: u7::from(127)
            },
            "Trying to go below the lowest octave should stay at lowest octave."
        );
    }
}
