// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use derivative::Derivative;
use eframe::emath::Numeric;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Neg, RangeInclusive, Sub},
};
use synonym::Synonym;

/// [SampleType] is the underlying primitive that makes up [StereoSample].
pub type SampleType = f64;

/// [SignalType] is the primitive used for general digital signal-related work.
pub type SignalType = f64;

/// Use [ParameterType] in places where a [Normal] or [BipolarNormal] could fit,
/// except you don't have any range restrictions. Any such usage should be
/// temporary.
pub type ParameterType = f64;

/// [Sample] represents a single-channel audio sample.
#[derive(Synonym, Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Sample(pub SampleType);
impl Sample {
    /// The [SampleType] value of silence.
    pub const SILENCE_VALUE: SampleType = 0.0;
    /// A [Sample] that is silent.
    pub const SILENCE: Sample = Sample(Self::SILENCE_VALUE);
    /// The maximum positive [SampleType] value of silence.
    pub const MAX_VALUE: SampleType = 1.0;
    /// A [Sample] having the maximum positive value.
    pub const MAX: Sample = Sample(Self::MAX_VALUE);
    /// The maximum negative [SampleType] value.
    pub const MIN_VALUE: SampleType = -1.0;
    /// A [Sample] having the maximum negative value.
    pub const MIN: Sample = Sample(Self::MIN_VALUE);

    /// Converts [Sample] into an i16 scaled to i16::MIN..i16::MAX, which is
    /// slightly harder than it seems because the negative range of
    /// two's-complement numbers is larger than the positive one.
    pub fn into_i16(&self) -> i16 {
        const MAX_AMPLITUDE: SampleType = i16::MAX as SampleType;
        const MIN_AMPLITUDE: SampleType = i16::MIN as SampleType;
        let v = self.0;

        if v < 0.0 {
            (v.abs() * MIN_AMPLITUDE) as i16
        } else {
            (v * MAX_AMPLITUDE) as i16
        }
    }

    fn almost_silent(&self) -> bool {
        self.0.abs() < 0.00001
    }
}
impl AddAssign for Sample {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl Add for Sample {
    type Output = Self;

    fn add(self, rhs: Sample) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl Mul for Sample {
    type Output = Self;

    fn mul(self, rhs: Sample) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl Mul<f64> for Sample {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}
// TODO #[deprecated] because it hides evidence that migration to SampleType
// isn't complete
impl Mul<f32> for Sample {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs as f64)
    }
}
impl Div<f64> for Sample {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}
impl Sub for Sample {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl Neg for Sample {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}
impl Mul<i16> for Sample {
    type Output = Self;

    fn mul(self, rhs: i16) -> Self::Output {
        Self(self.0 * rhs as f64)
    }
}
impl Mul<Normal> for Sample {
    type Output = Self;

    fn mul(self, rhs: Normal) -> Self::Output {
        Self(self.0 * rhs.0 as f64)
    }
}
impl From<f32> for Sample {
    fn from(value: f32) -> Self {
        Sample(value as f64)
    }
}
impl From<i32> for Sample {
    // TODO: this is an incomplete conversion, because we don't know what the
    // range of the i32 really is. So we leave it to someone else to divide by
    // the correct value to obtain the proper -1.0..=1.0 range.
    fn from(value: i32) -> Self {
        Sample(value as f64)
    }
}
// I predict this conversion will someday be declared evil. We're naively
// averaging the two channels. I'm not sure this makes sense in all situations.
impl From<StereoSample> for Sample {
    fn from(value: StereoSample) -> Self {
        Sample((value.0 .0 + value.1 .0) * 0.5)
    }
}
impl From<BipolarNormal> for Sample {
    fn from(value: BipolarNormal) -> Self {
        Sample(value.0)
    }
}
impl From<Normal> for Sample {
    fn from(value: Normal) -> Self {
        let as_bipolar_normal: BipolarNormal = value.into();
        Sample::from(as_bipolar_normal)
    }
}

/// [StereoSample] is a two-channel sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct StereoSample(pub Sample, pub Sample);
impl StereoSample {
    /// Silence (0.0).
    pub const SILENCE: StereoSample = StereoSample(Sample::SILENCE, Sample::SILENCE);
    /// The loudest positive value (1.0).
    pub const MAX: StereoSample = StereoSample(Sample::MAX, Sample::MAX);
    /// The loudest negative value (-1.0).
    pub const MIN: StereoSample = StereoSample(Sample::MIN, Sample::MIN);

    /// Creates a new [StereoSample] from left and right [Sample]s.
    pub fn new(left: Sample, right: Sample) -> Self {
        Self(left, right)
    }

