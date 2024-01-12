// Copyright (c) 2023 Mike Tsao. All rights reserved.

use anyhow::anyhow;
use eframe::{
    egui::{self, ComboBox, Sense, Slider, Widget},
    emath::RectTransform,
    epaint::{pos2, Color32, Rect, RectShape, Rounding, Stroke},
};
use ensnare_core::{
    generators::Waveform,
    types::{FrequencyHz, FrequencyRange},
};
use ensnare_core::{prelude::*, rng::Rng};
use spectrum_analyzer::{scaling::divide_by_N_sqrt, FrequencyLimit};
use strum::IntoEnumIterator;

/// A [Widget](eframe::egui::Widget) for picking an oscillator waveform.
pub fn waveform<'a>(waveform: &'a mut Waveform) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| WaveformWidget::new(waveform).ui(ui)
}

#[derive(Debug)]
struct WaveformWidget<'a> {
    waveform: &'a mut Waveform,
}
impl<'a> WaveformWidget<'a> {
    pub fn new(waveform: &'a mut Waveform) -> Self {
        Self { waveform }
    }
}
impl<'a> eframe::egui::Widget for WaveformWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut r = ComboBox::new(ui.next_auto_id(), "Waveform")
            .selected_text(self.waveform.to_string())
            .show_ui(ui, |ui| {
                let mut bool_response = false;
                for w in Waveform::iter() {
                    let s: &'static str = w.into();
                    if ui.selectable_value(self.waveform, w, s).changed() {
                        bool_response = true;
                    }
                }
                bool_response
            });
        if let Some(inner) = r.inner {
            if inner {
                r.response.mark_changed();
            }
        }
        r.response
    }
}

/// A [Widget](eframe::egui::Widget) for picking a frequency.
pub fn frequency<'a>(
    range: FrequencyRange,
    frequency: &'a mut FrequencyHz,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| FrequencyWidget::new(range, frequency).ui(ui)
}

#[derive(Debug)]
struct FrequencyWidget<'a> {
    range: FrequencyRange,
    frequency: &'a mut FrequencyHz,
}
impl<'a> FrequencyWidget<'a> {
    pub fn new(range: FrequencyRange, frequency: &'a mut FrequencyHz) -> Self {
        Self { range, frequency }
    }
}
impl<'a> eframe::egui::Widget for FrequencyWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut frequency = self.frequency.0;
        let range = self.range.as_range_frequency_hz();
        let slider = Slider::new(&mut frequency, range.start().0..=range.end().0);
        let response = ui.add(
            slider
                .fixed_decimals(self.range.fixed_digit_count())
                .suffix(FrequencyHz::UNITS_SUFFIX)
                .text("Frequency"),
        );
        if response.changed() {
            *self.frequency = FrequencyHz(frequency);
        }
        response
    }
}

/// Does a quick-and-dirty FFT of the input samples, producing a buffer that
/// is suitable for an unlabeled visualization. If you want labels, then do
/// this transformation yourself so you can display the Hz bucket labels.
///
/// TODO: there's a ton of heap usage in this method. See whether the crate
/// can be enhanced to work better with preallocated buffers.
pub fn analyze_spectrum(slice_1: &[Sample], slice_2: &[Sample]) -> anyhow::Result<Vec<f32>> {
    let rotated: Vec<f32> = slice_1
        .iter()
        .chain(slice_2.iter())
        .map(|s| s.0 as f32)
        .collect();
    let hann_window = spectrum_analyzer::windows::hann_window(&rotated);
    match spectrum_analyzer::samples_fft_to_spectrum(
        &hann_window,
        44100,
        FrequencyLimit::All,
        Some(&divide_by_N_sqrt),
    ) {
        Ok(spectrum) => Ok(spectrum.data().iter().map(|pair| pair.1.val()).collect()),
        Err(e) => Err(anyhow!("samples_fft_to_spectrum failed: {e:?}")),
    }
}

/// Wraps a [FrequencyDomain] as a [Widget](eframe::egui::Widget).
pub fn frequency_domain(values: &[f32]) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| FrequencyDomain::new(values).ui(ui)
}

/// Wraps a [TimeDomain] as a [Widget](eframe::egui::Widget).
pub fn time_domain<'a>(
    slice_1: &'a [Sample],
    slice_2: &'a [Sample],
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| TimeDomain::new(slice_1, slice_2).ui(ui)
}

/// Displays a series of [Sample]s in the time domain. That's a fancy way of
/// saying it shows the amplitudes.
///
/// The series is passed in as two slices because we expect that callers will
/// have used a ring buffer to hold the incoming samples, and we don't want to
/// require a contiguous buffer, which would require expensive ring-buffer
/// rotations most of the time.
#[derive(Debug)]
pub struct TimeDomain<'a> {
    slice_1: &'a [Sample],
    slice_2: &'a [Sample],
}
impl<'a> TimeDomain<'a> {
    fn new(slice_1: &'a [Sample], slice_2: &'a [Sample]) -> Self {
        Self { slice_1, slice_2 }
    }
}
impl<'a> eframe::egui::Widget for TimeDomain<'a> {
    fn ui(self, ui: &mut egui::Ui) -> eframe::egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let rect = response.rect.shrink(1.0);

        let buffer_len = self.slice_1.len() + self.slice_2.len();
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(
                0.0..=buffer_len as f32,
                Sample::MAX.0 as f32..=Sample::MIN.0 as f32,
            ),
            rect,
        );
        let mut shapes = Vec::default();

        shapes.push(eframe::epaint::Shape::Rect(RectShape::new(
            rect,
            Rounding::same(3.0),
            ui.visuals().window_fill,
            ui.visuals().window_stroke,
        )));

        for (i, sample) in self.slice_1.iter().chain(self.slice_2).enumerate() {
            shapes.push(eframe::epaint::Shape::LineSegment {
                points: [
                    to_screen * pos2(i as f32, Sample::MIN.0 as f32),
                    to_screen * pos2(i as f32, sample.0 as f32),
                ],
                stroke: Stroke::new(1.0, Color32::YELLOW),
            })
        }

        painter.extend(shapes);
        response
    }
}

/// Displays a series of [Sample]s in the frequency domain. Or, to put it
/// another way, shows a spectrum analysis of a clip.
#[derive(Debug)]
pub struct FrequencyDomain<'a> {
    values: &'a [f32],
}
impl<'a> FrequencyDomain<'a> {
    fn new(values: &'a [f32]) -> Self {
        Self { values }
    }
}
impl<'a> eframe::egui::Widget for FrequencyDomain<'a> {
    fn ui(self, ui: &mut egui::Ui) -> eframe::egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let rect = response.rect.shrink(1.0);

        let buf_min = 0.0;
        let buf_max = 1.0;

        #[allow(unused_variables)]
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(0.0..=self.values.len() as f32, buf_max..=buf_min),
            rect,
        );
        let mut shapes = Vec::default();

        shapes.push(eframe::epaint::Shape::Rect(RectShape::new(
            rect,
            Rounding::same(3.0),
            ui.visuals().window_fill,
            ui.visuals().window_stroke,
        )));

        for (i, value) in self.values.iter().enumerate() {
            shapes.push(eframe::epaint::Shape::LineSegment {
                points: [
                    to_screen * pos2(i as f32, buf_min),
                    to_screen * pos2(i as f32, *value),
                ],
                stroke: Stroke {
                    width: 1.0,
                    color: Color32::YELLOW,
                },
            });
        }

        painter.extend(shapes);
        response
    }
}
