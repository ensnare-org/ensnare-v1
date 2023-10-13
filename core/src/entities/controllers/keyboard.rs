// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{midi::prelude::*, prelude::*, traits::prelude::*};
use eframe::egui::{Context, Key};
use ensnare_proc_macros::{IsController, Uid};
use serde::{Deserialize, Serialize};

#[derive(Debug, IsController, Uid, Serialize, Deserialize)]
pub struct KeyboardController {
    uid: Uid,

    #[serde(skip)]
    pub ctx: Option<Context>,

    #[serde(skip)]
    octave: u8,
}
impl Default for KeyboardController {
    fn default() -> Self {
        Self {
            uid: Default::default(),
            ctx: Default::default(),
            octave: 5,
        }
    }
}
impl Displays for KeyboardController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if self.ctx.is_none() {
            self.ctx = Some(ui.ctx().clone());
        }
        ui.label("Coming soon!")
    }
}
impl HandlesMidi for KeyboardController {}
#[allow(unused_variables)]
impl Controls for KeyboardController {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {}

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if let Some(ctx) = &self.ctx {
            ctx.input(|i| {
                for event in i.events.iter() {
                    match event {
                        eframe::egui::Event::Key {
                            key,
                            pressed,
                            repeat,
                            modifiers,
                        } => {
                            if !*repeat && !modifiers.any() {
                                eprintln!("got key event {event:?}");
                                // TODO: we're missing all sorts of events, or
                                // at least not handling the ones we've gotten
                                // properly. Something wacky is going on.
                                if let Some((channel, message)) = self.handle_key(key, *pressed) {
                                    eprintln!("sending message {message:?}");
                                    control_events_fn(self.uid, EntityEvent::Midi(channel, message))
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });
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
    fn handle_key(&self, key: &Key, pressed: bool) -> Option<(MidiChannel, MidiMessage)> {
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
            _ => None,
        }
    }

    fn midi_note_message(&self, midi_note_number: u8, pressed: bool) -> (MidiChannel, MidiMessage) {
        let midi_note_number = midi_note_number + self.octave * 12;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_messages_for_keystrokes() {
        let k = KeyboardController::default();
        let (_, message) = k.handle_key(&Key::A, true).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C4 as u8),
                vel: u7::from(127)
            }
        );
    }
}
