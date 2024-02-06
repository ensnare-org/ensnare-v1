// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Handles digital-audio, wall-clock, and musical time.

use crate::prelude::*;
use anyhow::{anyhow, Error};
use derivative::Derivative;
use derive_more::Display;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    ops::{Add, AddAssign, Div, Mul, Range, Sub, SubAssign},
};
use strum_macros::{FromRepr, IntoStaticStr};

/// Beats per minute.
#[derive(Clone, Copy, Debug, Derivative, Serialize, Deserialize, PartialEq)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct Tempo(#[derivative(Default(value = "128.0"))] pub ParameterType);
impl fmt::Display for Tempo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:0.2} BPM", self.0))
    }
}
impl From<u16> for Tempo {
    fn from(value: u16) -> Self {
        Self(value as ParameterType)
    }
}
impl From<ParameterType> for Tempo {
    fn from(value: ParameterType) -> Self {
        Self(value)
    }
}
impl Tempo {
    /// The largest value we'll allow.
    pub const MAX_VALUE: ParameterType = 1024.0;

    /// The smallest value we'll allow. Note that zero is actually a degenerate
    /// case... maybe we should be picking 0.1 or similar.
    pub const MIN_VALUE: ParameterType = 0.0;

    /// Beats per second.
    pub fn bps(&self) -> ParameterType {
        self.0 / 60.0
    }

    pub fn range() -> std::ops::RangeInclusive<ParameterType> {
        Self::MIN_VALUE..=Self::MAX_VALUE
    }
}

/// [BeatValue] enumerates numerical divisors used in most music.  
#[derive(Clone, Debug, Default, FromRepr, IntoStaticStr)]
pub enum BeatValue {
    /// large/maxima
    Octuple = 128,
    /// long
    Quadruple = 256,
    /// breve
    Double = 512,
    /// semibreve
    Whole = 1024,
    /// minim
    Half = 2048,
    /// crotchet
    #[default]
    Quarter = 4096,
    /// quaver
    Eighth = 8192,
    /// semiquaver
    Sixteenth = 16384,
    /// demisemiquaver
    ThirtySecond = 32768,
    /// hemidemisemiquaver
    SixtyFourth = 65536,
    /// semihemidemisemiquaver / quasihemidemisemiquaver
    OneHundredTwentyEighth = 131072,
    /// demisemihemidemisemiquaver
    TwoHundredFiftySixth = 262144,
    /// winner winner chicken dinner
    FiveHundredTwelfth = 524288,
}
#[allow(missing_docs)]
impl BeatValue {
    pub fn divisor(value: BeatValue) -> f64 {
        value as u32 as f64 / 1024.0
    }

    pub fn from_divisor(divisor: f32) -> anyhow::Result<Self, anyhow::Error> {
        if let Some(value) = BeatValue::from_repr((divisor * 1024.0) as usize) {
            Ok(value)
        } else {
            Err(anyhow!("divisor {} is out of range", divisor))
        }
    }
}

/// [TimeSignature] represents a music [time
/// signature](https://en.wikipedia.org/wiki/Time_signature).
///
/// The top number of a time signature tells how many beats are in a measure.
/// The bottom number tells the value of a beat. For example, if the bottom
/// number is 4, then a beat is a quarter-note. And if the top number is 4, then
/// you should expect to see four beats in a measure, or four quarter-notes in a
/// measure.
///
/// If your song is playing at 60 beats per minute, and it's 4/4, then a
/// measure's worth of the song should complete in four seconds. That's because
/// each beat takes a second (60 beats/minute, 60 seconds/minute -> 60/60
/// beats/second = 60/60 seconds/beat), and a measure takes four beats (4
/// beats/measure * 1 second/beat = 4/1 seconds/measure).
///
/// If your song is playing at 120 beats per minute, and it's 4/4, then a
/// measure's worth of the song should complete in two seconds. That's because
/// each beat takes a half-second (120 beats/minute, 60 seconds/minute -> 120/60
/// beats/second = 60/120 seconds/beat), and a measure takes four beats (4
/// beats/measure * 1/2 seconds/beat = 4/2 seconds/measure).
#[derive(Clone, Control, Copy, Debug, Derivative, Eq, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct TimeSignature {
    /// The number of beats in a measure.
    #[control]
    #[derivative(Default(value = "4"))]
    pub top: usize,

    /// The value of a beat. Expressed as a reciprocal; for example, if it's 4,
    /// then the beat value is 1/4 or a quarter note.
    #[control]
    #[derivative(Default(value = "4"))]
    pub bottom: usize,
}
impl Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}/{}", self.top, self.bottom))
    }
}
#[allow(missing_docs)]
impl TimeSignature {
    /// C time = common time = 4/4
    /// https://en.wikipedia.org/wiki/Time_signature
    pub const COMMON_TIME: Self = TimeSignature { top: 4, bottom: 4 };

