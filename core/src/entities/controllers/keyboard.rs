// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{midi::prelude::*, prelude::*, traits::prelude::*};
use eframe::egui::Context;
use ensnare_proc_macros::{IsController, Uid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, IsController, Uid, Serialize, Deserialize)]
pub struct KeyboardController {
    uid: Uid,

    #[serde(skip)]
    pub ctx: Option<Context>,
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
                            eprintln!("got key event {event:?}");
                            // TODO: we're missing all sorts of events, or at
                            // least not handling the ones we've gotten
                            // properly.
                            control_events_fn(
                                self.uid,
                                EntityEvent::Midi(
                                    MidiChannel(0),
                                    if *pressed {
                                        new_note_on(69, 127)
                                    } else {
                                        new_note_off(69, 127)
                                    },
                                ),
                            )
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
