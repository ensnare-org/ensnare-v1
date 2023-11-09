// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::widgets::{core::drag_normal, generators::envelope_shaper};
use ensnare_core::{generators::Envelope, prelude::*};

/// Wraps an [EnvelopeWidget] as a [Widget](eframe::egui::Widget).
pub fn envelope<'a>(envelope: &'a mut Envelope) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| EnvelopeWidget::new(envelope).ui(ui)
}

/// An egui widget that draws an [Envelope].
#[derive(Debug)]
struct EnvelopeWidget<'a> {
    envelope: &'a mut Envelope,
}
impl<'a> EnvelopeWidget<'a> {
    fn new(envelope: &'a mut Envelope) -> Self {
        Self { envelope }
    }
}
impl<'a> Displays for EnvelopeWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut attack = self.envelope.attack();
        let mut decay = self.envelope.decay();
        let mut sustain = self.envelope.sustain();
        let mut release = self.envelope.release();

        let canvas_response = ui.add(envelope_shaper(
            &mut attack,
            &mut decay,
            &mut sustain,
            &mut release,
        ));
        if canvas_response.changed() {
            self.envelope.set_attack(attack);
            self.envelope.set_decay(decay);
            self.envelope.set_sustain(sustain);
            self.envelope.set_release(release);
        }
        let attack_response = ui.add(drag_normal(&mut attack, "Attack: "));
        if attack_response.changed() {
            self.envelope.set_attack(attack);
        }
        ui.end_row();
        let decay_response = ui.add(drag_normal(&mut decay, "Decay: "));
        if decay_response.changed() {
            self.envelope.set_decay(decay);
        }
        ui.end_row();
        let sustain_response = ui.add(drag_normal(&mut sustain, "Sustain: "));
        if sustain_response.changed() {
            self.envelope.set_sustain(sustain);
        }
        ui.end_row();
        let release_response = ui.add(drag_normal(&mut release, "Release: "));
        if release_response.changed() {
            self.envelope.set_release(release);
        }
        ui.end_row();
        canvas_response | attack_response | decay_response | sustain_response | release_response
    }
}