    /// 𝄵 time = cut common time = alla breve = 2/2
    /// https://en.wikipedia.org/wiki/Time_signature
    pub const CUT_TIME: Self = TimeSignature { top: 2, bottom: 2 };

    pub fn new_with(top: usize, bottom: usize) -> anyhow::Result<Self, Error> {
        if top == 0 {
            Err(anyhow!("Time signature top can't be zero."))
        } else if BeatValue::from_divisor(bottom as f32).is_ok() {
            Ok(Self { top, bottom })
        } else {
            Err(anyhow!("Time signature bottom was out of range."))
        }
    }

    /// Returns the duration, in [MusicalTime], of a single bar of music having
    /// this time signature. Note that [MusicalTime] requires a [Tempo] to
    /// calculate wall-clock time.
    pub fn duration(&self) -> MusicalTime {
        MusicalTime::new_with_beats(self.top())
    }

    pub fn beat_value(&self) -> BeatValue {
        // It's safe to unwrap because the constructor already blew up if the
        // bottom were out of range.
        BeatValue::from_divisor(self.bottom as f32).unwrap()
    }

    /// Sets the top value.
    pub fn set_top(&mut self, top: usize) {
        self.top = top;
    }

    /// Sets the bottom value. Must be a power of two. Does not check for
    /// validity.
    pub fn set_bottom(&mut self, bottom: usize) {
        self.bottom = bottom;
    }

    /// The top value.
    pub fn top(&self) -> usize {
        self.top
    }

    /// The bottom value.
    pub fn bottom(&self) -> usize {
        self.bottom
    }
}

/// [MusicalTime] is the universal unit of time. It is in terms of musical
/// beats. A "part" is a sixteenth of a beat, and a "unit" is 1/4096 of a part.
/// Thus, beats are divided into 65,536 units.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct MusicalTime {
    /// A unit is 1/65536 of a beat.
    units: usize,
}

#[allow(missing_docs)]
impl MusicalTime {
    /// A part is a sixteenth of a beat.
    pub const PARTS_IN_BEAT: usize = 16;
    pub const UNITS_IN_PART: usize = 4096;
    pub const UNITS_IN_BEAT: usize = Self::PARTS_IN_BEAT * Self::UNITS_IN_PART;

    /// A breve is also called a "double whole note"
    pub const DURATION_BREVE: MusicalTime = Self::new_with_beats(2);
    pub const DURATION_WHOLE: MusicalTime = Self::new_with_beats(1);
    pub const DURATION_HALF: MusicalTime = Self::new_with_parts(8);
    pub const DURATION_QUARTER: MusicalTime = Self::new_with_parts(4);
    pub const DURATION_EIGHTH: MusicalTime = Self::new_with_parts(2);
    pub const DURATION_SIXTEENTH: MusicalTime = Self::new_with_parts(1);
    pub const DURATION_ZERO: MusicalTime = Self::START;
    pub const TIME_ZERO: MusicalTime = Self::new_with_units(0);
    pub const TIME_END_OF_FIRST_BEAT: MusicalTime = Self::new_with_beats(1);
    pub const TIME_MAX: MusicalTime = Self::new_with_units(usize::MAX);

    pub const ONE_PART: MusicalTime = Self::new_with_parts(1);
    pub const ONE_UNIT: MusicalTime = Self::new_with_units(1);
    pub const ONE_BEAT: MusicalTime = Self::new_with_beats(1);
    pub const FOUR_FOUR_MEASURE: MusicalTime = Self::new_with_bars(&TimeSignature::COMMON_TIME, 1);

    pub const START: MusicalTime = Self::new_with_units(0);

    pub fn new(
        time_signature: &TimeSignature,
        bars: usize,
        beats: usize,
        parts: usize,
        units: usize,
    ) -> Self {
        Self {
            units: Self::bars_to_units(time_signature, bars)
                + Self::beats_to_units(beats)
                + Self::parts_to_units(parts)
                + units,
        }
    }

    // The entire number expressed in bars. This is provided for uniformity;
    // it's the highest unit in the struct, so total_bars() is always the same
    // as bars().
    pub fn total_bars(&self, time_signature: &TimeSignature) -> usize {
        self.bars(time_signature)
    }

