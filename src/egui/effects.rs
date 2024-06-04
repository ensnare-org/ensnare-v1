// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    cores::effects::{
        BiQuadFilterAllPassCore, BiQuadFilterBandPassCore, BiQuadFilterBandStopCore,
        BiQuadFilterHighPassCore, BiQuadFilterLowPass24dbCore,
    },
    prelude::*,
};
use eframe::egui::{Frame, Slider, Widget};
use ensnare::prelude::*;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum BiQuadFilterWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

pub struct BiQuadFilterBandPassWidget<'a> {
    filter: &'a mut BiQuadFilterBandPassCore,
    action: &'a mut Option<BiQuadFilterWidgetAction>,
}
impl<'a> BiQuadFilterBandPassWidget<'a> {
    fn new_with(
        filter: &'a mut BiQuadFilterBandPassCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> Self {
        Self { filter, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        filter: &'a mut BiQuadFilterBandPassCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterBandPassWidget::new_with(filter, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterBandPassWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut bw = self.filter.bandwidth();
        let (cutoff_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(
                Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                    .text("Cutoff")
                    .suffix(FrequencyHz::UNITS_SUFFIX),
            )
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterBandPassCore::CUTOFF_INDEX.into(),
            ));
        }
        if cutoff_response.inner.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let (bw_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(Slider::new(&mut bw, 0.0..=10.0).text("Bandwidth"))
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterBandPassCore::BANDWIDTH_INDEX.into(),
            ));
        }
        if bw_response.inner.changed() {
            self.filter.set_bandwidth(bw);
        };
        cutoff_response.response | bw_response.response
    }
}
pub struct BiQuadFilterBandStopWidget<'a> {
    filter: &'a mut BiQuadFilterBandStopCore,
    action: &'a mut Option<BiQuadFilterWidgetAction>,
}
impl<'a> BiQuadFilterBandStopWidget<'a> {
    fn new_with(
        filter: &'a mut BiQuadFilterBandStopCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> Self {
        Self { filter, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        filter: &'a mut BiQuadFilterBandStopCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterBandStopWidget::new_with(filter, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterBandStopWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut bandwidth = self.filter.bandwidth();
        let (cutoff_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(
                Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                    .text("Cutoff")
                    .suffix(FrequencyHz::UNITS_SUFFIX),
            )
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterBandStopCore::CUTOFF_INDEX.into(),
            ));
        }
        if cutoff_response.inner.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let (bw_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(Slider::new(&mut bandwidth, 0.0..=10.0).text("Bandwidth"))
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterBandStopCore::BANDWIDTH_INDEX.into(),
            ));
        }
        if bw_response.inner.changed() {
            self.filter.set_bandwidth(bandwidth);
        };
        cutoff_response.response | bw_response.response
    }
}
pub struct BiQuadFilterLowPass24dbWidget<'a> {
    filter: &'a mut BiQuadFilterLowPass24dbCore,
    action: &'a mut Option<BiQuadFilterWidgetAction>,
}
impl<'a> BiQuadFilterLowPass24dbWidget<'a> {
    fn new_with(
        filter: &'a mut BiQuadFilterLowPass24dbCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> Self {
        Self { filter, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        filter: &'a mut BiQuadFilterLowPass24dbCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| {
            BiQuadFilterLowPass24dbWidget::new_with(filter, action).ui(ui)
        }
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterLowPass24dbWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut pbr = self.filter.passband_ripple();
        let (cutoff_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(
                Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                    .text("Cutoff")
                    .suffix(FrequencyHz::UNITS_SUFFIX),
            )
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterLowPass24dbCore::CUTOFF_INDEX.into(),
            ));
        }
        if cutoff_response.inner.changed() {
            self.filter.set_cutoff(cutoff.into());
        }
        let (passband_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(Slider::new(&mut pbr, 0.0..=10.0).text("Passband"))
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterLowPass24dbCore::PASSBAND_RIPPLE_INDEX.into(),
            ));
        }
        if passband_response.inner.changed() {
            self.filter.set_passband_ripple(pbr);
        }
        cutoff_response.response | passband_response.response
    }
}

pub struct BiQuadFilterHighPassWidget<'a> {
    filter: &'a mut BiQuadFilterHighPassCore,
    action: &'a mut Option<BiQuadFilterWidgetAction>,
}
impl<'a> BiQuadFilterHighPassWidget<'a> {
    fn new_with(
        filter: &'a mut BiQuadFilterHighPassCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> Self {
        Self { filter, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        filter: &'a mut BiQuadFilterHighPassCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterHighPassWidget::new_with(filter, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterHighPassWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut q = self.filter.q();
        let (cutoff_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(
                Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                    .text("Cutoff")
                    .suffix(FrequencyHz::UNITS_SUFFIX),
            )
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterHighPassCore::CUTOFF_INDEX.into(),
            ));
        }
        if cutoff_response.inner.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let (q_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(Slider::new(&mut q, 0.0..=10.0).text("Q"))
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterHighPassCore::Q_INDEX.into(),
            ));
        }
        if q_response.inner.changed() {
            self.filter.set_q(q);
        };
        cutoff_response.response | q_response.response
    }
}

pub struct BiQuadFilterAllPassWidget<'a> {
    filter: &'a mut BiQuadFilterAllPassCore,
    action: &'a mut Option<BiQuadFilterWidgetAction>,
}
impl<'a> BiQuadFilterAllPassWidget<'a> {
    fn new(
        filter: &'a mut BiQuadFilterAllPassCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> Self {
        Self { filter, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        filter: &'a mut BiQuadFilterAllPassCore,
        action: &'a mut Option<BiQuadFilterWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterAllPassWidget::new(filter, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterAllPassWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut q = self.filter.q();
        let (cutoff_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(
                Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                    .text("Cutoff")
                    .suffix(FrequencyHz::UNITS_SUFFIX),
            )
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterAllPassCore::CUTOFF_INDEX.into(),
            ));
        }
        if cutoff_response.inner.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let (q_response, payload) = ui.dnd_drop_zone(Frame::default(), |ui| {
            ui.add(Slider::new(&mut q, 0.0..=10.0).text("Q"))
        });
        if let Some(source) = payload {
            *self.action = Some(BiQuadFilterWidgetAction::Link(
                *source,
                BiQuadFilterAllPassCore::Q_INDEX.into(),
            ));
        }
        if q_response.inner.changed() {
            self.filter.set_q(q);
        };
        cutoff_response.response | q_response.response
    }
}
