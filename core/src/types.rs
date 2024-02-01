// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::midi::{u7, MidiNote};
use bounded_vec_deque::BoundedVecDeque;
use crossbeam::{
    channel::{Receiver, Sender},
    queue::ArrayQueue,
};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, RangeInclusive, Sub},
    sync::{Arc, RwLock},
};
use strum_macros::{EnumCount, EnumIter, FromRepr};

pub mod prelude {
    pub use super::{
        BipolarNormal, ChannelPair, FrequencyHz, FrequencyRange, Normal, ParameterType, Ratio,
        Sample, SampleType, SignalType, StereoSample, TrackTitle,
    };
}

/// [SampleType] is the underlying primitive that makes up [StereoSample].
pub type SampleType = f64;

/// [SignalType] is the primitive used for general digital signal-related work.
pub type SignalType = f64;

/// Use [ParameterType] in places where a [Normal] or [BipolarNormal] could fit,
/// except you don't have any range restrictions. Any such usage should be
/// temporary.
pub type ParameterType = f64;

/// [Sample] represents a single-channel audio sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
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
impl From<f64> for Sample {
    fn from(value: f64) -> Self {
        Sample(value)
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

// TODO: I tried implementing this using the sort-of new generic const
// expressions, because I wanted to see whether I could have compile-time
// errors for attempts to set the value outside the range. I did not succeed.

/// [RangedF64] enforces the given range limits while not becoming too expensive
/// to use compared to a plain f64. It enforces the value at creation, when
/// setting it explicitly, when converting from an f64, and when getting it. But
/// math operations (Add, Sub, etc.) are not checked! This allows certain
/// operations to (hopefully temporarily) exceed the range, or for
/// floating-point precision problems to (again hopefully) get compensated for
/// later on.
///
/// Also note that [RangedF64] doesn't tell you when clamping happens. It just
/// does it, silently.
///
/// Altogether, [RangedF64] is good for gatekeeping -- parameters, return
/// values, etc., -- and somewhat OK at pure math. But we might decide to clamp
/// (heh) down on out-of-bounds conditions later on, so if you want to do math,
/// prefer f64 sourced from [RangedF64] rather than [RangedF64] itself.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RangedF64<const LOWER: i8, const UPPER: i8>(pub f64);
#[allow(missing_docs)]
impl<const LOWER: i8, const UPPER: i8> RangedF64<LOWER, UPPER> {
    /// The highest valid value.
    pub const MAX: f64 = UPPER as f64;
    /// The lowest valid value.
    pub const MIN: f64 = LOWER as f64;
    /// A zero value.
    pub const ZERO: f64 = 0.0;

    pub fn new(value: f64) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }
    #[deprecated]
    pub fn new_from_f32(value: f32) -> Self {
        Self::new(value as f64)
    }
    // These methods are annoying because they're inconsistent with the others
    // in this file. For example, StereoSample::MAX is a struct, not a
    // primitive. I think this happened because (1) a generic can't define a
    // constant like that -- which is reasonable -- but (2) I then defined
    // Normal/BipolarNormal etc. as old-style types, which meant I couldn't put
    // any consts inside them. TODO: try a new one of the newtype style, and
    // then take a afternoon converting the world to the new ones.
    pub const fn maximum() -> Self {
        Self(Self::MAX)
    }
    pub const fn minimum() -> Self {
        Self(Self::MIN)
    }
    pub const fn zero() -> Self {
        Self(Self::ZERO)
    }
    pub fn set(&mut self, value: f64) {
        self.0 = value.clamp(Self::MIN, Self::MAX);
    }

    pub fn scale(&self, factor: f64) -> f64 {
        self.0 * factor
    }

    pub fn to_percentage(&self) -> f64 {
        self.0 * 100.0
    }

    pub fn from_percentage(percentage: f64) -> Self {
        Self(percentage / 100.0)
    }
}
impl<const LOWER: i8, const UPPER: i8> Display for RangedF64<LOWER, UPPER> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
impl<const LOWER: i8, const UPPER: i8> Add for RangedF64<LOWER, UPPER> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl<const LOWER: i8, const UPPER: i8> Sub for RangedF64<LOWER, UPPER> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl<const LOWER: i8, const UPPER: i8> Add<f64> for RangedF64<LOWER, UPPER> {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl<const LOWER: i8, const UPPER: i8> Sub<f64> for RangedF64<LOWER, UPPER> {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self::Output {
        Self(self.0 - rhs)
    }
}
impl<const LOWER: i8, const UPPER: i8> From<RangedF64<LOWER, UPPER>> for f64 {
    fn from(value: RangedF64<LOWER, UPPER>) -> Self {
        value.0.clamp(Self::MIN, Self::MAX)
    }
}
impl<const LOWER: i8, const UPPER: i8> From<f64> for RangedF64<LOWER, UPPER> {
    fn from(value: f64) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }
}
impl<const LOWER: i8, const UPPER: i8> From<f32> for RangedF64<LOWER, UPPER> {
    fn from(value: f32) -> Self {
        Self(value.clamp(Self::MIN as f32, Self::MAX as f32) as f64)
    }
}
impl<const LOWER: i8, const UPPER: i8> MulAssign for RangedF64<LOWER, UPPER> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0 * rhs.0;
    }
}
impl<const LOWER: i8, const UPPER: i8> MulAssign<f64> for RangedF64<LOWER, UPPER> {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 = self.0 * rhs;
    }
}
/// A Normal is a [RangedF64] whose range is [0.0, 1.0].
pub type Normal = RangedF64<0, 1>;
#[allow(missing_docs)]
impl Normal {
    pub const fn range() -> RangeInclusive<f64> {
        0.0..=1.0
    }
    pub const fn new_const(value: f64) -> Self {
        Self(value)
    }
}
impl Default for Normal {
    // I'm deciding by royal fiat that a Normal defaults to 1.0. I keep running
    // into cases where a Normal gets default-constructed and zeroing out a
    // signal.
    fn default() -> Self {
        Self(1.0)
    }
}
impl From<Sample> for Normal {
    // Sample -1.0..=1.0
    // Normal 0.0..=1.0
    fn from(value: Sample) -> Self {
        Self(value.0 * 0.5 + 0.5)
    }
}
impl From<BipolarNormal> for Normal {
    fn from(value: BipolarNormal) -> Self {
        Self(value.0 * 0.5 + 0.5)
    }
}
impl From<FrequencyHz> for Normal {
    fn from(value: FrequencyHz) -> Self {
        FrequencyHz::frequency_to_percent(value.0)
    }
}
impl Mul<Normal> for f64 {
    type Output = Self;

