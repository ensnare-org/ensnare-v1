// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    prelude::*,
    time::Seconds,
    traits::{prelude::*, GeneratesEnvelope},
    widgets::{audio::waveform, core::drag_normal, generators::envelope_shaper},
};
use eframe::{
    egui::{Sense, Ui},
    emath,
    epaint::{pos2, Color32, PathShape, Pos2, Rect, Shape, Stroke, Vec2},
};
use ensnare_proc_macros::{Control, Params};
use kahan::KahanSum;
use nalgebra::{Matrix3, Matrix3x1};
use serde::{Deserialize, Serialize};
use std::{f64::consts::PI, fmt::Debug};
use strum::EnumCount;
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, FromRepr, IntoStaticStr};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    EnumCountMacro,
    EnumIter,
    FromRepr,
    PartialEq,
    IntoStaticStr,
    Serialize,
    Deserialize,
)]
#[serde(rename = "waveform", rename_all = "kebab-case")]
pub enum Waveform {
    None,
    #[default]
    Sine,
    Square,
    PulseWidth(Normal),
    Triangle,
    Sawtooth,
    Noise,
    DebugZero,
    DebugMax,
    DebugMin,

    TriangleSine, // TODO
}

// TODO: the existence of this conversion is bad. PWM is just different. Come up
// with some other way to automate waveform changes.
impl From<ControlValue> for Waveform {
    fn from(value: ControlValue) -> Self {
        Waveform::from_repr((value.0 * Waveform::COUNT as ParameterType) as usize)
            .unwrap_or_default()
    }
}
impl From<Waveform> for ControlValue {
    fn from(value: Waveform) -> Self {
        // TODO: is there a way to get the discriminant cheaply when the
        // enum is not
        // [unit-only](https://doc.rust-lang.org/reference/items/enumerations.html)?
        ((match value {
            Waveform::None => 0,
            Waveform::Sine => 1,
            Waveform::Square => 2,
            Waveform::PulseWidth(_) => 3,
            Waveform::Triangle => 4,
            Waveform::Sawtooth => 5,
            Waveform::Noise => 6,
            Waveform::DebugZero => 7,
            Waveform::DebugMax => 8,
            Waveform::DebugMin => 9,
            Waveform::TriangleSine => 10,
        } as f64)
            / Waveform::COUNT as f64)
            .into()
    }
}

impl OscillatorParams {
    pub fn default_with_waveform(waveform: Waveform) -> Self {
        Self {
            waveform,
            frequency: FrequencyHz::from(440.0),
            ..Default::default()
        }
    }
}

#[derive(Debug, Control, Params, Serialize, Deserialize)]
pub struct Oscillator {
    #[control]
    #[params]
    pub(crate) waveform: Waveform,

    /// Hertz. Any positive number. 440 = A4
    #[control]
    #[params]
    frequency: FrequencyHz,

    /// if not zero, then ignores the `frequency` field and uses this one
    /// instead. TODO: Option<>
    #[control]
    #[params]
    fixed_frequency: Option<FrequencyHz>,

    /// Designed for pitch correction at construction time.
    #[control]
    #[params]
    frequency_tune: Ratio,

    /// [-1, 1] is typical range, with -1 halving the frequency, and 1 doubling
    /// it. Designed for LFOs.
    #[control]
    #[params]
    frequency_modulation: BipolarNormal,

    /// A factor applied to the root frequency. It is used for FM synthesis.
    #[control]
    #[params]
    linear_frequency_modulation: ParameterType,

    /// working variables to generate semi-deterministic noise.
    noise_x1: u32,
    noise_x2: u32,

    /// An internal copy of the current sample rate.
    #[serde(skip)]
    sample_rate: SampleRate,

    /// The internal clock. Advances once per tick().
    ///
    #[serde(skip)]
    ticks: usize,

    #[serde(skip)]
    signal: BipolarNormal,

    // It's important for us to remember the "cursor" in the current waveform,
    // because the frequency can change over time, so recalculating the position
    // as if the current frequency were always the frequency leads to click,
    // pops, transients, and suckage.
    //
    // Needs Kahan summation algorithm to avoid accumulation of FP errors.
    #[serde(skip)]
    cycle_position: KahanSum<f64>,

    #[serde(skip)]
    delta: f64,
    #[serde(skip)]
    delta_updated: bool,

    // Whether this oscillator's owner should sync other oscillators to this
    // one. Calculated during tick().
    #[serde(skip)]
    should_sync: bool,

    // If this is a synced oscillator, then whether we should reset our waveform
    // to the start.
    #[serde(skip)]
    is_sync_pending: bool,

    // Set on init and reset().
    #[serde(skip)]
    reset_handled: bool,
}
impl Default for Oscillator {
    fn default() -> Self {
        Self {
            waveform: Default::default(),
            frequency: FrequencyHz(440.0),
            fixed_frequency: Default::default(),
            frequency_tune: Default::default(),
            frequency_modulation: Default::default(),
            linear_frequency_modulation: Default::default(),
            noise_x1: 0x70f4f854,
            noise_x2: 0xe1e9f0a7,
            sample_rate: Default::default(),
            ticks: Default::default(),
            signal: Default::default(),
            cycle_position: Default::default(),
            delta: Default::default(),
            delta_updated: Default::default(),
            should_sync: Default::default(),
            is_sync_pending: Default::default(),
            reset_handled: Default::default(),
        }
    }
}
impl Generates<BipolarNormal> for Oscillator {
    fn value(&self) -> BipolarNormal {
        self.signal
    }

    fn generate_batch_values(&mut self, values: &mut [BipolarNormal]) {
        for v in values {
            self.tick(1);
            *v = self.value();
        }
    }
}
impl Configurable for Oscillator {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.reset_handled = false;
    }
}
impl Ticks for Oscillator {
    fn tick(&mut self, tick_count: usize) {
        for _ in 0..tick_count {
            if !self.reset_handled {
                self.ticks = 0; // TODO: this might not be the right thing to do

                self.update_delta();
                self.cycle_position =
                    KahanSum::new_with_value((self.delta * self.ticks as f64).fract());
            } else {
                self.ticks += 1;
            }

            let cycle_position = self.calculate_cycle_position();
            let amplitude_for_position = self.amplitude_for_position(self.waveform, cycle_position);
            self.signal = BipolarNormal::from(amplitude_for_position);

            // We need this to be at the end of tick() because any code running
            // during tick() might look at it.
            self.reset_handled = true;
        }
    }
}
impl Oscillator {
    pub fn new_with(params: &OscillatorParams) -> Self {
        Self {
            waveform: params.waveform(),
            frequency: params.frequency(),
            // TODO https://github.com/sowbug/groove/issues/135
            // fixed_frequency: params.fixed_frequency(),
            frequency_tune: params.frequency_tune(),
            frequency_modulation: params.frequency_modulation(),
            ..Default::default()
        }
    }

    fn adjusted_frequency(&self) -> FrequencyHz {
        let unmodulated_frequency = if let Some(fixed_frequency) = self.fixed_frequency {
            fixed_frequency
        } else {
            self.frequency * self.frequency_tune
        };
        unmodulated_frequency
            * FrequencyHz(
                2.0f64.powf(self.frequency_modulation.0) + self.linear_frequency_modulation,
            )
    }

    pub fn set_frequency(&mut self, frequency: FrequencyHz) {
        self.frequency = frequency;
        self.delta_updated = false;
    }

    pub fn set_fixed_frequency(&mut self, frequency: FrequencyHz) {
        self.fixed_frequency = Some(frequency);
        self.delta_updated = false;
    }

    pub fn set_frequency_modulation(&mut self, frequency_modulation: BipolarNormal) {
        self.frequency_modulation = frequency_modulation;
        self.delta_updated = false;
    }