    pub fn bars(&self, time_signature: &TimeSignature) -> usize {
        self.total_beats() / time_signature.top
    }

    #[allow(unused_variables)]
    pub fn set_bars(&mut self, bars: usize) {
        panic!()
    }

    // The entire number expressed in beats.
    pub fn total_beats(&self) -> usize {
        self.units / Self::UNITS_IN_BEAT
    }

    pub fn beats(&self, time_signature: &TimeSignature) -> usize {
        self.total_beats() % time_signature.top
    }

    #[allow(unused_variables)]
    pub fn set_beats(&mut self, beats: u8) {
        panic!()
    }

    // The entire number expressed in parts.
    pub fn total_parts(&self) -> usize {
        self.units / Self::UNITS_IN_PART
    }

    // A part is one sixteenth of a beat.
    pub fn parts(&self) -> usize {
        self.total_parts() % Self::PARTS_IN_BEAT
    }

    #[allow(unused_variables)]
    pub fn set_parts(&mut self, parts: u8) {
        panic!()
    }

    // The entire number expressed in units.
    pub const fn total_units(&self) -> usize {
        self.units
    }

    pub const fn units(&self) -> usize {
        self.units % Self::UNITS_IN_PART
    }

    #[allow(unused_variables)]
    pub fn set_units(&mut self, units: usize) {
        panic!()
    }

    pub fn reset(&mut self) {
        self.units = Default::default();
    }

    pub const fn bars_to_units(time_signature: &TimeSignature, bars: usize) -> usize {
        Self::beats_to_units(time_signature.top * bars)
    }

    pub const fn beats_to_units(beats: usize) -> usize {
        beats * Self::UNITS_IN_BEAT
    }

    pub const fn parts_to_units(parts: usize) -> usize {
        parts * (Self::UNITS_IN_PART)
    }

    pub const fn new_with_bars(time_signature: &TimeSignature, bars: usize) -> Self {
        Self::new_with_beats(time_signature.top * bars)
    }

    pub const fn new_with_beats(beats: usize) -> Self {
        Self::new_with_units(beats * Self::UNITS_IN_BEAT)
    }

    pub fn new_with_fractional_beats(beats: f64) -> Self {
        Self::new_with_units((beats * Self::UNITS_IN_BEAT as f64) as usize)
    }

    pub const fn new_with_parts(parts: usize) -> Self {
        Self::new_with_units(parts * Self::UNITS_IN_PART)
    }

    pub const fn new_with_units(units: usize) -> Self {
        Self { units }
    }

    pub fn new_with_frames(tempo: Tempo, sample_rate: SampleRate, frames: usize) -> Self {
        Self::new_with_units(Self::frames_to_units(tempo, sample_rate, frames))
    }

    pub fn frames_to_units(tempo: Tempo, sample_rate: SampleRate, frames: usize) -> usize {
        let elapsed_beats = (frames as f64 / sample_rate.0 as f64) * tempo.bps();
        let elapsed_fractional_units =
            (elapsed_beats.fract() * Self::UNITS_IN_BEAT as f64 + 0.5) as usize;
        Self::beats_to_units(elapsed_beats.floor() as usize) + elapsed_fractional_units
    }

    pub fn units_to_frames(tempo: Tempo, sample_rate: SampleRate, units: usize) -> usize {
        let frames_per_second: f64 = sample_rate.into();
        let seconds_per_beat = 1.0 / tempo.bps();
        let frames_per_beat = seconds_per_beat * frames_per_second;

        (frames_per_beat * (units as f64 / Self::UNITS_IN_BEAT as f64) + 0.5) as usize
    }

    /// Returns a [Range] that contains nothing.
    pub fn empty_range() -> std::ops::Range<Self> {
        std::ops::Range {
            start: Self::TIME_MAX,
            end: Self::TIME_MAX,
        }
    }

    pub fn as_frames(&self, tempo: Tempo, sample_rate: SampleRate) -> usize {
        Self::units_to_frames(tempo, sample_rate, self.units)
    }

    pub fn quantized(&self, quantum: MusicalTime) -> MusicalTime {
        let quanta = (self.units + quantum.units / 2) / quantum.units;
        MusicalTime::new_with_units(quanta * quantum.units)
    }