    // This method should be used only for testing. TODO: get rid of this. Now
    // that we're in a separate crate, we can't easily limit this to test cfg
    // only. That means it's part of the API.
    //
    // TODO: epsilon comparisons are bad. Recommend float-cmp crate instead of
    // this.
    #[allow(missing_docs)]
    pub fn almost_equals(&self, rhs: Self) -> bool {
        let epsilon = 0.0000001;
        (self.0 .0 - rhs.0 .0).abs() < epsilon && (self.1 .0 - rhs.1 .0).abs() < epsilon
    }

    /// Converts [StereoSample] into a pair of i16 scaled to i16::MIN..i16::MAX
    pub fn into_i16(&self) -> (i16, i16) {
        (self.0.into_i16(), self.1.into_i16())
    }

    pub(crate) fn almost_silent(&self) -> bool {
        self.0.almost_silent() && self.1.almost_silent()
    }
}
impl Add for StereoSample {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        StereoSample(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl AddAssign for StereoSample {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}
impl Sum for StereoSample {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(Sample::SILENCE, Sample::SILENCE), |a, b| {
            Self(a.0 + b.0, a.1 + b.1)
        })
    }
}
impl From<Sample> for StereoSample {
    fn from(value: Sample) -> Self {
        Self(value, value)
    }
}
impl From<f64> for StereoSample {
    fn from(value: f64) -> Self {
        Self(Sample(value), Sample(value))
    }
}
impl Mul<f64> for StereoSample {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}
impl Mul<Normal> for StereoSample {
    type Output = Self;

    fn mul(self, rhs: Normal) -> Self::Output {
        self * rhs.0
    }
}

/// [FrequencyHz] is a frequency measured in
/// [Hertz](https://en.wikipedia.org/wiki/Hertz), or cycles per second. Because
/// we're usually discussing human hearing or LFOs, we can expect [FrequencyHz]
/// to range from about 0.0 to about 22,000.0. But because of
/// [aliasing](https://en.wikipedia.org/wiki/Nyquist_frequency), it's not
/// surprising to see 2x the upper range, which is where the 44.1kHz CD-quality
/// sampling rate comes from, and when we pick rendering rates, we might go up
/// to 192kHz (2x for sampling a 96kHz signal).
///
/// Eventually we might impose a non-negative restriction on this type.
#[derive(Clone, Copy, Debug, Derivative, PartialEq, PartialOrd, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct FrequencyHz(#[derivative(Default(value = "440.0"))] pub ParameterType);
#[allow(missing_docs)]
impl FrequencyHz {
    pub const FREQUENCY_TO_LINEAR_BASE: ParameterType = 800.0;
    pub const FREQUENCY_TO_LINEAR_COEFFICIENT: ParameterType = 25.0;

    pub const UNITS_SUFFIX: &'static str = " Hz";

    // https://docs.google.com/spreadsheets/d/1uQylh2h77-fuJ6OM0vjF7yjRXflLFP0yQEnv5wbaP2c/edit#gid=0
    // =LOGEST(Sheet1!B2:B23, Sheet1!A2:A23,true, false)
    //
    // Column A is 24db filter percentages from all the patches. Column B is
    // envelope-filter percentages from all the patches.
    pub fn percent_to_frequency(percentage: Normal) -> ParameterType {
        Self::FREQUENCY_TO_LINEAR_COEFFICIENT * Self::FREQUENCY_TO_LINEAR_BASE.powf(percentage.0)
    }

    pub fn frequency_to_percent(frequency: ParameterType) -> Normal {
        debug_assert!(frequency >= 0.0);

        // I was stressed out about slightly negative values, but then I decided
        // that adjusting the log numbers to handle more edge cases wasn't going
        // to make a practical difference. So I'm clamping to [0, 1].
        Normal::from(
            (frequency / Self::FREQUENCY_TO_LINEAR_COEFFICIENT).log(Self::FREQUENCY_TO_LINEAR_BASE),
        )
    }

    pub fn zero() -> Self {
        FrequencyHz(0.0)
    }
}
impl From<f64> for FrequencyHz {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl From<FrequencyHz> for f64 {
    fn from(value: FrequencyHz) -> Self {
        value.0
    }
}
impl From<f32> for FrequencyHz {
    fn from(value: f32) -> Self {
        Self(value as ParameterType)
    }
}
impl From<FrequencyHz> for f32 {
    fn from(value: FrequencyHz) -> Self {
        value.0 as f32
    }
}
impl Mul for FrequencyHz {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl Mul<f64> for FrequencyHz {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl Mul<Ratio> for FrequencyHz {
    type Output = Self;