    pub fn set_linear_frequency_modulation(&mut self, linear_frequency_modulation: ParameterType) {
        self.linear_frequency_modulation = linear_frequency_modulation;
        self.delta_updated = false;
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn frequency_modulation(&self) -> BipolarNormal {
        self.frequency_modulation
    }

    pub fn linear_frequency_modulation(&self) -> ParameterType {
        self.linear_frequency_modulation
    }

    pub fn frequency(&self) -> FrequencyHz {
        self.frequency
    }

    pub fn should_sync(&self) -> bool {
        self.should_sync
    }

    pub fn sync(&mut self) {
        self.is_sync_pending = true;
    }

    fn update_delta(&mut self) {
        if !self.delta_updated {
            self.delta = (self.adjusted_frequency() / FrequencyHz::from(self.sample_rate.0)).0;

            // This resets the accumulated error.
            self.cycle_position = KahanSum::new_with_value(self.cycle_position.sum());

            self.delta_updated = true;
        }
    }

    fn calculate_cycle_position(&mut self) -> f64 {
        self.update_delta();

        // Process any sync() calls since last tick. The point of sync() is to
        // restart the synced oscillator's cycle, so position zero is correct.
        //
        // Note that if the clock is reset, then synced oscillators will
        // momentarily have the wrong cycle_position, because in their own
        // check_for_clock_reset() they'll calculate a position, but then in
        // this method they'll detect that they're supposed to sync and will
        // reset to zero. This also means that for one cycle, the main
        // oscillator will have started at a synthetic starting point, but the
        // synced ones will have started at zero. I don't think this is
        // important.
        if self.is_sync_pending {
            self.is_sync_pending = false;
            self.cycle_position = Default::default();
        }

        // If we haven't just reset, add delta to the previous position and mod
        // 1.0.
        let next_cycle_position_unrounded = if !self.reset_handled {
            0.0
        } else {
            self.cycle_position += self.delta;
            self.cycle_position.sum()
        };

        self.should_sync = if !self.reset_handled {
            // If we're in the first post-reset tick(), then we want other
            // oscillators to sync.
            true
        } else if next_cycle_position_unrounded > 0.999999999999 {
            // This special case is to deal with an FP precision issue that was
            // causing square waves to flip one sample too late in unit tests. We
            // take advantage of it to also record whether we should signal to
            // synced oscillators that it's time to sync.

            // Very extreme FM synthesis beta values can cause this assertion to
            // fail, so it's disabled. I don't think it's a real problem because
            // all the waveform calculators handle cycles >= 1.0 as if they were
            // mod 1.0, and the assertion otherwise never fired after initial
            // Oscillator development.
            //
            // I'm keeping it here to keep myself humble.
            //
            // debug_assert_lt!(next_cycle_position_unrounded, 2.0);

            self.cycle_position += -1.0;
            true
        } else {
            false
        };

        self.cycle_position.sum()
    }

    // https://en.wikipedia.org/wiki/Sine_wave
    // https://en.wikipedia.org/wiki/Square_wave
    // https://en.wikipedia.org/wiki/Triangle_wave
    // https://en.wikipedia.org/wiki/Sawtooth_wave
    // https://www.musicdsp.org/en/latest/Synthesis/216-fast-whitenoise-generator.html
    //
    // Some of these have seemingly arbitrary phase-shift constants in their
    // formulas. The reason for them is to ensure that every waveform starts at
    // amplitude zero, which makes it a lot easier to avoid transients when a
    // waveform starts up. See Pirkle DSSPC++ p.133 for visualization.
    fn amplitude_for_position(&mut self, waveform: Waveform, cycle_position: f64) -> f64 {
        match waveform {
            Waveform::None => 0.0,
            Waveform::Sine => (cycle_position * 2.0 * PI).sin(),
            Waveform::Square => -(cycle_position - 0.5).signum(),
            Waveform::PulseWidth(duty_cycle) => -(cycle_position - duty_cycle.0).signum(),
            Waveform::Triangle => {
                4.0 * (cycle_position - (0.5 + cycle_position).floor()).abs() - 1.0
            }
            Waveform::Sawtooth => 2.0 * (cycle_position - (0.5 + cycle_position).floor()),
            Waveform::Noise => {
                // TODO: this is stateful, so random access will sound different
                // from sequential, as will different sample rates. It also
                // makes this method require mut. Is there a noise algorithm
                // that can modulate on time_seconds? (It's a complicated
                // question, potentially.)
                self.noise_x1 ^= self.noise_x2;
                let tmp = 2.0 * (self.noise_x2 as f64 - (u32::MAX as f64 / 2.0)) / u32::MAX as f64;
                (self.noise_x2, _) = self.noise_x2.overflowing_add(self.noise_x1);
                tmp
            }
            // TODO: figure out whether this was an either-or
            Waveform::TriangleSine => {
                4.0 * (cycle_position - (0.75 + cycle_position).floor() + 0.25).abs() - 1.0
            }
            Waveform::DebugZero => 0.0,
            Waveform::DebugMax => 1.0,
            Waveform::DebugMin => -1.0,
        }
    }

    pub fn set_frequency_tune(&mut self, frequency_tune: Ratio) {
        self.frequency_tune = frequency_tune;
    }

    pub fn fixed_frequency(&self) -> Option<FrequencyHz> {
        self.fixed_frequency
    }

    pub fn frequency_tune(&self) -> Ratio {
        self.frequency_tune
    }
}

/// Wraps an [OscillatorWidget] as a [Widget](eframe::egui::Widget).
pub fn oscillator<'a>(oscillator: &'a mut Oscillator) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| OscillatorWidget::new(oscillator).ui(ui)
}
/// An egui widget for [Oscillator].
#[derive(Debug)]
struct OscillatorWidget<'a> {
    oscillator: &'a mut Oscillator,
}
impl<'a> Displays for OscillatorWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(waveform(&mut self.oscillator.waveform))
    }
}
impl<'a> OscillatorWidget<'a> {
    fn new(oscillator: &'a mut Oscillator) -> Self {
        Self { oscillator }
    }
}

// TODO: see https://corrode.dev/blog/enums/ and mull over it
#[derive(Clone, Copy, Debug, Default)]
enum State {
    #[default]
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
    Shutdown,
}

impl EnvelopeParams {
    pub fn new_with(attack: Normal, decay: Normal, sustain: Normal, release: Normal) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
        }
    }

    // The #[control] #[params] macro system doesn't currently let us override derived
    // Default, and I wasn't sure whether it was right to default Normal to 1.0,
    // so I'm creating a custom default method. I think that only test/toy code
    // would rely on defaults for an envelope.
    pub fn safe_default() -> Self {
        Self::new_with(0.002.into(), 0.005.into(), 0.8.into(), 0.01.into())
    }
}

#[derive(Debug, Default, Control, Params, Serialize, Deserialize)]
pub struct Envelope {
    #[control]
    #[params]
    attack: Normal,
    #[control]
    #[params]
    decay: Normal,
    #[control]
    #[params]
    sustain: Normal,
    #[control]
    #[params]
    release: Normal,

    #[serde(skip)]
    sample_rate: SampleRate,
    #[serde(skip)]
    state: State,
    #[serde(skip)]
    was_reset: bool,

    #[serde(skip)]
    ticks: usize,
    #[serde(skip)]
    time: Seconds,

    #[serde(skip)]
    uncorrected_amplitude: KahanSum<f64>,
    #[serde(skip)]
    corrected_amplitude: f64,
    #[serde(skip)]
    delta: f64,
    #[serde(skip)]
    amplitude_target: f64,
    #[serde(skip)]
    time_target: Seconds,

    // Whether the amplitude was set to an explicit value during this frame,
    // which means that the caller is expecting to get an amplitude of that
    // exact value, which means that we should return the PRE-update value
    // rather than the usual post-update value.
    #[serde(skip)]
    amplitude_was_set: bool,

    // Polynomial coefficients for convex
    #[serde(skip)]
    convex_a: f64,
    #[serde(skip)]
    convex_b: f64,
    #[serde(skip)]
    convex_c: f64,

    // Polynomial coefficients for concave
    #[serde(skip)]
    concave_a: f64,
    #[serde(skip)]
    concave_b: f64,
    #[serde(skip)]
    concave_c: f64,
}
impl GeneratesEnvelope for Envelope {
    fn trigger_attack(&mut self) {
        self.set_state(State::Attack);
    }
    fn trigger_release(&mut self) {
        self.set_state(State::Release);
    }
    fn trigger_shutdown(&mut self) {
        self.set_state(State::Shutdown);
    }
    fn is_idle(&self) -> bool {
        matches!(self.state, State::Idle)
    }
}
impl Generates<Normal> for Envelope {
    fn value(&self) -> Normal {
        Normal::new(self.corrected_amplitude)
    }

    fn generate_batch_values(&mut self, values: &mut [Normal]) {
        // TODO: this is probably no more efficient than calling amplitude()
        // individually, but for now we're just getting the interface right.
        // Later we'll take advantage of it.
        for v in values {
            self.tick(1);
            *v = self.value();
        }
    }
}
impl Configurable for Envelope {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.was_reset = true;
    }
}
impl Ticks for Envelope {
    fn tick(&mut self, tick_count: usize) {
        // TODO: same comment as above about not yet taking advantage of
        // batching
        for _ in 0..tick_count {
            let pre_update_amplitude = self.uncorrected_amplitude.sum();
            if self.was_reset {
                self.was_reset = false;
            } else {
                self.ticks += 1;
                self.update_amplitude();
            }
            self.time = Seconds(self.ticks as f64 / self.sample_rate.0 as f64);

            self.handle_state();

            let linear_amplitude = if self.amplitude_was_set {
                self.amplitude_was_set = false;
                pre_update_amplitude
            } else {
                self.uncorrected_amplitude.sum()
            };
            self.corrected_amplitude = match self.state {
                State::Attack => self.transform_linear_to_convex(linear_amplitude),
                State::Decay | State::Release => self.transform_linear_to_concave(linear_amplitude),
                _ => linear_amplitude,
            };
        }
    }
}
impl Envelope {
    pub const MIN_SECONDS: f64 = 0.0;
    pub const MAX_SECONDS: f64 = 30.0;

    pub fn new_with(params: &EnvelopeParams) -> Self {
        Self {
            attack: params.attack(),
            decay: params.decay(),
            sustain: params.sustain(),
            release: params.release(),
            sample_rate: Default::default(),
            state: State::Idle,
            was_reset: true,
            ticks: Default::default(),
            time: Default::default(),
            uncorrected_amplitude: Default::default(),
            corrected_amplitude: 0.0,
            delta: Default::default(),
            amplitude_target: Default::default(),
            time_target: Default::default(),
            amplitude_was_set: Default::default(),
            convex_a: Default::default(),
            convex_b: Default::default(),
            convex_c: Default::default(),
            concave_a: Default::default(),
            concave_b: Default::default(),
            concave_c: Default::default(),
        }
    }