    // TODO: this is an oversimplied heuristic to quantize according to the
    // given time signature.
    pub fn quantized_for_time_signature(&self, time_signature: &TimeSignature) -> MusicalTime {
        self.quantized(MusicalTime::ONE_BEAT / time_signature.bottom)
    }

    pub fn quantized_to_measure(&self, time_signature: &TimeSignature) -> MusicalTime {
        self.quantized(MusicalTime::new_with_beats(time_signature.top))
    }
}
impl Display for MusicalTime {
    // Because MusicalTime doesn't know the time signature, it can't display the
    // number of bars here.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:5}.{:02}.{:05}",
            self.total_beats() + 1,
            self.parts(),
            self.units()
        )
    }
}
impl Add<Self> for MusicalTime {
    type Output = Self;

    // We look at only the left side's beats-per-bar value, rather than trying
    // to reconcile different ones.
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            units: self.units + rhs.units,
        }
    }
}
impl Add<usize> for MusicalTime {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            units: self.units + rhs,
        }
    }
}
impl AddAssign<Self> for MusicalTime {
    fn add_assign(&mut self, rhs: Self) {
        self.units += rhs.units;
    }
}
impl SubAssign<Self> for MusicalTime {
    fn sub_assign(&mut self, rhs: Self) {
        self.units -= rhs.units;
    }
}
impl Mul<usize> for MusicalTime {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self {
            units: self.units * rhs,
        }
    }
}
impl Div<usize> for MusicalTime {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        Self {
            units: self.units / rhs,
        }
    }
}
impl Sub<Self> for MusicalTime {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            units: self.units - rhs.units,
        }
    }
}

/// A [ViewRange] indicates a musical time range. It's used to determine what
/// the UI should show when it's rendering something in a timeline.
#[derive(Debug, Derivative, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct ViewRange(
    #[derivative(Default(value = "MusicalTime::START..MusicalTime::new_with_beats(4)"))]
    pub  std::ops::Range<MusicalTime>,
);
impl From<Range<MusicalTime>> for ViewRange {
    fn from(value: Range<MusicalTime>) -> Self {
        Self(value.start..value.end)
    }
}

/// A [TimeRange] describes a range of [MusicalTime]. Its principal usage is to
/// determine which time slice to handle during [Controls::work()].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TimeRange(pub std::ops::Range<MusicalTime>);
impl TimeRange {
    pub fn new_with_start_and_end(start: MusicalTime, end: MusicalTime) -> Self {
        Self(start..end)
    }
    pub fn new_with_start_and_duration(start: MusicalTime, duration: MusicalTime) -> Self {
        Self(start..(start + duration))
    }

    /// Ensures that the extent includes the extent of the given item.
    pub fn expand_with_range(&mut self, item: &TimeRange) {
        self.0.start = self.0.start.min(item.0.start);
        self.0.end = self.0.end.max(item.0.end);
    }

    /// Ensures that the extent includes the given instant.
    pub fn expand_with_time(&mut self, time: MusicalTime) {
        self.0.start = self.0.start.min(time);
        self.0.end = self.0.end.max(time);
    }