    fn mul(self, rhs: Normal) -> Self::Output {
        self * rhs.0
    }
}
impl Mul<f64> for Normal {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl From<Normal> for f32 {
    fn from(val: Normal) -> Self {
        val.0 as f32
    }
}
impl Mul<Self> for Normal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl Sub<Normal> for f64 {
    type Output = Self;

    fn sub(self, rhs: Normal) -> Self::Output {
        self - rhs.0
    }
}

/// A BipolarNormal is a [RangedF64] whose range is [-1.0, 1.0].
pub type BipolarNormal = RangedF64<-1, 1>;
#[allow(missing_docs)]
impl BipolarNormal {
    pub const fn range() -> RangeInclusive<f64> {
        -1.0..=1.0
    }
}
impl Default for BipolarNormal {
    fn default() -> Self {
        Self(0.0)
    }
}

impl From<Sample> for BipolarNormal {
    // A [Sample] has the same range as a [BipolarNormal], so no conversion is
    // necessary.
    fn from(value: Sample) -> Self {
        Self(value.0)
    }
}
impl Mul<Normal> for BipolarNormal {
    type Output = BipolarNormal;

    fn mul(self, rhs: Normal) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl From<BipolarNormal> for StereoSample {
    fn from(value: BipolarNormal) -> Self {
        StereoSample::from(value.0)
    }
}
impl From<Normal> for BipolarNormal {
    fn from(value: Normal) -> Self {
        Self(value.0 * 2.0 - 1.0)
    }
}
impl From<Normal> for FrequencyHz {
    fn from(val: Normal) -> Self {
        FrequencyHz::percent_to_frequency(val).into()
    }
}
impl From<BipolarNormal> for f32 {
    fn from(val: BipolarNormal) -> Self {
        val.into()
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

/// The [Ratio] type is a multiplier. A value of 2.0 would multiply another
/// value by two (a x 2.0:1.0), and a value of 0.5 would divide it by two (a x
/// 1.0:2.0 = a x 0.5).
///
/// Negative ratios are meaningless for current use cases.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Ratio(pub ParameterType);
impl Default for Ratio {
    fn default() -> Self {
        Self(1.0)
    }
}
impl From<f64> for Ratio {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl From<BipolarNormal> for Ratio {
    fn from(value: BipolarNormal) -> Self {
        Self(2.0f64.powf(value.0 * 3.0))
    }
}
impl From<Ratio> for BipolarNormal {
    fn from(value: Ratio) -> Self {
        BipolarNormal::from(value.0.log2() / 3.0)
    }
}
impl From<Normal> for Ratio {
    fn from(value: Normal) -> Self {
        Self::from(BipolarNormal::from(value))
    }
}
impl From<Ratio> for Normal {
    fn from(value: Ratio) -> Self {
        Self::from(BipolarNormal::from(value))
    }
}
impl From<f32> for Ratio {
    fn from(value: f32) -> Self {
        Self(value as ParameterType)
    }
}
impl Mul<ParameterType> for Ratio {
    type Output = Self;

    fn mul(self, rhs: ParameterType) -> Self::Output {
        Ratio(self.0 * rhs)
    }
}
impl Div<ParameterType> for Ratio {
    type Output = Self;

    fn div(self, rhs: ParameterType) -> Self::Output {
        Ratio(self.0 / rhs)
    }
}
impl Mul<Ratio> for ParameterType {
    type Output = Self;

    fn mul(self, rhs: Ratio) -> Self::Output {
        self * rhs.0
    }
}
impl Div<Ratio> for ParameterType {
    type Output = Self;

    fn div(self, rhs: Ratio) -> Self::Output {
        self / rhs.0
    }
}

/// A ring buffer of stereo samples that the audio stream consumes.
pub type AudioQueue = Arc<ArrayQueue<StereoSample>>;

/// A ring buffer of mono samples used to visualize the generated audio stream.
#[derive(Debug)]
pub struct VisualizationQueue(pub Arc<RwLock<BoundedVecDeque<Sample>>>);
impl Default for VisualizationQueue {
    fn default() -> Self {
        const LEN: usize = 256;
        let mut deque = VecDeque::new();
        deque.resize(LEN, Sample::default());
        Self(Arc::new(RwLock::new(BoundedVecDeque::from_unbounded(
            deque, LEN,
        ))))
    }
}
impl Clone for VisualizationQueue {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

/// A convenience struct to bundle both halves of a [crossbeam_channel]
/// together.
///
/// This is actually for more than just convenience: because Serde needs to be
/// able to assign defaults to individual fields on a struct by calling
/// stateless functions, we have to create both sender and receiver at once in a
/// single field.
#[derive(Debug)]
pub struct ChannelPair<T> {
    #[allow(missing_docs)]
    pub sender: Sender<T>,
    #[allow(missing_docs)]
    pub receiver: Receiver<T>,
}
impl<T> Default for ChannelPair<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

// TODO: TrackTitle is parked here because I couldn't find a better place for it.

/// Newtype for track title string.
#[derive(Debug, Derivative, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct TrackTitle(#[derivative(Default(value = "\"Untitled\".to_string()"))] pub String);
impl From<&str> for TrackTitle {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// [ModSerial] is a simple counter that lets us inform subscribers that
/// something has changed. Subscribers should keep a usize and compare to see
/// whether it differs from the one that we're currently reporting. If it does,
/// then they should update it and deal with the change.
#[derive(Debug, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct ModSerial(
    // We start at something other than usize::default() so that
    // everyone else can use the default value and fire their update
    // code on the first call to has_changed().
    #[derivative(Default(value = "1000"))] pub usize,
);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    EnumCount,
    EnumIter,
    Eq,
    FromRepr,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    Red,
    Vermilion,
    Orange,
    Amber,
    Yellow,
    Lime,
    Chartreuse,
    Ddahal,
    Green,
    Erin,
    Spring,
    Gashyanta,
    Cyan,
    Capri,
    Azure,
    Cerulean,
    Blue,
    Volta,
    Violet,
    Llew,
    Magenta,
    Cerise,
    Rose,
    Crimson,
    #[default]
    Gray1,
    Gray2,
    Gray3,
    Gray4,
    Gray5,
    Gray6,
    Gray7,
    Gray8,
}

#[cfg(test)]
mod tests {
    use crate::control::ControlValue;

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
    fn normal_mainline() {
        let a = Normal::new(0.2);
        let b = Normal::new(0.1);

        // Add(Normal)
        assert_eq!(a + b, Normal::new(0.2 + 0.1), "Addition should work.");

        // Sub(Normal)
        assert_eq!(a - b, Normal::new(0.1), "Subtraction should work.");

        // Add(f64)
        assert_eq!(a + 0.2f64, Normal::new(0.4), "Addition of f64 should work.");

        // Sub(f64)
        assert_eq!(a - 0.1, Normal::new(0.1), "Subtraction of f64 should work.");
    }

    #[test]
    fn normal_out_of_bounds() {
        assert_eq!(
            Normal::new(-1.0),
            Normal::new(0.0),
            "Normal below 0.0 should be clamped to 0.0"
        );
        assert_eq!(
            Normal::new(1.1),
            Normal::new(1.0),
            "Normal above 1.0 should be clamped to 1.0"
        );
    }

    #[test]
    fn convert_sample_to_normal() {
        assert_eq!(
            Normal::from(Sample(-0.5)),
            Normal::new(0.25),
            "Converting Sample -0.5 to Normal should yield 0.25"
        );
        assert_eq!(
            Normal::from(Sample(0.0)),
            Normal::new(0.5),
            "Converting Sample 0.0 to Normal should yield 0.5"
        );
    }

    #[test]
    fn convert_bipolar_normal_to_normal() {
        assert_eq!(
            Normal::from(BipolarNormal::from(-1.0)),
            Normal::new(0.0),
            "Bipolar -> Normal wrong"
        );
        assert_eq!(
            Normal::from(BipolarNormal::from(0.0)),
            Normal::new(0.5),
            "Bipolar -> Normal wrong"
        );
        assert_eq!(
            Normal::from(BipolarNormal::from(1.0)),
            Normal::new(1.0),
            "Bipolar -> Normal wrong"
        );
    }

    #[test]
    fn convert_normal_to_bipolar_normal() {
        assert_eq!(
            BipolarNormal::from(Normal::from(0.0)),
            BipolarNormal::new(-1.0),
            "Normal -> Bipolar wrong"
        );
        assert_eq!(
            BipolarNormal::from(Normal::from(0.5)),
            BipolarNormal::new(0.0),
            "Normal -> Bipolar wrong"
        );
        assert_eq!(
            BipolarNormal::from(Normal::from(1.0)),
            BipolarNormal::new(1.0),
            "Normal -> Bipolar wrong"
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
