// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{Slider, Widget};
use ensnare_core::{prelude::*, types::FrequencyRange};

/// Wraps a [BiQuadFilterLowPass24dbWidget] as a [Widget](eframe::egui::Widget).
pub fn bi_quad_filter_low_pass_24db<'a>(
    inner: &'a mut ensnare_core::stuff::filter::BiQuadFilterLowPass24db,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| BiQuadFilterLowPass24dbWidget::new(inner).ui(ui)
}
struct BiQuadFilterLowPass24dbWidget<'a> {
    inner: &'a mut ensnare_core::stuff::filter::BiQuadFilterLowPass24db,
}
impl<'a> BiQuadFilterLowPass24dbWidget<'a> {
    fn new(inner: &'a mut ensnare_core::stuff::filter::BiQuadFilterLowPass24db) -> Self {
        Self { inner }
    }
}
impl<'a> Widget for BiQuadFilterLowPass24dbWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.inner.cutoff().0;
        let mut pbr = self.inner.passband_ripple();
        let cutoff_response = ui.add(
            Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                .text("Cutoff")
                .suffix(FrequencyHz::UNITS_SUFFIX),
        );
        if cutoff_response.changed() {
            self.inner.set_cutoff(cutoff.into());
        };
        let passband_response = ui.add(Slider::new(&mut pbr, 0.0..=10.0).text("Passband"));
        if passband_response.changed() {
            self.inner.set_passband_ripple(pbr);
        };
        cutoff_response | passband_response
    }
}