    /// Adds to both start and end. This is less ambiguous than implementing
    /// Add<MusicalTime>, which could reasonably add only to the end.
    pub fn shift_right(&self, time: MusicalTime) -> TimeRange {
        TimeRange(self.0.start + time..self.0.end + time)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Seconds(pub f64);
impl Seconds {
    pub fn zero() -> Seconds {
        Seconds(0.0)
    }

    pub fn infinite() -> Seconds {
        Seconds(-1.0)
    }
}
impl From<f64> for Seconds {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl From<f32> for Seconds {
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}
impl From<Seconds> for f64 {
    fn from(value: Seconds) -> Self {
        value.0
    }
}
impl From<Seconds> for f32 {
    fn from(value: Seconds) -> Self {
        value.0 as f32
    }
}
impl Add<f64> for Seconds {
    type Output = Seconds;

    fn add(self, rhs: f64) -> Self::Output {
        Seconds(self.0 + rhs)
    }
}
impl Add<Seconds> for Seconds {
    type Output = Self;

    fn add(self, rhs: Seconds) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl Div<Seconds> for Seconds {
    type Output = Self;

    fn div(self, rhs: Seconds) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl Mul<f64> for Seconds {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl Div<f64> for Seconds {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}
impl Div<usize> for Seconds {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as f64)
    }
}

/// Samples per second. Always a positive integer; cannot be zero.
#[derive(Clone, Copy, Debug, Derivative, Display, PartialEq, Eq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct SampleRate(#[derivative(Default(value = "44100"))] pub usize);
#[allow(missing_docs)]
impl SampleRate {
    pub const DEFAULT_SAMPLE_RATE: usize = 44100;
    pub const DEFAULT: SampleRate = SampleRate::new(Self::DEFAULT_SAMPLE_RATE);

    pub const fn new(value: usize) -> Self {
        if value != 0 {
            Self(value)
        } else {
            Self(Self::DEFAULT_SAMPLE_RATE)
        }
    }
}
impl From<f64> for SampleRate {
    fn from(value: f64) -> Self {
        Self::new(value as usize)
    }
}
impl From<SampleRate> for f64 {
    fn from(value: SampleRate) -> Self {
        value.0 as f64
    }
}
impl From<SampleRate> for usize {
    fn from(value: SampleRate) -> Self {
        value.0
    }
}
impl From<usize> for SampleRate {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
impl From<SampleRate> for u32 {
    fn from(value: SampleRate) -> Self {
        value.0 as u32
    }
}
impl Mul<Seconds> for SampleRate {
    type Output = SampleRate;

    // TODO: written in a fugue state, not sure it makes sense. Context is
    // (sample rate x seconds) = buffer size. It works for that case, but I'm
    // not sure it generally works.
    fn mul(self, rhs: Seconds) -> Self::Output {
        Self((self.0 as f64 * rhs.0) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::{ControlIndex, ControlValue, Controllable};

    #[test]
    fn tempo() {
        let t = Tempo::default();
        assert_eq!(t.0, 128.0);
    }

    #[test]
    fn sample_rate_default_is_sane() {
        let sr = SampleRate::default();
        assert_eq!(sr.0, 44100);
    }

    #[test]
    fn valid_time_signatures_can_be_instantiated() {
        let ts = TimeSignature::default();
        assert_eq!(ts.top, 4);
        assert_eq!(ts.bottom, 4);

        let _ts = TimeSignature::new_with(ts.top, ts.bottom).ok().unwrap();
        // assert!(matches!(ts.beat_value(), BeatValue::Quarter));
    }

    #[test]
    fn time_signature_with_bad_top_is_invalid() {
        assert!(TimeSignature::new_with(0, 4).is_err());
    }

    #[test]
    fn time_signature_with_bottom_not_power_of_two_is_invalid() {
        assert!(TimeSignature::new_with(4, 5).is_err());
    }

    #[test]
    fn time_signature_invalid_bottom_below_range() {
        assert!(TimeSignature::new_with(4, 0).is_err());
    }

    #[test]
    fn time_signature_invalid_bottom_above_range() {
        // 2^10 = 1024, 1024 * 1024 = 1048576, which is higher than
        // BeatValue::FiveHundredTwelfth value of 524288
        let bv = BeatValue::from_divisor(2.0f32.powi(10));
        assert!(bv.is_err());
    }

    #[test]
    fn musical_time_at_time_zero() {
        // Default is time zero
        let t = MusicalTime::default();
        assert_eq!(t.total_bars(&TimeSignature::default()), 0);
        assert_eq!(t.total_beats(), 0);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.units(), 0);
    }

    #[test]
    fn musical_time_to_frame_conversions() {
        let ts = TimeSignature::default();
        let tempo = Tempo::default();
        let sample_rate = SampleRate::default();

        // These are here to catch any change in defaults that would invalidate lots of tests.
        assert_eq!(ts.top, 4);
        assert_eq!(ts.bottom, 4);
        assert_eq!(tempo.0, 128.0);
        assert_eq!(<SampleRate as Into<usize>>::into(sample_rate), 44100);

        const ONE_4_4_BAR_IN_SECONDS: f64 = 60.0 * 4.0 / 128.0;
        const ONE_BEAT_IN_SECONDS: f64 = 60.0 / 128.0;
        const ONE_PART_IN_SECONDS: f64 = ONE_BEAT_IN_SECONDS / 16.0;
        const ONE_UNIT_IN_SECONDS: f64 = ONE_BEAT_IN_SECONDS / (16.0 * 4096.0);
        assert_eq!(ONE_4_4_BAR_IN_SECONDS, 1.875);
        assert_eq!(ONE_BEAT_IN_SECONDS, 0.46875);

        for (bars, beats, parts, units, seconds) in [
            (0, 0, 0, 0, 0.0),
            (0, 0, 0, 1, ONE_UNIT_IN_SECONDS),
            (0, 0, 1, 0, ONE_PART_IN_SECONDS),
            (0, 1, 0, 0, ONE_BEAT_IN_SECONDS),
            (1, 0, 0, 0, ONE_4_4_BAR_IN_SECONDS),
            (128 / 4, 0, 0, 0, 60.0),
        ] {
            let sample_rate_f64: f64 = sample_rate.into();
            let frames = (seconds * sample_rate_f64).round() as usize;
            let time = MusicalTime::new(&ts, bars, beats, parts, units);
            assert_eq!(
                time.as_frames(tempo, sample_rate),
                frames,
                "Expected {}.{}.{}.{} -> {} frames",
                bars,
                beats,
                parts,
                units,
                frames,
            );
        }
    }

    #[test]
    fn frame_to_musical_time_conversions() {
        let ts = TimeSignature::default();
        let tempo = Tempo::default();
        let sample_rate = SampleRate::default();

        for (frames, bars, beats, parts, units) in [
            (0, 0, 0, 0, 0),
            (2646000, 32, 0, 0, 0), // one full minute
            (44100, 0, 2, 2, 546),  // one second = 128 bpm / 60 seconds/min =
                                    // 2.13333333 beats, which breaks down to 2
                                    // beats, 2 parts that are each 1/16 of a
                                    // beat = 2.133333 parts (yeah, that happens
                                    // to be the same as the 2.133333 for
                                    // beats), and multiply the .1333333 by 4096
                                    // to get units.
        ] {
            assert_eq!(
                MusicalTime::new(&ts, bars, beats, parts, units).total_units(),
                MusicalTime::frames_to_units(tempo, sample_rate, frames),
                "Expected {} frames -> {}.{}.{}.{}",
                frames,
                bars,
                beats,
                parts,
                units,
            );
        }
    }

    #[test]
    fn conversions_are_consistent() {
        let ts = TimeSignature::default();
        let tempo = Tempo::default();

        // We're picking a nice round number so that we don't hit tricky .99999 issues.
        let sample_rate = SampleRate::from(32768);

        for bars in 0..4 {
            for beats in 0..ts.top() {
                for parts in 0..MusicalTime::PARTS_IN_BEAT {
                    // If we stick to just a part-level division of MusicalTime, then we expect time ->
                    // frames -> time to be exact, because frames is
                    // (typically) higher resolution than time. But frames
                    // -> time -> frames is not expected to be exact.
                    let units = 0;
                    let t = MusicalTime::new(&ts, bars, beats, parts, units);
                    let frames = t.as_frames(tempo, sample_rate);
                    let t_from_f = MusicalTime {
                        units: MusicalTime::frames_to_units(tempo, sample_rate, frames),
                    };
                    assert_eq!(
                        t, t_from_f,
                        "{:?} - {}.{}.{}.{} -> {frames} -> {:?} <<< PROBLEM",
                        t, bars, beats, parts, units, t_from_f
                    );
                }
            }
        }
    }

    #[test]
    fn musical_time_math() {
        let ts = TimeSignature::default();
        // Advancing by bar works
        let mut t = MusicalTime::default();
        t += MusicalTime::new_with_bars(&ts, 1);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);

        // Advancing by beat works
        let mut t = MusicalTime::default();
        t += MusicalTime::ONE_BEAT;
        assert_eq!(t.beats(&ts), 1);
        let mut t = MusicalTime::new(&ts, 0, ts.top - 1, 0, 0);
        t += MusicalTime::ONE_BEAT;
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);

        // Advancing by part works
        let mut t = MusicalTime::default();
        t += MusicalTime::new_with_parts(1);
        assert_eq!(t.bars(&ts), 0);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.parts(), 1);
        let mut t = MusicalTime::new(&ts, 0, 0, MusicalTime::PARTS_IN_BEAT - 1, 0);
        t += MusicalTime::new_with_parts(1);
        assert_eq!(t.bars(&ts), 0);
        assert_eq!(t.beats(&ts), 1);
        assert_eq!(t.parts(), 0);

        // Advancing by subpart works
        let mut t = MusicalTime::default();
        t += MusicalTime::ONE_UNIT;
        assert_eq!(t.bars(&ts), 0);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.units(), 1);
        let mut t = MusicalTime::new(&ts, 0, 0, 0, MusicalTime::UNITS_IN_PART - 1);
        t += MusicalTime::ONE_UNIT;
        assert_eq!(t.bars(&ts), 0);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.parts(), 1);
        assert_eq!(t.units(), 0);

        // One more big rollover to be sure
        let mut t = MusicalTime::new(&ts, 0, 3, 15, MusicalTime::UNITS_IN_PART - 1);
        t += MusicalTime::ONE_UNIT;
        assert_eq!(t.bars(&ts), 1);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.units(), 0);
    }