    fn mul(self, rhs: Ratio) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl Div for FrequencyHz {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl From<usize> for FrequencyHz {
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}
impl From<FrequencyHz> for usize {
    fn from(value: FrequencyHz) -> Self {
        value.0 as usize
    }
}
impl From<MidiNote> for FrequencyHz {
    fn from(value: MidiNote) -> Self {
        let key = value as u8;
        Self::from(2.0_f64.powf((key as f64 - 69.0) / 12.0) * 440.0)
    }
}
// Beware: u7 is understood to represent a MIDI key ranging from 0..128. This
// method will return very strange answers if you're expecting it to hand back
// FrequencyHz(42) from a u7(42).
impl From<u7> for FrequencyHz {
    fn from(value: u7) -> Self {
        Self::from(MidiNote::from_repr(value.as_int() as usize).unwrap())
    }
}
impl Display for FrequencyHz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
#[cfg(feature = "egui")]
impl Numeric for FrequencyHz {
    const INTEGRAL: bool = false;

    const MIN: Self = Self(0.01);

    const MAX: Self = Self(22050.0);

    fn to_f64(self) -> f64 {
        self.0
    }

    fn from_f64(num: f64) -> Self {
        Self(num)
    }
}

/// Useful ranges of frequencies. Originally designed for picking egui widget
/// boundaries.
#[derive(Debug, Default)]
pub enum FrequencyRange {
    /// Most humans can hear (with a little extra on the high end).
    #[default]
    Audible,
    /// Most humans can feel but not hear (with a little extra on either end).
    Subaudible,
    /// Typical digital-audio sampling rates.
    Processing,
}
impl FrequencyRange {
    pub fn as_range(&self) -> RangeInclusive<ParameterType> {
        match self {
            FrequencyRange::Subaudible => 0.01..=64.0,
            FrequencyRange::Audible => 20.0..=22500.0,
            FrequencyRange::Processing => (22500.0 / 8.0)..=(1024.0 * 256.0),
        }
    }

    pub fn as_range_frequency_hz(&self) -> RangeInclusive<FrequencyHz> {
        let range = self.as_range();
        FrequencyHz(*range.start())..=FrequencyHz(*range.end())
    }

    /// The recommended number of digits after the decimal point for this range.
    pub fn fixed_digit_count(&self) -> usize {
        match self {
            FrequencyRange::Subaudible => 2,
            FrequencyRange::Audible => 1,
            FrequencyRange::Processing => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_to_stereo() {
        assert_eq!(StereoSample::from(Sample::MIN), StereoSample::MIN);
        assert_eq!(StereoSample::from(Sample::SILENCE), StereoSample::SILENCE);
        assert_eq!(StereoSample::from(Sample::MAX), StereoSample::MAX);
    }

    #[test]
    fn stereo_to_mono() {
        assert_eq!(Sample::from(StereoSample::MIN), Sample::MIN);
        assert_eq!(Sample::from(StereoSample::SILENCE), Sample::SILENCE);
        assert_eq!(Sample::from(StereoSample::MAX), Sample::MAX);

        assert_eq!(
            Sample::from(StereoSample::new(1.0.into(), 0.0.into())),
            Sample::from(0.5)
        );
    }

    #[test]
    fn convert_sample_to_i16() {
        assert_eq!(Sample::MAX.into_i16(), i16::MAX);
        assert_eq!(Sample::MIN.into_i16(), i16::MIN);
        assert_eq!(Sample::SILENCE.into_i16(), 0);
    }

    #[test]
    fn convert_stereo_sample_to_i16() {
        let s = StereoSample(Sample::MIN, Sample::MAX);
        let (l, r) = s.into_i16();
        assert_eq!(l, i16::MIN);
        assert_eq!(r, i16::MAX);
    }

    #[test]
    fn ratio_ok() {
        assert_eq!(Ratio::from(BipolarNormal::from(-1.0)).0, 0.125);
        assert_eq!(Ratio::from(BipolarNormal::from(0.0)).0, 1.0);
        assert_eq!(Ratio::from(BipolarNormal::from(1.0)).0, 8.0);

        assert_eq!(BipolarNormal::from(Ratio::from(0.125)).0, -1.0);
        assert_eq!(BipolarNormal::from(Ratio::from(1.0)).0, 0.0);
        assert_eq!(BipolarNormal::from(Ratio::from(8.0)).0, 1.0);
    }

    #[test]
    fn ratio_control_ok() {
        assert_eq!(Ratio::from(ControlValue(0.0)).0, 0.125);
        assert_eq!(Ratio::from(ControlValue(0.5)).0, 1.0);
        assert_eq!(Ratio::from(ControlValue(1.0)).0, 8.0);

        assert_eq!(ControlValue::from(Ratio::from(0.125)).0, 0.0);
        assert_eq!(ControlValue::from(Ratio::from(1.0)).0, 0.5);
        assert_eq!(ControlValue::from(Ratio::from(8.0)).0, 1.0);
    }
}
