// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use eframe::egui::{Slider, Widget};

pub struct BiQuadFilterBandPassWidget<'a> {
    filter: &'a mut crate::cores::BiQuadFilterBandPass,
}
impl<'a> BiQuadFilterBandPassWidget<'a> {
    fn new_with(filter: &'a mut crate::cores::BiQuadFilterBandPass) -> Self {
        Self { filter }
    }

    pub fn widget(
        filter: &'a mut crate::cores::BiQuadFilterBandPass,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterBandPassWidget::new_with(filter).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterBandPassWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut bw = self.filter.bandwidth();
        let cutoff_response = ui.add(
            Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                .text("Cutoff")
                .suffix(FrequencyHz::UNITS_SUFFIX),
        );
        if cutoff_response.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let bw_response = ui.add(Slider::new(&mut bw, 0.0..=10.0).text("Bandwidth"));
        if bw_response.changed() {
            self.filter.set_bandwidth(bw);
        };
        cutoff_response | bw_response
    }
}
pub struct BiQuadFilterBandStopWidget<'a> {
    filter: &'a mut crate::cores::BiQuadFilterBandStop,
}
impl<'a> BiQuadFilterBandStopWidget<'a> {
    fn new_with(filter: &'a mut crate::cores::BiQuadFilterBandStop) -> Self {
        Self { filter }
    }

    pub fn widget(
        filter: &'a mut crate::cores::BiQuadFilterBandStop,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterBandStopWidget::new_with(filter).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterBandStopWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut bandwidth = self.filter.bandwidth();
        let cutoff_response = ui.add(
            Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                .text("Cutoff")
                .suffix(FrequencyHz::UNITS_SUFFIX),
        );
        if cutoff_response.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let bw_response = ui.add(Slider::new(&mut bandwidth, 0.0..=10.0).text("Bandwidth"));
        if bw_response.changed() {
            self.filter.set_bandwidth(bandwidth);
        };
        cutoff_response | bw_response
    }
}
pub struct BiQuadFilterLowPass24dbWidget<'a> {
    filter: &'a mut crate::cores::BiQuadFilterLowPass24db,
}
impl<'a> BiQuadFilterLowPass24dbWidget<'a> {
    fn new_with(filter: &'a mut crate::cores::BiQuadFilterLowPass24db) -> Self {
        Self { filter }
    }

    pub fn widget(
        filter: &'a mut crate::cores::BiQuadFilterLowPass24db,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterLowPass24dbWidget::new_with(filter).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterLowPass24dbWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut pbr = self.filter.passband_ripple();
        let cutoff_response = ui.add(
            Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                .text("Cutoff")
                .suffix(FrequencyHz::UNITS_SUFFIX),
        );
        if cutoff_response.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let passband_response = ui.add(Slider::new(&mut pbr, 0.0..=10.0).text("Passband"));
        if passband_response.changed() {
            self.filter.set_passband_ripple(pbr);
        };
        cutoff_response | passband_response
    }
}

pub struct BiQuadFilterHighPassWidget<'a> {
    filter: &'a mut crate::cores::BiQuadFilterHighPass,
}
impl<'a> BiQuadFilterHighPassWidget<'a> {
    fn new_with(filter: &'a mut crate::cores::BiQuadFilterHighPass) -> Self {
        Self { filter }
    }

    pub fn widget(
        filter: &'a mut crate::cores::BiQuadFilterHighPass,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterHighPassWidget::new_with(filter).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterHighPassWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut q = self.filter.q();
        let cutoff_response = ui.add(
            Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                .text("Cutoff")
                .suffix(FrequencyHz::UNITS_SUFFIX),
        );
        if cutoff_response.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let q_response = ui.add(Slider::new(&mut q, 0.0..=10.0).text("Q"));
        if q_response.changed() {
            self.filter.set_q(q);
        };
        cutoff_response | q_response
    }
}

pub struct BiQuadFilterAllPassWidget<'a> {
    filter: &'a mut crate::cores::BiQuadFilterAllPass,
}
impl<'a> BiQuadFilterAllPassWidget<'a> {
    fn new(filter: &'a mut crate::cores::BiQuadFilterAllPass) -> Self {
        Self { filter }
    }

    pub fn widget(
        filter: &'a mut crate::cores::BiQuadFilterAllPass,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| BiQuadFilterAllPassWidget::new(filter).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for BiQuadFilterAllPassWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.filter.cutoff().0;
        let mut q = self.filter.q();
        let cutoff_response = ui.add(
            Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                .text("Cutoff")
                .suffix(FrequencyHz::UNITS_SUFFIX),
        );
        if cutoff_response.changed() {
            self.filter.set_cutoff(cutoff.into());
        };
        let q_response = ui.add(Slider::new(&mut q, 0.0..=10.0).text("Q"));
        if q_response.changed() {
            self.filter.set_q(q);
        };
        cutoff_response | q_response
    }
}