    #[test]
    fn musical_time_math_add_trait() {
        let ts = TimeSignature::default();

        let bar_unit = MusicalTime::new(&ts, 1, 0, 0, 0);
        let beat_unit = MusicalTime::new(&ts, 0, 1, 0, 0);
        let part_unit = MusicalTime::new(&ts, 0, 0, 1, 0);
        let unit_unit = MusicalTime::new(&ts, 0, 0, 0, 1);

        // Advancing by bar works
        let t = MusicalTime::default() + bar_unit;
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);

        // Advancing by beat works
        let mut t = MusicalTime::default() + beat_unit;

        assert_eq!(t.beats(&ts), 1);
        t += beat_unit;
        assert_eq!(t.beats(&ts), 2);
        assert_eq!(t.bars(&ts), 0);
        t += beat_unit;
        assert_eq!(t.beats(&ts), 3);
        assert_eq!(t.bars(&ts), 0);
        t += beat_unit;
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);

        // Advancing by part works
        let mut t = MusicalTime::default();
        assert_eq!(t.bars(&ts), 0);
        assert_eq!(t.beats(&ts), 0);
        for i in 0..MusicalTime::PARTS_IN_BEAT {
            assert_eq!(t.parts(), i);
            t += part_unit;
        }
        assert_eq!(t.beats(&ts), 1);
        assert_eq!(t.parts(), 0);

        // Advancing by unit works
        let mut t = MusicalTime::default();
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 0);
        assert_eq!(t.parts(), 0);
        for i in 0..MusicalTime::UNITS_IN_PART {
            assert_eq!(t.units(), i);
            t += unit_unit;
        }
        assert_eq!(t.parts(), 1);
        assert_eq!(t.units(), 0);

        // One more big rollover to be sure
        let mut t = MusicalTime::new(
            &ts,
            0,
            3,
            MusicalTime::PARTS_IN_BEAT - 1,
            MusicalTime::UNITS_IN_PART - 1,
        );
        t += unit_unit;
        assert_eq!(t.bars(&ts), 1);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.units(), 0);
    }

    #[test]
    fn musical_time_math_other_time_signatures() {
        let ts = TimeSignature { top: 9, bottom: 64 };
        let t = MusicalTime::new(&ts, 0, 8, 15, 4095) + MusicalTime::new(&ts, 0, 0, 0, 1);
        assert_eq!(t.bars(&ts), 1);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.units(), 0);
    }

    #[test]
    fn musical_time_overflow() {
        let ts = TimeSignature::new_with(4, 256).unwrap();

        let time = MusicalTime::new(
            &ts,
            0,
            ts.top - 1,
            MusicalTime::PARTS_IN_BEAT - 1,
            MusicalTime::UNITS_IN_PART - 1,
        );

        let t = time + MusicalTime::ONE_BEAT;
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);

        let t = time + MusicalTime::new_with_parts(1);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);

        let t = time + MusicalTime::ONE_UNIT;
        assert_eq!(t.units(), 0);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);
    }

    #[test]
    fn advances_time_correctly_with_various_sample_rates() {
        let mut transport = Transport::default();
        transport.update_tempo(Tempo(60.0));

        let vec = vec![100, 997, 22050, 44100, 48000, 88200, 98689, 100000, 262144];
        for sample_rate in vec {
            transport.play();
            transport.update_sample_rate(SampleRate(sample_rate));

            let mut time_range_covered = 0;
            for _ in 0..transport.sample_rate().0 {
                let range = transport.advance(1);
                let delta_units = (range.0.end - range.0.start).total_units();
                time_range_covered += delta_units;
            }
            assert_eq!(time_range_covered, MusicalTime::UNITS_IN_BEAT,
            "Sample rate {} Hz: after advancing one second of frames at 60 BPM, we should have covered {} MusicalTime units",
            sample_rate, MusicalTime::UNITS_IN_BEAT);

            assert_eq!(
                transport.current_time(),
                MusicalTime::ONE_BEAT,
                "Transport should be exactly on the one-beat mark."
            );

            // We put this at the end of the loop rather than the start because
            // we'd like to test that the initial post-new state is correct
            // without first calling skip_to_start().
            transport.skip_to_start();
        }
    }

    #[test]
    fn sample_rate_math() {
        let sample_rate = SampleRate::from(44100);
        let seconds = Seconds::from(2.0);

        assert_eq!(usize::from(sample_rate * seconds), 88200);
    }

    #[test]
    fn transport_is_automatable() {
        let mut t = TransportBuilder::default().build().unwrap();

        assert_eq!(t.tempo(), Tempo::default());

        assert_eq!(
            t.control_index_count(),
            1,
            "Transport should have one automatable parameter"
        );
        const TEMPO_INDEX: ControlIndex = ControlIndex(0);
        assert_eq!(
            t.control_name_for_index(TEMPO_INDEX),
            Some("tempo".to_string()),
            "Transport's parameter name should be 'tempo'"
        );
        t.control_set_param_by_index(TEMPO_INDEX, ControlValue::MAX);
        assert_eq!(t.tempo(), Tempo::from(Tempo::MAX_VALUE));
        t.control_set_param_by_index(TEMPO_INDEX, ControlValue::MIN);
        assert_eq!(t.tempo(), Tempo::from(Tempo::MIN_VALUE));
    }

    #[test]
    fn quantization_basics() {
        let t1p = MusicalTime::ONE_PART;
        let t1pm1 = MusicalTime::ONE_PART - MusicalTime::ONE_UNIT;
        let t1pp1 = MusicalTime::ONE_PART + MusicalTime::ONE_UNIT;
        let t2p = MusicalTime::ONE_PART * 2;
        let t0p = MusicalTime::DURATION_ZERO;

        assert_eq!(
            t1p.quantized(MusicalTime::ONE_UNIT),
            t1p,
            "min quantization should never change result"
        );
        assert_eq!(t1pm1.quantized(MusicalTime::ONE_UNIT), t1pm1);
        assert_eq!(t1pp1.quantized(MusicalTime::ONE_UNIT), t1pp1);

        assert_eq!(
            t1p.quantized(MusicalTime::ONE_PART),
            t1p,
            "quantizing on ordinary unit should choose closer point"
        );
        assert_eq!(t1pm1.quantized(MusicalTime::ONE_PART), t1p);
        assert_eq!(t1pp1.quantized(MusicalTime::ONE_PART), t1p);

        assert_eq!(
            t1p.quantized(t2p),
            t2p,
            "quantizing to larger unit should choose closer point"
        );
        assert_eq!(t1pm1.quantized(t2p), t0p);
        assert_eq!(t1pp1.quantized(t2p), t2p);
    }

    #[test]
    fn quantization_to_time_signature() {
        assert_eq!(
            MusicalTime::DURATION_SIXTEENTH
                .quantized_for_time_signature(&TimeSignature::COMMON_TIME),
            MusicalTime::DURATION_ZERO
        );
        assert_eq!(
            (MusicalTime::DURATION_EIGHTH - MusicalTime::ONE_UNIT)
                .quantized_for_time_signature(&TimeSignature::COMMON_TIME),
            MusicalTime::DURATION_ZERO
        );
        assert_eq!(
            (MusicalTime::DURATION_EIGHTH)
                .quantized_for_time_signature(&TimeSignature::COMMON_TIME),
            MusicalTime::DURATION_QUARTER
        );
        assert_eq!(
            (MusicalTime::DURATION_QUARTER)
                .quantized_for_time_signature(&TimeSignature::COMMON_TIME),
            MusicalTime::DURATION_QUARTER
        );
    }

    #[test]
    fn quantization_to_measure() {
        assert_eq!(
            MusicalTime::DURATION_SIXTEENTH.quantized_to_measure(&TimeSignature::COMMON_TIME),
            MusicalTime::DURATION_ZERO
        );
        assert_eq!(
            MusicalTime::DURATION_WHOLE.quantized_to_measure(&TimeSignature::COMMON_TIME),
            MusicalTime::DURATION_ZERO
        );
        assert_eq!(
            MusicalTime::DURATION_BREVE.quantized_to_measure(&TimeSignature::COMMON_TIME),
            MusicalTime::FOUR_FOUR_MEASURE
        );
        assert_eq!(
            (MusicalTime::DURATION_WHOLE * 3).quantized_to_measure(&TimeSignature::COMMON_TIME),
            MusicalTime::FOUR_FOUR_MEASURE
        );
        assert_eq!(
            (MusicalTime::DURATION_WHOLE * 6).quantized_to_measure(&TimeSignature::COMMON_TIME),
            MusicalTime::FOUR_FOUR_MEASURE * 2
        );
    }
}