    pub fn from_seconds_to_normal(seconds: Seconds) -> Normal {
        Normal::new(seconds.0 / Self::MAX_SECONDS)
    }

    pub fn from_normal_to_seconds(normal: Normal) -> Seconds {
        Seconds(normal.0 * Self::MAX_SECONDS)
    }

    fn update_amplitude(&mut self) {
        self.uncorrected_amplitude += self.delta;
    }

    fn handle_state(&mut self) {
        let (next_state, awaiting_target) = match self.state {
            State::Idle => (State::Idle, false),
            State::Attack => (State::Decay, true),
            State::Decay => (State::Sustain, true),
            State::Sustain => (State::Sustain, false),
            State::Release => (State::Idle, true),
            State::Shutdown => (State::Idle, true),
        };
        if awaiting_target && self.has_reached_target() {
            self.set_state(next_state);
        }
    }

    fn has_reached_target(&mut self) -> bool {
        #[allow(clippy::if_same_then_else)]
        let has_hit_target = if self.delta == 0.0 {
            // This is probably a degenerate case, but we don't want to be stuck
            // forever in the current state.
            true
        } else if self.time_target.0 != 0.0 && self.time >= self.time_target {
            // If we have a time target and we've hit it, then we're done even
            // if the amplitude isn't quite there yet.
            true
        } else {
            // Is the difference between the current value and the target
            // smaller than the delta? This is a fancy way of saying we're as
            // close as we're going to get without overshooting the next time.
            (self.uncorrected_amplitude.sum() - self.amplitude_target).abs() < self.delta.abs()
        };

        if has_hit_target {
            // Set to the exact amplitude target in case of precision errors. We
            // don't want to set self.amplitude_was_set here because this is
            // happening after the update, so we'll already be returning the
            // amplitude snapshotted at the right time.
            self.uncorrected_amplitude = KahanSum::new_with_value(self.amplitude_target);
        }
        has_hit_target
    }

    // For all the set_state_() methods, we assume that the prior state actually
    // happened, and that the amplitude is set to a reasonable value. This
    // matters, for example, if attack is zero and decay is non-zero. If we jump
    // straight from idle to decay, then decay is decaying from the idle
    // amplitude of zero, which is wrong.
    fn set_state(&mut self, new_state: State) {
        match new_state {
            State::Idle => {
                self.state = State::Idle;
                self.uncorrected_amplitude = Default::default();
                self.delta = 0.0;
            }
            State::Attack => {
                if self.attack == Normal::minimum() {
                    self.set_explicit_amplitude(Normal::maximum());
                    self.set_state(State::Decay);
                } else {
                    self.state = State::Attack;
                    let target_amplitude = Normal::maximum().0;
                    self.set_target(Normal::maximum(), self.attack, false, false);
                    let current_amplitude = self.uncorrected_amplitude.sum();

                    (self.convex_a, self.convex_b, self.convex_c) = Self::calculate_coefficients(
                        current_amplitude,
                        current_amplitude,
                        (target_amplitude - current_amplitude) / 2.0 + current_amplitude,
                        (target_amplitude - current_amplitude) / 1.5 + current_amplitude,
                        target_amplitude,
                        target_amplitude,
                    );
                }
            }
            State::Decay => {
                if self.decay == Normal::minimum() {
                    self.set_explicit_amplitude(self.sustain);
                    self.set_state(State::Sustain);
                } else {
                    self.state = State::Decay;
                    let target_amplitude = self.sustain.0;
                    self.set_target(self.sustain, self.decay, true, false);
                    let current_amplitude = self.uncorrected_amplitude.sum();
                    (self.concave_a, self.concave_b, self.concave_c) = Self::calculate_coefficients(
                        current_amplitude,
                        current_amplitude,
                        (current_amplitude - target_amplitude) / 2.0 + target_amplitude,
                        (current_amplitude - target_amplitude) / 3.0 + target_amplitude,
                        target_amplitude,
                        target_amplitude,
                    );
                }
            }
            State::Sustain => {
                self.state = State::Sustain;
                self.set_target(self.sustain, Normal::maximum(), false, false);
            }
            State::Release => {
                if self.release == Normal::minimum() {
                    self.set_explicit_amplitude(Normal::maximum());
                    self.set_state(State::Idle);
                } else {
                    self.state = State::Release;
                    let target_amplitude = 0.0;
                    self.set_target(Normal::minimum(), self.release, true, false);
                    let current_amplitude = self.uncorrected_amplitude.sum();
                    (self.concave_a, self.concave_b, self.concave_c) = Self::calculate_coefficients(
                        current_amplitude,
                        current_amplitude,
                        (current_amplitude - target_amplitude) / 2.0 + target_amplitude,
                        (current_amplitude - target_amplitude) / 3.0 + target_amplitude,
                        target_amplitude,
                        target_amplitude,
                    );
                }
            }
            State::Shutdown => {
                self.state = State::Shutdown;
                self.set_target(
                    Normal::minimum(),
                    Envelope::from_seconds_to_normal(Seconds(1.0 / 1000.0)),
                    false,
                    true,
                );
            }
        }
    }

    fn set_explicit_amplitude(&mut self, amplitude: Normal) {
        self.uncorrected_amplitude = KahanSum::new_with_value(amplitude.0);
        self.amplitude_was_set = true;
    }

    fn set_target(
        &mut self,
        target_amplitude: Normal,
        duration: Normal,
        calculate_for_full_amplitude_range: bool,
        fast_reaction: bool,
    ) {
        self.amplitude_target = target_amplitude.into();
        if duration != Normal::maximum() {
            let fast_reaction_extra_frame = if fast_reaction { 1.0 } else { 0.0 };
            let range = if calculate_for_full_amplitude_range {
                -1.0
            } else {
                self.amplitude_target - self.uncorrected_amplitude.sum()
            };
            let duration_seconds = Self::from_normal_to_seconds(duration);
            self.time_target = self.time + duration_seconds;
            self.delta = if duration != Normal::minimum() {
                range / (duration_seconds.0 * self.sample_rate.0 as f64 + fast_reaction_extra_frame)
            } else {
                0.0
            };
            if fast_reaction {
                self.uncorrected_amplitude += self.delta;
            }
        } else {
            self.time_target = Seconds::infinite();
            self.delta = 0.0;
        }
    }

    fn calculate_coefficients(
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    ) -> (f64, f64, f64) {
        if x0 == x1 && x1 == x2 && y0 == y1 && y1 == y2 {
            // The curve we're asking about is actually just a point. Return an
            // identity.
            return (0.0, 1.0, 0.0);
        }
        let m = Matrix3::new(
            1.0,
            x0,
            x0.powi(2),
            1.0,
            x1,
            x1.powi(2),
            1.0,
            x2,
            x2.powi(2),
        );
        let y = Matrix3x1::new(y0, y1, y2);
        let r = m.try_inverse();
        if let Some(r) = r {
            let abc = r * y;
            (abc[0], abc[1], abc[2])
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    fn transform_linear_to_convex(&self, linear_value: f64) -> f64 {
        self.convex_c * linear_value.powi(2) + self.convex_b * linear_value + self.convex_a
    }
    fn transform_linear_to_concave(&self, linear_value: f64) -> f64 {
        self.concave_c * linear_value.powi(2) + self.concave_b * linear_value + self.concave_a
    }

    pub fn attack(&self) -> Normal {
        self.attack
    }

    pub fn decay(&self) -> Normal {
        self.decay
    }

    pub fn sustain(&self) -> Normal {
        self.sustain
    }

    pub fn release(&self) -> Normal {
        self.release
    }

    pub fn set_attack(&mut self, attack: Normal) {
        self.attack = attack;
    }

    pub fn set_decay(&mut self, decay: Normal) {
        self.decay = decay;
    }

    pub fn set_sustain(&mut self, sustain: Normal) {
        self.sustain = sustain;
    }

    pub fn set_release(&mut self, release: Normal) {
        self.release = release;
    }

    // TODO: experimental, not sure if this is the right pattern. It is
    // basically a from_params() that's meant to allow changes without
    // disrupting everything, which probably means it won't be the kind of thing
    // a macro can generate.
    pub fn update_from_params(&mut self, params: &EnvelopeParams) {
        self.set_attack(params.attack());
        self.set_decay(params.decay());
        self.set_sustain(params.sustain());
        self.set_release(params.release());
    }

    /// The current value of the envelope generator. Note that this value is
    /// often not the one you want if you really care about getting the
    /// amplitude at specific interesting time points in the envelope's
    /// lifecycle. If you call it before the current time slice's tick(), then
    /// you get the value before any pending events (which is probably bad), and
    /// if you call it after the tick(), then you get the value for the *next*
    /// time slice (which is probably bad). It's better to use the value
    /// returned by tick(), which is in between pending events but after
    /// updating for the time slice.
    #[allow(dead_code)]
    fn debug_amplitude(&self) -> Normal {
        Normal::new(self.uncorrected_amplitude.sum())
    }

    fn debug_state(&self) -> &State {
        &self.state
    }

    pub(crate) fn debug_is_shutting_down(&self) -> bool {
        matches!(self.debug_state(), State::Shutdown)
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> eframe::egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 64.0), Sense::hover());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let control_point_radius = 8.0;

        let x_max = response.rect.size().x;
        let y_max = response.rect.size().y;

        let attack_x_scaled = self.attack.0 as f32 * x_max / 4.0;
        let decay_x_scaled = self.decay.0 as f32 * x_max / 4.0;
        let sustain_y_scaled = (1.0 - self.sustain.0 as f32) * y_max;
        let release_x_scaled = self.release.0 as f32 * x_max / 4.0;
        let mut control_points = [
            pos2(attack_x_scaled, 0.0),
            pos2(attack_x_scaled + decay_x_scaled, sustain_y_scaled),
            pos2(
                attack_x_scaled
                    + decay_x_scaled
                    + (x_max - (attack_x_scaled + decay_x_scaled + release_x_scaled)) / 2.0,
                sustain_y_scaled,
            ),
            pos2(x_max - release_x_scaled, sustain_y_scaled),
        ];

        let mut which_changed = usize::MAX;
        let control_point_shapes: Vec<Shape> = control_points
            .iter_mut()
            .enumerate()
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * control_point_radius);

