// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{prelude::*, traits::prelude::*};
use ensnare_proc_macros::{Control, Params};
use kahan::KahanSum;
use more_asserts::{debug_assert_ge, debug_assert_le};
use nalgebra::{Matrix3, Matrix3x1};
use serde::{Deserialize, Serialize};
use std::{f64::consts::PI, fmt::Debug, ops::Range};
use strum::EnumCount;
use strum::IntoEnumIterator;
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
    waveform: Waveform,

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
                2.0f64.powf(self.frequency_modulation.value()) + self.linear_frequency_modulation,
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
            self.delta =
                (self.adjusted_frequency() / FrequencyHz::from(self.sample_rate.value())).0;

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
            Waveform::PulseWidth(duty_cycle) => -(cycle_position - duty_cycle.value()).signum(),
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
