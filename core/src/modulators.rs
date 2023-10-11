// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    drag_drop::{DragDropManager, DragDropSource},
    prelude::*,
    traits::prelude::*,
    types::{BipolarNormal, Normal},
};
use eframe::egui::Slider;
use ensnare_proc_macros::{Control, Params};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum DcaAction {
    LinkControl(Uid, ControlIndex),
}

/// The Digitally Controller Amplifier (DCA) handles gain and pan for many kinds
/// of synths.
///
/// See DSSPC++, Section 7.9 for requirements. TODO: implement
#[derive(Debug, Default, Control, Params, Serialize, Deserialize)]
pub struct Dca {
    #[control]
    #[params]
    gain: Normal,
    #[control]
    #[params]
    pan: BipolarNormal,

    #[serde(skip)]
    action: Option<DcaAction>,
}
impl Dca {
    pub fn new_with(params: &DcaParams) -> Self {
        Self {
            gain: params.gain(),
            pan: params.pan(),
            action: None,
        }
    }

    pub fn transform_audio_to_stereo(&mut self, input_sample: Sample) -> StereoSample {
        // See Pirkle, DSSPC++, p.73
        let input_sample: f64 = input_sample.0 * self.gain.0;
        let left_pan: f64 = 1.0 - 0.25 * (self.pan.0 + 1.0f64).powi(2);
        let right_pan: f64 = 1.0 - (0.5 * self.pan.0 - 0.5f64).powi(2);
        StereoSample::new(
            (left_pan * input_sample).into(),
            (right_pan * input_sample).into(),
        )
    }

    pub fn gain(&self) -> Normal {
        self.gain
    }

    pub fn set_gain(&mut self, gain: Normal) {
        self.gain = gain;
    }

    pub fn pan(&self) -> BipolarNormal {
        self.pan
    }

    pub fn set_pan(&mut self, pan: BipolarNormal) {
        self.pan = pan;
    }

    pub fn update_from_params(&mut self, params: &DcaParams) {
        self.set_gain(params.gain());
        self.set_pan(params.pan());
    }
}
impl Displays for Dca {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(dca(self))
    }
}
impl Acts for Dca {
    type Action = DcaAction;

    fn set_action(&mut self, action: Self::Action) {
        debug_assert!(
            self.action.is_none(),
            "Uh-oh, tried to set to {action} but it was already set to {:?}",
            self.action
        );
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<Self::Action> {
        self.action.take()
    }
}

/// Wraps a [DcaWidget] as a [Widget](eframe::egui::Widget).
pub fn dca<'a>(dca: &'a mut Dca) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| DcaWidget::new(dca).ui(ui)
}

/// An egui widget for [Dca].
#[derive(Debug)]
struct DcaWidget<'a> {
    dca: &'a mut Dca,
}
impl<'a> Displays for DcaWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut drop_index = None;
        let response = {
            let mut value = self.dca.gain().0;
            let response = DragDropManager::drop_target(ui, true, |ui| {
                ui.add(Slider::new(&mut value, Normal::range()).text("Gain"))
            })
            .inner;
            if DragDropManager::is_dropped(ui, &response) {
                drop_index = Some(self.dca.control_index_for_name("gain").unwrap());
            }
            ui.end_row();
            if response.changed() {
                self.dca.set_gain(Normal::from(value));
            }
            response
        } | {
            let mut value = self.dca.pan().0;
            let response = DragDropManager::drop_target(ui, true, |ui| {
                ui.add(Slider::new(&mut value, BipolarNormal::range()).text("Pan (L-R)"))
            })
            .inner;
            if DragDropManager::is_dropped(ui, &response) {
                drop_index = Some(self.dca.control_index_for_name("pan").unwrap());
            }
            ui.end_row();
            if response.changed() {
                self.dca.set_pan(BipolarNormal::from(value));
            }
            response
        };
        if let Some(index) = drop_index {
            if let Some(source) = DragDropManager::source() {
                match source {
                    DragDropSource::ControlSource(source_uid) => {
                        self.dca
                            .set_action(DcaAction::LinkControl(source_uid, index));
                    }
                    _ => {}
                }
            }
        }

        response
    }
}
impl<'a> DcaWidget<'a> {
    fn new(dca: &'a mut Dca) -> Self {
        Self { dca }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dca_mainline() {
        let mut dca = Dca::new_with(&DcaParams {
            gain: 1.0.into(),
            pan: BipolarNormal::zero(),
        });
        const VALUE_IN: Sample = Sample(0.5);
        const VALUE: Sample = Sample(0.5);
        assert_eq!(
            dca.transform_audio_to_stereo(VALUE_IN),
            StereoSample::new(VALUE * 0.75, VALUE * 0.75),
            "Pan center should give 75% equally to each channel"
        );

        dca.set_pan(BipolarNormal::new(-1.0));
        assert_eq!(
            dca.transform_audio_to_stereo(VALUE_IN),
            StereoSample::new(VALUE, 0.0.into()),
            "Pan left should give 100% to left channel"
        );

        dca.set_pan(BipolarNormal::new(1.0));
        assert_eq!(
            dca.transform_audio_to_stereo(VALUE_IN),
            StereoSample::new(0.0.into(), VALUE),
            "Pan right should give 100% to right channel"
        );
    }
}