                let point_in_screen = to_screen.transform_pos(*point);
                let point_rect = Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());
                if point_response.drag_delta() != Vec2::ZERO {
                    which_changed = i;
                }

                // Restrict change to only the dimension we care about, so
                // it looks less janky.
                let mut drag_delta = point_response.drag_delta();
                match which_changed {
                    0 => drag_delta.y = 0.0,
                    1 => drag_delta.y = 0.0,
                    2 => drag_delta.x = 0.0,
                    3 => drag_delta.y = 0.0,
                    usize::MAX => {}
                    _ => unreachable!(),
                }

                *point += drag_delta;
                *point = to_screen.from().clamp(*point);

                let point_in_screen = to_screen.transform_pos(*point);
                let stroke = ui.style().interact(&point_response).fg_stroke;

                Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
            })
            .collect();

        if which_changed != usize::MAX {
            match which_changed {
                0 => {
                    self.set_attack((control_points[0].x / (x_max / 4.0)).into());
                }
                1 => {
                    self.set_decay(
                        ((control_points[1].x - control_points[0].x) / (x_max / 4.0)).into(),
                    );
                }
                2 => {
                    self.set_sustain((1.0 - control_points[2].y / y_max).into());
                }
                3 => {
                    self.set_release(((x_max - control_points[3].x) / (x_max / 4.0)).into());
                }
                _ => unreachable!(),
            }
        }

        let control_points = [
            pos2(0.0, y_max),
            control_points[0],
            control_points[1],
            control_points[2],
            control_points[3],
            pos2(x_max, y_max),
        ];
        let points_in_screen: Vec<Pos2> = control_points.iter().map(|p| to_screen * *p).collect();

        painter.add(PathShape::line(
            points_in_screen,
            Stroke {
                width: 2.0,
                color: Color32::YELLOW,
            },
        ));
        painter.extend(control_point_shapes);

        if which_changed != usize::MAX {
            response.mark_changed();
        }
        response
    }
}
impl Displays for Envelope {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(envelope(self))
    }
}

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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum SteppedEnvelopeFunction {
    #[default]
    Linear,
    Logarithmic,
    Exponential,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SteppedEnvelopeStep {
    pub interval: std::ops::Range<SignalType>,
    pub start_value: SignalType,
    pub end_value: SignalType,
    pub step_function: SteppedEnvelopeFunction,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SteppedEnvelope {
    steps: Vec<SteppedEnvelopeStep>,
}
impl SteppedEnvelope {
    const EMPTY_STEP: SteppedEnvelopeStep = SteppedEnvelopeStep {
        interval: std::ops::Range {
            start: 0.0,
            end: 0.0,
        },
        start_value: 0.0,
        end_value: 0.0,
        step_function: SteppedEnvelopeFunction::Linear,
    };

    pub fn push_step(&mut self, step: SteppedEnvelopeStep) {
        self.steps.push(step);

        // self.debug_validate_steps();
    }

    fn steps(&self) -> &[SteppedEnvelopeStep] {
        &self.steps
    }

    pub fn step_for_time(&self, time: f64) -> &SteppedEnvelopeStep {
        let steps = self.steps();
        if steps.is_empty() {
            return &Self::EMPTY_STEP;
        }

        let mut candidate_step: &SteppedEnvelopeStep = steps.first().unwrap();
        for step in steps {
            if candidate_step.interval.end == f64::MAX {
                // Any step with max end_time is terminal.
                break;
            }
            debug_assert!(step.interval.start >= candidate_step.interval.start);
            debug_assert!(step.interval.end >= candidate_step.interval.start);

            if step.interval.start > time {
                // This step starts in the future. If all steps' start times
                // are in order, then we can't do better than what we have.
                break;
            }
            if step.interval.end < time {
                // This step already ended. It's invalid for this point in time.
                continue;
            }
            candidate_step = step;
        }
        candidate_step
    }

    pub fn value_for_step_at_time(&self, step: &SteppedEnvelopeStep, time: f64) -> SignalType {
        if step.interval.start == step.interval.end || step.start_value == step.end_value {
            return step.end_value;
        }
        let elapsed_time = time - step.interval.start;
        let total_interval_time = step.interval.end - step.interval.start;
        let percentage_complete = elapsed_time / total_interval_time;
        let total_interval_value_delta = step.end_value - step.start_value;

        let multiplier = if percentage_complete == 0.0 {
            0.0
        } else {
            match step.step_function {
                SteppedEnvelopeFunction::Linear => percentage_complete,
                SteppedEnvelopeFunction::Logarithmic => {
                    (percentage_complete.log(10000.0) * 2.0 + 1.0).clamp(0.0, 1.0)
                }
                SteppedEnvelopeFunction::Exponential => 100.0f64.powf(percentage_complete) / 100.0,
            }
        };
        let mut value = step.start_value + total_interval_value_delta * multiplier;
        if (step.end_value > step.start_value && value > step.end_value)
            || (step.end_value < step.start_value && value < step.end_value)
        {
            value = step.end_value;
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{midi::MidiNote, time::Transport};
    use float_cmp::approx_eq;
    use more_asserts::{assert_gt, assert_lt};
    use std::{env::current_dir, fs, path::PathBuf};

    const SAMPLE_BUFFER_SIZE: usize = 64;

    pub trait DebugTicks: Ticks {
        fn debug_tick_until(&mut self, tick_number: usize);
    }

    impl DebugTicks for Oscillator {
        fn debug_tick_until(&mut self, tick_number: usize) {
            if self.ticks < tick_number {
                self.tick(tick_number - self.ticks);
            }
        }
    }

    impl SteppedEnvelope {
        #[allow(dead_code)]
        #[allow(unused_variables)]
        fn debug_validate_steps(&self) {
            debug_assert!(!self.steps.is_empty());
            debug_assert_eq!(self.steps.first().unwrap().interval.start, 0.0);
            // TODO: this should be optional depending on who's using it ..... debug_assert_eq!(self.steps.last().unwrap().interval.end, f32::MAX);
            let mut start_time = 0.0;
            let mut end_time = 0.0;
            let steps = self.steps();
            for step in steps {
                // debug_assert_le!(step.interval.start, step.interval.end); // Next step has non-negative duration
                // debug_assert_ge!(step.interval.start, start_time); // We're not moving backward in time
                // debug_assert_le!(step.interval.start, end_time); // Next step leaves no gaps (overlaps OK)
                start_time = step.interval.start;
                end_time = step.interval.end;

                // We don't require subsequent steps to be valid, as long as
                // an earlier step covered the rest of the time range.
                if step.interval.end == f64::MAX {
                    break;
                }
            }
            // TODO same debug_assert_eq!(end_time, f32::MAX);
        }
    }

    fn create_oscillator(waveform: Waveform, tune: Ratio, note: MidiNote) -> Oscillator {
        let mut oscillator = Oscillator::new_with(&OscillatorParams {
            waveform,
            frequency: FrequencyHz::from(note),
            ..Default::default()
        });
        oscillator.set_frequency_tune(tune);
        oscillator
    }

    #[test]
    fn oscillator_pola() {
        let mut oscillator =
            Oscillator::new_with(&OscillatorParams::default_with_waveform(Waveform::Sine));

        // we'll get a few samples in case the oscillator happens to start at
        // zero
        let mut values = [BipolarNormal::default(); 3];
        oscillator.generate_batch_values(&mut values);
        assert_ne!(0.0, values[1].0, "Default Oscillator should not be silent");
    }

    // Make sure we're dealing with at least a pulse-width wave of amplitude
    // 1.0, which means that every value is either 1.0 or -1.0.
    #[test]
    fn square_wave_is_correct_amplitude() {
        const SAMPLE_RATE: SampleRate = SampleRate::new(63949); // Prime number
        const FREQUENCY: FrequencyHz = FrequencyHz(499.0);
        let mut oscillator = Oscillator::new_with(&OscillatorParams {
            waveform: Waveform::Square,
            frequency: FREQUENCY,
            ..Default::default()
        });
        oscillator.update_sample_rate(SAMPLE_RATE);

        // Below Nyquist limit
        assert_lt!(FREQUENCY, FrequencyHz((SAMPLE_RATE.0 / 2) as f64));

        for _ in 0..SAMPLE_RATE.0 {
            oscillator.tick(1);
            let f = oscillator.value().0;
            assert_eq!(f, f.signum());
        }
    }

    #[test]
    fn square_wave_frequency_is_accurate() {
        // For this test, we want the sample rate and frequency to be nice even
        // numbers so that we don't have to deal with edge cases.
        const SAMPLE_RATE: SampleRate = SampleRate::new(65536);
        const FREQUENCY: FrequencyHz = FrequencyHz(128.0);
        let mut oscillator = Oscillator::new_with(&OscillatorParams {
            waveform: Waveform::Square,
            frequency: FREQUENCY,
            ..Default::default()
        });
        oscillator.update_sample_rate(SAMPLE_RATE);

        let mut n_pos = 0;
        let mut n_neg = 0;
        let mut last_sample = 1.0;
        let mut transitions = 0;
        for _ in 0..SAMPLE_RATE.0 {
            oscillator.tick(1);
            let f = oscillator.value().0;
            if f == 1.0 {
                n_pos += 1;
            } else if f == -1.0 {
                n_neg += 1;
            } else {
                panic!("square wave emitted strange amplitude: {f}");
            }
            if f != last_sample {
                transitions += 1;
                last_sample = f;
            }
        }
        assert_eq!(n_pos + n_neg, SAMPLE_RATE.0);
        assert_eq!(n_pos, n_neg);

        // The -1 is because we stop at the end of the cycle, and the transition
        // back to 1.0 should be at the start of the next cycle.
        assert_eq!(transitions, FREQUENCY.0 as i32 * 2 - 1);
    }

    #[test]
    fn square_wave_shape_is_accurate() {
        const SAMPLE_RATE: SampleRate = SampleRate::new(65536);
        const FREQUENCY: FrequencyHz = FrequencyHz(2.0);
        let mut oscillator = Oscillator::new_with(&OscillatorParams {
            waveform: Waveform::Square,
            frequency: FREQUENCY,
            ..Default::default()
        });
        oscillator.update_sample_rate(SAMPLE_RATE);

        oscillator.tick(1);
        assert_eq!(
            oscillator.value().0,
            1.0,
            "the first sample of a square wave should be 1.0"
        );

        // Halfway between the first and second cycle, the wave should
        // transition from 1.0 to -1.0.
        //
        // We're fast-forwarding two different ways in this test. The first is
        // by just ticking the clock the desired number of times, so we're not
        // really fast-forwarding; we're just playing normally and ignoring the
        // results. The second is by testing that the oscillator responds
        // reasonably to clock.set_samples(). I haven't decided whether entities
        // need to pay close attention to clock.set_samples() other than not
        // exploding, so I might end up deleting that part of the test.
        oscillator.tick(SAMPLE_RATE.0 / 4 - 2);
        assert_eq!(oscillator.value().0, 1.0);
        oscillator.tick(1);
        assert_eq!(oscillator.value().0, 1.0);
        oscillator.tick(1);
        assert_eq!(oscillator.value().0, -1.0);
        oscillator.tick(1);
        assert_eq!(oscillator.value().0, -1.0);

        // Then should transition back to 1.0 at the first sample of the second
        // cycle.
        //
        // As noted above, we're using clock.set_samples() here.
        oscillator.debug_tick_until(SAMPLE_RATE.0 / 2 - 2);
        assert_eq!(oscillator.value().0, -1.0);
        oscillator.tick(1);
        assert_eq!(oscillator.value().0, -1.0);
        oscillator.tick(1);
        assert_eq!(oscillator.value().0, 1.0);
        oscillator.tick(1);
        assert_eq!(oscillator.value().0, 1.0);
    }

    #[test]
    fn sine_wave_is_balanced() {
        const FREQUENCY: FrequencyHz = FrequencyHz(1.0);
        let mut oscillator = Oscillator::new_with(&OscillatorParams {
            waveform: Waveform::Sine,
            frequency: FREQUENCY,
            ..Default::default()
        });
        oscillator.update_sample_rate(SampleRate::DEFAULT);

        let mut n_pos = 0;
        let mut n_neg = 0;
        let mut n_zero = 0;
        for _ in 0..SampleRate::DEFAULT_SAMPLE_RATE {
            oscillator.tick(1);
            let f = oscillator.value().0;
            if f < -0.0000001 {
                n_neg += 1;
            } else if f > 0.0000001 {
                n_pos += 1;
            } else {
                n_zero += 1;
            }
        }
        assert_eq!(n_zero, 2);
        assert_eq!(n_pos, n_neg);
        assert_eq!(n_pos + n_neg + n_zero, SampleRate::DEFAULT_SAMPLE_RATE);
    }

    // For now, only Oscillator implements source_signal(). We'll probably make
    // it a trait later.
    pub fn render_signal_as_audio_source(
        source: &mut Oscillator,
        run_length_in_seconds: usize,
    ) -> Vec<Sample> {
        let mut samples = Vec::default();
        for _ in 0..SampleRate::DEFAULT_SAMPLE_RATE * run_length_in_seconds {
            source.tick(1);
            samples.push(Sample::from(source.value().0));
        }
        samples
    }

    fn read_samples_from_mono_wav_file(filename: &PathBuf) -> Vec<Sample> {
        let mut reader = hound::WavReader::open(filename).unwrap();
        let mut r = Vec::default();

        for sample in reader.samples::<i16>() {
            r.push(Sample::from(
                sample.unwrap() as SampleType / i16::MAX as SampleType,
            ));
        }
        r
    }

    pub fn samples_match_known_good_wav_file(
        samples: Vec<Sample>,
        filename: &PathBuf,
        acceptable_deviation: SampleType,
    ) -> bool {
        let known_good_samples = read_samples_from_mono_wav_file(filename);
        if known_good_samples.len() != samples.len() {
            eprintln!("Provided samples of different length from known-good");
            return false;
        }
        for i in 0..samples.len() {
            if (samples[i] - known_good_samples[i]).0.abs() >= acceptable_deviation {
                eprintln!(
                    "Samples differed at position {i}: known-good {}, test {}",
                    known_good_samples[i].0, samples[i].0
                );
                return false;
            }
        }
        true
    }

    #[test]
    fn square_matches_known_good() {
        let test_cases = vec![
            (1.0, "1Hz"),
            (100.0, "100Hz"),
            (1000.0, "1000Hz"),
            (10000.0, "10000Hz"),
            (20000.0, "20000Hz"),
        ];
        for test_case in test_cases {
            let mut osc = Oscillator::new_with(&OscillatorParams {
                waveform: Waveform::Square,
                frequency: test_case.0.into(),
                ..Default::default()
            });
            let samples = render_signal_as_audio_source(&mut osc, 1);
            let mut filename = TestOnlyPaths::data_path();
            filename.push("audacity");
            filename.push("44100Hz-mono");
            filename.push(format!("square-{}.wav", test_case.1));

            assert!(
                samples_match_known_good_wav_file(samples, &filename, 0.001),
                "while testing square {}Hz",
                test_case.0
            );
        }
    }

    fn get_test_cases() -> Vec<(FrequencyHz, &'static str)> {
        vec![
            (FrequencyHz(1.0), "1Hz"),
            (FrequencyHz(100.0), "100Hz"),
            (FrequencyHz(1000.0), "1000Hz"),
            (FrequencyHz(10000.0), "10000Hz"),
            (FrequencyHz(20000.0), "20000Hz"),
        ]
    }

    #[test]
    fn sine_matches_known_good() {
        for test_case in get_test_cases() {
            let mut osc = Oscillator::new_with(&OscillatorParams {
                waveform: Waveform::Sine,
                frequency: test_case.0.into(),
                ..Default::default()
            });
            let samples = render_signal_as_audio_source(&mut osc, 1);
            let mut filename = TestOnlyPaths::data_path();
            filename.push("audacity");
            filename.push("44100Hz-mono");
            filename.push(format!("sine-{}.wav", test_case.1));

            assert!(
                samples_match_known_good_wav_file(samples, &filename, 0.001),
                "while testing sine {}Hz",
                test_case.0
            );
        }
    }

    #[test]
    fn sawtooth_matches_known_good() {
        for test_case in get_test_cases() {
            let mut osc = Oscillator::new_with(&OscillatorParams {
                waveform: Waveform::Sawtooth,
                frequency: test_case.0.into(),
                ..Default::default()
            });
            let samples = render_signal_as_audio_source(&mut osc, 1);
            let mut filename = TestOnlyPaths::data_path();
            filename.push("audacity");
            filename.push("44100Hz-mono");
            filename.push(format!("sawtooth-{}.wav", test_case.1));

            assert!(
                samples_match_known_good_wav_file(samples, &filename, 0.001),
                "while testing sawtooth {}Hz",
                test_case.0
            );
        }
    }

    #[test]
    fn triangle_matches_known_good() {
        for test_case in get_test_cases() {
            let mut osc = Oscillator::new_with(&OscillatorParams {
                waveform: Waveform::Triangle,
                frequency: test_case.0.into(),
                ..Default::default()
            });
            let samples = render_signal_as_audio_source(&mut osc, 1);
            let mut filename = TestOnlyPaths::data_path();
            filename.push("audacity");
            filename.push("44100Hz-mono");
            filename.push(format!("triangle-{}.wav", test_case.1));

            assert!(
                samples_match_known_good_wav_file(samples, &filename, 0.01),
                "while testing triangle {}Hz",
                test_case.0
            );
        }
    }

    #[test]
    fn oscillator_modulated() {
        let mut oscillator = create_oscillator(Waveform::Sine, Ratio::from(1.0), MidiNote::C4);
        // Default
        assert_eq!(
            oscillator.adjusted_frequency(),
            FrequencyHz::from(MidiNote::C4)
        );

        // Explicitly zero (none)
        oscillator.set_frequency_modulation(BipolarNormal::from(0.0));
        assert_eq!(
            oscillator.adjusted_frequency(),
            FrequencyHz::from(MidiNote::C4)
        );

        // Max
        oscillator.set_frequency_modulation(BipolarNormal::from(1.0));
        assert_eq!(
            oscillator.adjusted_frequency(),
            FrequencyHz::from(MidiNote::C5)
        );

        // Min
        oscillator.set_frequency_modulation(BipolarNormal::from(-1.0));
        assert_eq!(
            oscillator.adjusted_frequency(),
            FrequencyHz::from(MidiNote::C3)
        );

        // Halfway between zero and max
        oscillator.set_frequency_modulation(BipolarNormal::from(0.5));
        assert_eq!(
            oscillator.adjusted_frequency(),
            FrequencyHz::from(MidiNote::C4) * 2.0f64.sqrt()
        );
    }

    #[test]
    fn oscillator_cycle_restarts_on_time() {
        let mut oscillator =
            Oscillator::new_with(&OscillatorParams::default_with_waveform(Waveform::Sine));
        const FREQUENCY: FrequencyHz = FrequencyHz(2.0);
        oscillator.set_frequency(FREQUENCY);
        oscillator.update_sample_rate(SampleRate::DEFAULT);

        const TICKS_IN_CYCLE: usize = SampleRate::DEFAULT_SAMPLE_RATE / 2; // That 2 is FREQUENCY
        assert_eq!(TICKS_IN_CYCLE, 44100 / 2);

        // We assume that synced oscillators can take care of their own init.
        assert!(
            !oscillator.should_sync(),
            "On init, the oscillator should NOT flag that any init/reset work needs to happen."
        );

        // Now run through and see that we're flagging cycle start at the right
        // time. Note the = in the for loop range; we're expecting a flag at the
        // zeroth sample of each cycle.
        for tick in 0..=TICKS_IN_CYCLE {
            let expected = match tick {
                0 => true,              // zeroth sample of first cycle
                TICKS_IN_CYCLE => true, // zeroth sample of second cycle
                _ => false,
            };

            oscillator.tick(1);
            assert_eq!(
                oscillator.should_sync(),
                expected,
                "expected {expected} at sample #{tick}"
            );
        }

        // Let's try again after rewinding the clock. It should recognize
        // something happened and restart the cycle.
        oscillator.tick(1);
        assert!(
            !oscillator.should_sync(),
            "Oscillator shouldn't sync midway through cycle."
        );

        // Then we actually change the clock. We'll pick something we know is
        // off-cycle. We don't treat this as a should-sync event, because we
        // assume that synced oscillators will also notice the clock change and
        // do the right thing. At worst, we'll be off for a single main
        // oscillator cycle. No normal audio performance will involve a clock
        // shift, so it's OK to have the wrong timbre for a tiny fraction of a
        // second.
        oscillator.update_sample_rate(SampleRate::DEFAULT);
        oscillator.tick(1);
        assert!(
            oscillator.should_sync(),
            "After reset, oscillator should sync."
        );
        oscillator.tick(1);
        assert!(
            !oscillator.should_sync(),
            "Oscillator shouldn't sync twice when syncing after reset."
        );

        // Let's run through again, but this time go for a whole second, and
        // count the number of flags.
        oscillator.update_sample_rate(SampleRate::DEFAULT);
        let mut cycles = 0;
        for _ in 0..SampleRate::DEFAULT_SAMPLE_RATE {
            oscillator.tick(1);
            if oscillator.should_sync() {
                cycles += 1;
            }
        }
        assert_eq!(cycles, usize::from(FREQUENCY));
    }

    // Where possible, we'll erase the envelope type and work only with the
    // Envelope trait, so that we can confirm that the trait alone is useful.
    fn get_ge_trait_stuff() -> (Transport, impl GeneratesEnvelope) {
        let mut transport = Transport::default();
        transport.play();
        let envelope = Envelope::new_with(&EnvelopeParams::new_with(
            (0.1).into(),
            (0.2).into(),
            Normal::new(0.8),
            (0.3).into(),
        ));
        (transport, envelope)
    }

    #[test]
    fn generates_envelope_trait_idle() {
        let (mut transport, mut e) = get_ge_trait_stuff();

        assert!(e.is_idle(), "Envelope should be idle on creation.");

        e.tick(1);
        transport.advance(1);
        assert!(e.is_idle(), "Untriggered envelope should remain idle.");
        assert_eq!(
            e.value().0,
            0.0,
            "Untriggered envelope should remain amplitude zero."
        );
    }

    fn run_until<F>(
        envelope: &mut impl GeneratesEnvelope,
        transport: &mut Transport,
        time_marker: MusicalTime,
        mut test: F,
    ) -> Normal
    where
        F: FnMut(Normal, &Transport),
    {
        let mut amplitude: Normal = Normal::new(0.0);
        loop {
            envelope.tick(1);
            transport.advance(1);
            let should_continue = transport.current_time() < time_marker;
            if !should_continue {
                break;
            }
            amplitude = envelope.value();
            test(amplitude, transport);
        }
        amplitude
    }

    #[test]
    fn generates_envelope_trait_instant_trigger_response() {
        let (mut transport, mut e) = get_ge_trait_stuff();

        transport.update_sample_rate(SampleRate::DEFAULT);
        e.update_sample_rate(SampleRate::DEFAULT);

        e.trigger_attack();
        e.tick(1);
        transport.advance(1);
        assert!(
            !e.is_idle(),
            "Envelope should be active immediately upon trigger"
        );

        // We apply a small fudge factor to account for the fact that the MMA
        // convex transform rounds to zero pretty aggressively, so attacks take
        // a bit of time before they are apparent. I'm not sure whether this is
        // a good thing; it objectively makes attack laggy (in this case 16
        // samples late!).
        for _ in 0..17 {
            e.tick(1);
            transport.advance(1);
        }
        assert_gt!(
            e.value().0,
            0.0,
            "Envelope amplitude should increase immediately upon trigger"
        );
    }

    #[test]
    fn generates_envelope_trait_attack_decay_duration() {
        let mut transport = Transport::default();
        // This is an ugly way to get seconds and beats to match up. This
        // happened because these tests were written for Clock, which worked in
        // units of wall-clock time, and we migrated to MusicalTime, which is
        // based on beats.
        transport.set_tempo(Tempo(60.0));
        transport.play();

        let attack: Normal = Envelope::from_seconds_to_normal(Seconds(0.1));
        let decay: Normal = Envelope::from_seconds_to_normal(Seconds(0.2));
        const SUSTAIN: Normal = Normal::new_const(0.8);
        let release: Normal = Envelope::from_seconds_to_normal(Seconds(0.3));
        let mut envelope =
            Envelope::new_with(&EnvelopeParams::new_with(attack, decay, SUSTAIN, release));

        // An even sample rate means we can easily calculate how much time was spent in each state.
        transport.update_sample_rate(SampleRate::from(100));
        envelope.update_sample_rate(SampleRate::from(100));

        let mut time_marker = transport.current_time()
            + MusicalTime::new_with_fractional_beats(Envelope::from_normal_to_seconds(attack).0);
        envelope.trigger_attack();
        assert!(
            matches!(envelope.debug_state(), State::Attack),
            "Expected SimpleEnvelopeState::Attack after trigger, but got {:?} instead",
            envelope.debug_state()
        );
        let mut last_amplitude = envelope.value();

        envelope.tick(1);

        let amplitude = run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |amplitude, transport| {
                assert_lt!(
                    last_amplitude,
                    amplitude,
                    "Expected amplitude to rise through attack time ending at {time_marker}, but it didn't at time {}", transport.current_time().total_units()
                );
                last_amplitude = amplitude;
            },
        );
        assert!(matches!(envelope.debug_state(), State::Decay));
        assert!(
            approx_eq!(f64, amplitude.0, 1.0f64, epsilon = 0.0000000000001),
            "Amplitude should reach maximum after attack (was {}, difference {}).",
            amplitude.0,
            (1.0 - amplitude.0).abs()
        );

        time_marker +=
            MusicalTime::new_with_fractional_beats(Envelope::from_normal_to_seconds(decay).0);
        let amplitude = run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |_amplitude, _clock| {},
        );
        assert_eq!(
            amplitude, SUSTAIN,
            "Amplitude should reach sustain level after decay."
        );
        assert!(matches!(envelope.debug_state(), State::Sustain));
    }

    // Decay and release rates should be determined as if the envelope stages
    // were operating on a full 1.0..=0.0 amplitude range. Thus, the expected
    // time for the stage is not necessarily the same as the parameter.
    fn expected_decay_time(decay: Normal, sustain: Normal) -> Seconds {
        Envelope::from_normal_to_seconds(decay * (1.0 - sustain.0))
    }

    fn expected_release_time(release: Normal, current_amplitude: Normal) -> Seconds {
        Envelope::from_normal_to_seconds(release * current_amplitude)
    }

    #[test]
    fn generates_envelope_trait_sustain_duration_then_release() {
        let mut transport = Transport::default();
        transport.set_tempo(Tempo(60.0));
        transport.play();

        let attack: Normal = Envelope::from_seconds_to_normal(Seconds(0.1));
        let decay: Normal = Envelope::from_seconds_to_normal(Seconds(0.2));
        const SUSTAIN: Normal = Normal::new_const(0.8);
        let release: Normal = Envelope::from_seconds_to_normal(Seconds(0.3));
        let mut envelope =
            Envelope::new_with(&EnvelopeParams::new_with(attack, decay, SUSTAIN, release));

        envelope.trigger_attack();
        envelope.tick(1);
        let mut time_marker = transport.current_time()
            + MusicalTime::new_with_fractional_beats(
                Envelope::from_normal_to_seconds(attack).0 + expected_decay_time(decay, SUSTAIN).0,
            );
        transport.advance(1);

        // Skip past attack/decay.
        run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |_amplitude, _clock| {},
        );

        time_marker += MusicalTime::new_with_fractional_beats(0.5);
        let amplitude = run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |amplitude, _clock| {
                assert_eq!(
                    amplitude, SUSTAIN,
                    "Amplitude should remain at sustain level while note is still triggered"
                );
            },
        )
        .0;

        envelope.trigger_release();
        time_marker += MusicalTime::new_with_fractional_beats(
            expected_release_time(release, amplitude.into()).0,
        );
        let mut last_amplitude = amplitude;
        let amplitude = run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |inner_amplitude, _clock| {
                assert_lt!(
                    inner_amplitude.0,
                    last_amplitude,
                    "Amplitude should begin decreasing as soon as note off."
                );
                last_amplitude = inner_amplitude.0;
            },
        );

        // These assertions are checking the next frame's state, which is right
        // because we want to test what happens after the release ends.
        assert!(
            envelope.is_idle(),
            "Envelope should be idle when release ends, but it wasn't (amplitude is {})",
            amplitude.0
        );
        assert_eq!(
            envelope.debug_amplitude().0,
            0.0,
            "Amplitude should be zero when release ends"
        );
    }

    #[test]
    fn simple_envelope_interrupted_decay_with_second_attack() {
        let mut transport = Transport::default();
        transport.set_tempo(Tempo(60.0));
        transport.play();

        // These settings are copied from Welsh Piano's filter envelope, which
        // is where I noticed some unwanted behavior.
        let attack: Normal = Envelope::from_seconds_to_normal(Seconds(0.0));
        let decay: Normal = Envelope::from_seconds_to_normal(Seconds(5.22));
        const SUSTAIN: Normal = Normal::new_const(0.25);
        let release: Normal = Envelope::from_seconds_to_normal(Seconds(0.5));
        let mut envelope =
            Envelope::new_with(&EnvelopeParams::new_with(attack, decay, SUSTAIN, release));

        transport.update_sample_rate(SampleRate::DEFAULT);
        envelope.update_sample_rate(SampleRate::DEFAULT);

        envelope.tick(1);
        transport.advance(1);

        assert_eq!(
            envelope.value(),
            Normal::minimum(),
            "Amplitude should start at zero"
        );

        // See https://floating-point-gui.de/errors/comparison/ for standard
        // warning about comparing floats and looking for epsilons.
        envelope.trigger_attack();
        envelope.tick(1);
        let mut time_marker = transport.current_time();
        transport.advance(1);
        assert!(
            approx_eq!(f64, envelope.value().0, Normal::maximum().0, ulps = 8),
            "Amplitude should reach peak upon trigger, but instead of {} we got {}",
            Normal::maximum().0,
            envelope.value().0,
        );
        envelope.tick(1);
        transport.advance(1);
        assert_lt!(
            envelope.value(),
            Normal::maximum(),
            "Zero-attack amplitude should begin decreasing immediately after peak"
        );

        // Jump to halfway through decay.
        time_marker += MusicalTime::new_with_fractional_beats(
            Envelope::from_normal_to_seconds(attack).0
                + Envelope::from_normal_to_seconds(decay).0 / 2.0,
        );
        let amplitude = run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |_amplitude, _clock| {},
        );
        assert_lt!(
            amplitude,
            Normal::maximum(),
            "Amplitude should have decayed halfway through decay"
        );

        // Release the trigger.
        envelope.trigger_release();
        let _amplitude = envelope.tick(1);
        transport.advance(1);

        // And hit it again.
        envelope.trigger_attack();
        envelope.tick(1);
        let mut time_marker = transport.current_time();
        transport.advance(1);
        assert!(
            approx_eq!(f64, envelope.value().0, Normal::maximum().0, ulps = 8),
            "Amplitude should reach peak upon second trigger"
        );

        // Then release again.
        envelope.trigger_release();

        // Check that we keep decreasing amplitude to zero, not to sustain.
        time_marker +=
            MusicalTime::new_with_fractional_beats(Envelope::from_normal_to_seconds(release).0);
        let mut last_amplitude = envelope.value().0;
        let _amplitude = run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |inner_amplitude, _clock| {
                assert_lt!(
                    inner_amplitude.0,
                    last_amplitude,
                    "Amplitude should continue decreasing after note off"
                );
                last_amplitude = inner_amplitude.0;
            },
        );

        // These assertions are checking the next frame's state, which is right
        // because we want to test what happens after the release ends.
        assert!(
            envelope.is_idle(),
            "Envelope should be idle when release ends"
        );
        assert_eq!(
            envelope.debug_amplitude().0,
            0.0,
            "Amplitude should be zero when release ends"
        );
    }

    // Per Pirkle, DSSPC++, p.87-88, decay and release times determine the
    // *slope* but not necessarily the *duration* of those phases of the
    // envelope. The slope assumes the specified time across a full 1.0-to-0.0
    // range. This means that the actual decay and release times for a given
    // envelope can be shorter than its parameters might suggest.
    #[test]
    fn generates_envelope_trait_decay_and_release_based_on_full_amplitude_range() {
        let mut transport = Transport::default();
        transport.set_tempo(Tempo(60.0));
        transport.play();
        const ATTACK: Normal = Normal::minimum();
        let decay: Normal = Envelope::from_seconds_to_normal(Seconds(0.8));
        let sustain = Normal::new_const(0.5);
        let release: Normal = Envelope::from_seconds_to_normal(Seconds(0.4));
        let mut envelope =
            Envelope::new_with(&EnvelopeParams::new_with(ATTACK, decay, sustain, release));

        transport.update_sample_rate(SampleRate::DEFAULT);
        envelope.update_sample_rate(SampleRate::DEFAULT);

        // Decay after note-on should be shorter than the decay value.
        envelope.trigger_attack();
        let mut time_marker = transport.current_time()
            + MusicalTime::new_with_fractional_beats(expected_decay_time(decay, sustain).0);
        let amplitude = run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |_amplitude, _clock| {},
        )
        .0;
        assert!(approx_eq!(f64, amplitude, sustain.0, epsilon=0.0001),
            "Expected to see sustain level {} instead of {} at time {} (which is {:.1}% of decay time {}, based on full 1.0..=0.0 amplitude range)",
            sustain.0,
            amplitude,
            time_marker,
            decay,
            100.0 * (1.0 - sustain.0)
        );

        // Release after note-off should also be shorter than the release value.
        envelope.trigger_release();
        let expected_release_time = expected_release_time(release, envelope.value().into());
        time_marker +=
            MusicalTime::new_with_fractional_beats(expected_release_time.0 - 0.000000000000001); // I AM SICK OF FP PRECISION ERRORS
        let amplitude = run_until(
            &mut envelope,
            &mut transport,
            time_marker,
            |inner_amplitude, transport| {
                assert_gt!(
                    inner_amplitude.0,
                    0.0,
                    "We should not reach idle before time {}, but we did at time {}.",
                    &time_marker,
                    transport.current_time()
                )
            },
        );
        let portion_of_full_amplitude_range = sustain.0;
        assert!(
            envelope.is_idle(),
            "Expected release to end after time {}, which is {:.1}% of release time {}. Amplitude is {}",
            expected_release_time.0,
            100.0 * portion_of_full_amplitude_range,
            release,
            amplitude.0
        );
    }

    // https://docs.google.com/spreadsheets/d/1DSkut7rLG04Qx_zOy3cfI7PMRoGJVr9eaP5sDrFfppQ/edit#gid=0
    #[test]
    fn coeff() {
        let (a, b, c) = Envelope::calculate_coefficients(0.0, 1.0, 0.5, 0.25, 1.0, 0.0);
        assert_eq!(a, 1.0);
        assert_eq!(b, -2.0);
        assert_eq!(c, 1.0);
    }

    #[test]
    fn envelope_amplitude_batching() {
        let mut e = Envelope::new_with(&EnvelopeParams::new_with(
            Envelope::from_seconds_to_normal(Seconds(0.1)),
            Envelope::from_seconds_to_normal(Seconds(0.2)),
            Normal::new(0.5),
            Envelope::from_seconds_to_normal(Seconds(0.3)),
        ));

        // Initialize the buffer with a nonsense value so we know it got
        // overwritten by the method we're about to call.
        //
        // TODO: that buffer size should be pulled from somewhere centralized.
        let mut amplitudes = [Normal::from(0.888); SAMPLE_BUFFER_SIZE];

        // The envelope starts out in the idle state, and we haven't triggered
        // it.
        e.generate_batch_values(&mut amplitudes);
        amplitudes.iter().for_each(|i| {
            assert_eq!(
                i.0,
                Normal::MIN,
                "Each value in untriggered EG's buffer should be set to silence"
            );
        });

        // Now trigger the envelope and see what happened.
        e.trigger_attack();
        e.generate_batch_values(&mut amplitudes);
        assert!(
            amplitudes.iter().any(|i| { i.0 != Normal::MIN }),
            "Once triggered, the EG should generate non-silent values"
        );
    }

    #[test]
    fn envelope_shutdown_state() {
        let mut e = Envelope::new_with(&EnvelopeParams::new_with(
            Normal::minimum(),
            Normal::minimum(),
            Normal::maximum(),
            Envelope::from_seconds_to_normal(Seconds(0.5)),
        ));
        e.update_sample_rate(SampleRate::from(2000));

        // With sample rate 1000, each sample is 0.5 millisecond.
        let mut amplitudes: [Normal; 10] = [Normal::default(); 10];

        e.trigger_attack();
        e.generate_batch_values(&mut amplitudes);
        assert!(
            amplitudes.iter().all(|s| { s.0 == Normal::MAX }),
            "After enqueueing attack, amplitude should be max"
        );

        e.trigger_shutdown();
        e.generate_batch_values(&mut amplitudes);
        assert_lt!(
            amplitudes[0].0,
            (Normal::MAX - Normal::MIN) / 2.0,
            "At sample rate {}, shutdown state should take two samples to go from 1.0 to 0.0, but when we checked it's {}.",
            e.sample_rate, amplitudes[0].0
        );
        assert_eq!(
            amplitudes[1].0,
            Normal::MIN,
            "At sample rate {}, shutdown state should reach 0.0 within two samples.",
            e.sample_rate
        );
    }

    // Bugfix: if sustain was 100%, attack was zero, and decay was nonzero, then
    // the decay curve called for a change from amplitude 1.0 to amplitude 1.0,
    // which meant we asked the matrix math to calculate coefficients for a
    // singularity, which netted out to amplitude being zero while we waited for
    // it to reach 1.0 (or for the decay timeout to fire, which was how we
    // progressed at all to sustain). Solution: notice that start/end
    // coordinates are identical, and return identity coefficients so that the
    // conversion from linear to curved produced the target amplitude, causing
    // the state to advance to sustain. Amazing that I didn't catch this right
    // away.
    #[test]
    fn sustain_full() {
        let mut e = Envelope::new_with(&EnvelopeParams::new_with(
            Normal::minimum(),
            Envelope::from_seconds_to_normal(Seconds(0.67)),
            Normal::maximum(),
            Envelope::from_seconds_to_normal(Seconds(0.5)),
        ));
        e.update_sample_rate(SampleRate::from(44100));
        assert_eq!(e.value().0, 0.0);
        e.tick(1);
        assert_eq!(e.value().0, 0.0);

        e.trigger_attack();
        e.tick(1);
        assert_eq!(e.value(), Normal::maximum());
    }

    impl SteppedEnvelopeStep {
        pub(crate) fn new_with_duration(
            start_time: f64,
            duration: f64,
            start_value: SignalType,
            end_value: SignalType,
            step_function: SteppedEnvelopeFunction,
        ) -> Self {
            Self {
                interval: std::ops::Range {
                    start: start_time,
                    end: if duration == f64::MAX {
                        duration
                    } else {
                        start_time + duration
                    },
                },
                start_value,
                end_value,
                step_function,
            }
        }
    }

    #[test]
    fn envelope_step_functions() {
        const START_TIME: f64 = 3.14159;
        const DURATION: f64 = 2.71828;
        const START_VALUE: SignalType = 1.0;
        const END_VALUE: SignalType = 1.0 + 10.0;

        let mut envelope = SteppedEnvelope::default();
        // This envelope is here just to offset the one we're testing,
        // to catch bugs where we assumed the start time was 0.0.
        envelope.push_step(SteppedEnvelopeStep::new_with_duration(
            0.0,
            START_TIME,
            0.0,
            0.0,
            SteppedEnvelopeFunction::Linear,
        ));
        envelope.push_step(SteppedEnvelopeStep::new_with_duration(
            START_TIME,
            DURATION,
            START_VALUE,
            END_VALUE,
            SteppedEnvelopeFunction::Linear,
        ));

        // We're lazy and ask for the step only once because we know there's only one.
        let step = envelope.step_for_time(START_TIME);
        assert_eq!(
            envelope.value_for_step_at_time(step, START_TIME),
            START_VALUE
        );
        assert_eq!(
            envelope.value_for_step_at_time(step, START_TIME + DURATION / 2.0),
            1.0 + 10.0 / 2.0
        );
        assert_eq!(
            envelope.value_for_step_at_time(step, START_TIME + DURATION),
            END_VALUE
        );

        let mut envelope = SteppedEnvelope::default();
        envelope.push_step(SteppedEnvelopeStep::new_with_duration(
            0.0,
            START_TIME,
            0.0,
            0.0,
            SteppedEnvelopeFunction::Linear,
        ));
        envelope.push_step(SteppedEnvelopeStep::new_with_duration(
            START_TIME,
            DURATION,
            START_VALUE,
            END_VALUE,
            SteppedEnvelopeFunction::Logarithmic,
        ));

        let step = envelope.step_for_time(START_TIME);
        assert_eq!(
            envelope.value_for_step_at_time(step, START_TIME),
            START_VALUE
        ); // special case log(0) == 0.0
        assert!(approx_eq!(
            f64,
            envelope.value_for_step_at_time(step, START_TIME + DURATION / 2.0),
            1.0 + 8.49485,
            epsilon = 0.001
        )); // log(0.5, 10000) corrected for (0.0..=1.0)
        assert_eq!(
            envelope.value_for_step_at_time(step, START_TIME + DURATION),
            END_VALUE
        );

        let mut envelope = SteppedEnvelope::default();
        envelope.push_step(SteppedEnvelopeStep::new_with_duration(
            0.0,
            START_TIME,
            0.0,
            0.0,
            SteppedEnvelopeFunction::Linear,
        ));
        envelope.push_step(SteppedEnvelopeStep::new_with_duration(
            START_TIME,
            DURATION,
            START_VALUE,
            END_VALUE,
            SteppedEnvelopeFunction::Exponential,
        ));

        let step = envelope.step_for_time(START_TIME);
        assert_eq!(
            envelope.value_for_step_at_time(step, START_TIME),
            START_VALUE
        );
        assert_eq!(
            envelope.value_for_step_at_time(step, START_TIME + DURATION / 2.0),
            1.0 + 10.0 * 0.1
        );
        assert_eq!(
            envelope.value_for_step_at_time(step, START_TIME + DURATION),
            END_VALUE
        );
    }

    /// Since this struct is in a crate that many other crates use, we can't
    /// protect it with a #[cfg(test)]. But we do put it in the tests module, so
    /// that it'll look strange if anyone tries using it in a non-test
    /// configuration.
    pub struct TestOnlyPaths;

    impl TestOnlyPaths {
        pub fn cwd() -> PathBuf {
            PathBuf::from(
                current_dir()
                    .ok()
                    .map(PathBuf::into_os_string)
                    .and_then(|exe| exe.into_string().ok())
                    .unwrap(),
            )
        }

        pub fn data_path() -> PathBuf {
            const TEST_DATA: &'static str = "test-data";
            let mut path_buf = Self::cwd();
            path_buf.push(TEST_DATA);
            path_buf
        }

        /// Returns a [PathBuf] representing the target/ build directory, creating
        /// it if necessary.
        pub fn writable_out_path() -> PathBuf {
            const OUT_DATA: &'static str = "target";
            let mut path_buf = Self::cwd();
            path_buf.push(OUT_DATA);
            if fs::create_dir_all(&path_buf).is_ok() {
                path_buf
            } else {
                panic!(
                    "Could not create output directory {:?} for writing",
                    &path_buf
                );
            }
        }
    }
}
