// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    generators::{Oscillator, OscillatorParams, Waveform},
    midi::prelude::*,
    prelude::*,
    traits::prelude::*,
};
use eframe::{
    egui::{Frame, Label, Response, Ui},
    emath,
    epaint::{self, pos2, vec2, Color32, Pos2, Rect, Stroke},
};
use ensnare_proc_macros::{Control, IsController, IsControllerEffect, Params, Uid};
use serde::{Deserialize, Serialize};
use std::ops::{Range, RangeInclusive};

pub mod arpeggiator;
pub mod even_smaller_sequencer;
pub mod mini_sequencer;
pub mod old_sequencer;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum SignalPassthroughType {
    #[default]
    /// Maps -1.0..=1.0 to 0.0..=1.0. Min amplitude becomes 0.0, silence becomes
    /// 0.5, and max amplitude becomes 1.0.
    Compressed,

    /// Based on the absolute value of the sample. Silence is 0.0, and max
    /// amplitude of either polarity is 1.0.
    Amplitude,

    /// Based on the absolute value of the sample. Silence is 1.0, and max
    /// amplitude of either polarity is 0.0.
    AmplitudeInverted,
}

/// Uses an input signal as a control source. Transformation depends on
/// configuration. Uses the standard Sample::from(StereoSample) methodology of
/// averaging the two channels to create a single signal.
#[derive(Control, Debug, Default, IsControllerEffect, Params, Uid, Serialize, Deserialize)]
pub struct SignalPassthroughController {
    uid: Uid,
    passthrough_type: SignalPassthroughType,

    #[serde(skip)]
    control_value: ControlValue,

    // We don't issue consecutive identical events, so we need to remember
    // whether we've sent the current value.
    #[serde(skip)]
    has_value_been_issued: bool,

    #[serde(skip)]
    is_performing: bool,
}
impl Serializable for SignalPassthroughController {}
impl Configurable for SignalPassthroughController {}
impl Controls for SignalPassthroughController {
    fn update_time(&mut self, _range: &Range<MusicalTime>) {
        // We can ignore because we already have our own de-duplicating logic.
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if !self.is_performing {
            return;
        }
        if !self.has_value_been_issued {
            self.has_value_been_issued = true;
            control_events_fn(self.uid, EntityEvent::Control(self.control_value))
        }
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn skip_to_start(&mut self) {}

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl HandlesMidi for SignalPassthroughController {}
impl TransformsAudio for SignalPassthroughController {
    fn transform_audio(&mut self, input_sample: StereoSample) -> StereoSample {
        let sample: Sample = input_sample.into();
        let control_value = match self.passthrough_type {
            SignalPassthroughType::Compressed => {
                let as_bipolar_normal: BipolarNormal = sample.into();
                as_bipolar_normal.into()
            }
            SignalPassthroughType::Amplitude => ControlValue(sample.0.abs()),
            SignalPassthroughType::AmplitudeInverted => ControlValue(1.0 - sample.0.abs()),
        };
        if self.control_value != control_value {
            self.has_value_been_issued = false;
            self.control_value = control_value;
        }
        input_sample
    }

    fn transform_channel(&mut self, _channel: usize, _input_sample: Sample) -> Sample {
        // We've overridden transform_audio(), so nobody should be calling this
        // method.
        todo!();
    }
}
impl Displays for SignalPassthroughController {
    fn ui(&mut self, ui: &mut Ui) -> eframe::egui::Response {
        ui.label(self.name())
    }
}
impl SignalPassthroughController {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_amplitude_passthrough_type() -> Self {
        Self {
            passthrough_type: SignalPassthroughType::Amplitude,
            ..Default::default()
        }
    }

    pub fn new_amplitude_inverted_passthrough_type() -> Self {
        Self {
            passthrough_type: SignalPassthroughType::AmplitudeInverted,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WaveformWidget {}
impl Displays for WaveformWidget {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let time = ui.input(|i| i.time);

            let desired_size = ui.available_width() * vec2(1.0, 0.35);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

            let mut shapes = vec![];

            for &mode in &[2, 3, 5] {
                let mode = mode as f64;
                let n = 120;
                let speed = 1.5;

                let points: Vec<Pos2> = (0..=n)
                    .map(|i| {
                        let t = i as f64 / (n as f64);
                        let amp = (time * speed * mode).sin() / mode;
                        let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
                        to_screen * pos2(t as f32, y as f32)
                    })
                    .collect();

                let thickness = 10.0 / mode as f32;
                shapes.push(epaint::Shape::line(points, Stroke::new(thickness, color)));
            }

            ui.painter().extend(shapes);
        });
        ui.vertical_centered(|ui| ui.add(Label::new("hello!")))
            .inner
    }
}

/// Uses an internal LFO as a control source.
#[derive(Debug, Control, IsController, Params, Uid, Serialize, Deserialize)]
pub struct LfoController {
    uid: Uid,

    #[control]
    #[params]
    waveform: Waveform,
    #[control]
    #[params]
    frequency: FrequencyHz,

    oscillator: Oscillator,

    #[serde(skip)]
    is_performing: bool,

    #[serde(skip)]
    waveform_widget: WaveformWidget,

    #[serde(skip)]
    time_range: Range<MusicalTime>,

    #[serde(skip)]
    last_frame: usize,
}
impl Serializable for LfoController {}
impl Configurable for LfoController {
    fn sample_rate(&self) -> SampleRate {
        self.oscillator.sample_rate()
    }
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.oscillator.update_sample_rate(sample_rate);
    }
}
impl Controls for LfoController {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.time_range = range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        let frames = self.time_range.start.as_frames(
            Tempo::from(120),
            SampleRate::from(self.oscillator.sample_rate()),
        );

        if frames != self.last_frame {
            let tick_count = if frames >= self.last_frame {
                // normal case; oscillator should advance the calculated number
                // of frames
                //
                // TODO: this is unlikely to be frame-accurate, because
                // Orchestrator is currently going from frames -> beats
                // (inaccurate), and then we're going from beats -> frames. We
                // could include frame count in update_time(), as discussed in
                // #132, which would mean we don't have to be smart at all about
                // it.
                frames - self.last_frame
            } else {
                self.last_frame = frames;
                0
            };
            self.last_frame += tick_count;
            self.oscillator.tick(tick_count);
        }
        control_events_fn(
            self.uid,
            EntityEvent::Control(self.oscillator.value().into()),
        );
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn skip_to_start(&mut self) {
        // TODO: think how important it is for LFO oscillator to start at zero
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl HandlesMidi for LfoController {}
impl LfoController {
    pub fn new_with(params: &LfoControllerParams) -> Self {
        Self {
            uid: Default::default(),
            oscillator: Oscillator::new_with(&OscillatorParams {
                waveform: params.waveform,
                frequency: params.frequency,
                ..Default::default()
            }),
            waveform: params.waveform(),
            frequency: params.frequency(),
            is_performing: false,

            waveform_widget: Default::default(),
            time_range: Default::default(),
            last_frame: Default::default(),
        }
    }

    pub const fn frequency_range() -> RangeInclusive<ParameterType> {
        0.0..=100.0
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
        self.oscillator.set_waveform(waveform);
    }

    pub fn frequency(&self) -> FrequencyHz {
        self.frequency
    }

    pub fn set_frequency(&mut self, frequency: FrequencyHz) {
        self.frequency = frequency;
        self.oscillator.set_frequency(frequency);
    }
}

impl Displays for LfoController {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        // TODO: come up with a better pattern for .changed() to happen at
        // the same level as whoever called show().
        if self.frequency.show(ui, Self::frequency_range()) {
            self.set_frequency(self.frequency);
        }
        if self.waveform.show(ui).inner.is_some() {
            self.set_waveform(self.waveform);
        }
        self.waveform_widget.ui(ui)
    }
}

/// [Timer] runs for a specified amount of time, then indicates that it's done.
/// It is useful when you need something to happen after a certain amount of
/// wall-clock time, rather than musical time.
#[derive(Debug, Control, IsController, Uid, Serialize, Deserialize)]
pub struct Timer {
    uid: Uid,

    duration: MusicalTime,

    #[serde(skip)]
    is_performing: bool,

    #[serde(skip)]
    is_finished: bool,

    #[serde(skip)]
    end_time: Option<MusicalTime>,
}
impl Serializable for Timer {}
#[allow(missing_docs)]
impl Timer {
    pub fn new_with(duration: MusicalTime) -> Self {
        Self {
            uid: Default::default(),
            duration,
            is_performing: false,
            is_finished: false,
            end_time: Default::default(),
        }
    }

    pub fn duration(&self) -> MusicalTime {
        self.duration
    }

    pub fn set_duration(&mut self, duration: MusicalTime) {
        self.duration = duration;
    }
}
impl HandlesMidi for Timer {}
impl Configurable for Timer {}
impl Controls for Timer {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        if self.is_performing {
            if self.duration == MusicalTime::default() {
                // Zero-length timers fire immediately.
                self.is_finished = true;
            } else {
                if let Some(end_time) = self.end_time {
                    if range.contains(&end_time) {
                        self.is_finished = true;
                    }
                } else {
                    // The first time we're called with an update_time() while
                    // performing, we take that as the start of the timer.
                    self.end_time = Some(range.start + self.duration);
                }
            }
        }
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Displays for Timer {}

// TODO: needs tests!
/// [Trigger] issues a control signal after a specified amount of time.
#[derive(Debug, Control, IsController, Uid, Serialize, Deserialize)]
pub struct Trigger {
    uid: Uid,

    timer: Timer,

    value: ControlValue,

    has_triggered: bool,
    is_performing: bool,
}
impl Serializable for Trigger {}
impl Controls for Trigger {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.timer.update_time(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.timer.is_finished() && self.is_performing && !self.has_triggered {
            self.has_triggered = true;
            control_events_fn(self.uid, EntityEvent::Control(self.value));
        }
    }

    fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }

    fn play(&mut self) {
        self.is_performing = true;
        self.timer.play();
    }

    fn stop(&mut self) {
        self.is_performing = false;
        self.timer.stop();
    }

    fn skip_to_start(&mut self) {
        self.has_triggered = false;
        self.timer.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Configurable for Trigger {
    fn sample_rate(&self) -> SampleRate {
        self.timer.sample_rate()
    }
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.timer.update_sample_rate(sample_rate)
    }
}
impl HandlesMidi for Trigger {}
impl Trigger {
    pub fn new_with(timer: Timer, value: ControlValue) -> Self {
        Self {
            uid: Default::default(),
            timer,
            value,
            has_triggered: false,
            is_performing: false,
        }
    }

    pub fn value(&self) -> ControlValue {
        self.value
    }

    pub fn set_value(&mut self, value: ControlValue) {
        self.value = value;
    }
}
impl Displays for Trigger {
    fn ui(&mut self, ui: &mut Ui) -> eframe::egui::Response {
        ui.label(self.name())
    }
}
