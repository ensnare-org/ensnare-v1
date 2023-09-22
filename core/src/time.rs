// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    core::ParameterType,
    midi::HandlesMidi,
    traits::{Configurable, ControlEventsFn, Controls, Displays, Serializable},
    uid::Uid,
};
use anyhow::{anyhow, Error};
use derive_builder::Builder;
use derive_more::Display;
use eframe::{
    egui::{Frame, Margin, Ui},
    epaint::{Color32, Stroke, Vec2},
};
use ensnare_proc_macros::{Control, IsController, Uid};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Mul, Range, Sub, SubAssign},
};
use strum_macros::{FromRepr, IntoStaticStr};

/// Beats per minute.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tempo(pub ParameterType);
impl Default for Tempo {
    fn default() -> Self {
        Self(128.0)
    }
}
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

    #[allow(missing_docs)]
    /// A getter for the raw value.
    pub fn value(&self) -> ParameterType {
        self.0
    }
    /// Beats per second.
    pub fn bps(&self) -> ParameterType {
        self.0 / 60.0
    }
}

/// [BeatValue] enumerates numerical divisors used in most music.  
#[derive(Clone, Debug, Default, FromRepr, IntoStaticStr, Serialize, Deserialize)]
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
impl Displays for BeatValue {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.allocate_ui(Vec2::new(60.0, 24.0), |ui| {
            Self::show_beat_value(ui, &format!("{} beats", BeatValue::divisor(self.clone())));
        })
        .response
    }
}
impl BeatValue {
    fn show_beat_value(ui: &mut Ui, label: &str) {
        Frame::none()
            .stroke(Stroke::new(2.0, Color32::GRAY))
            .fill(Color32::DARK_GRAY)
            .inner_margin(Margin::same(2.0))
            .outer_margin(Margin {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 5.0,
            })
            .show(ui, |ui| {
                ui.label(label);
            });
    }

    #[allow(missing_docs)]
    pub fn show_inherited(ui: &mut Ui) {
        Self::show_beat_value(ui, "inherited");
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
#[derive(Clone, Control, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimeSignature {
    /// The number of beats in a measure.
    #[control]
    pub top: usize,

    /// The value of a beat. Expressed as a reciprocal; for example, if it's 4,
    /// then the beat value is 1/4 or a quarter note.
    #[control]
    pub bottom: usize,
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
impl Default for TimeSignature {
    fn default() -> Self {
        Self { top: 4, bottom: 4 }
    }
}

/// [MusicalTime] is the universal unit of time. It is in terms of musical
/// beats. A "part" is a sixteenth of a beat, and a "unit" is 1/4096 of a part.
/// Thus, beats are divided into 65,536 units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
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

    pub const START: MusicalTime = MusicalTime { units: 0 };

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
    pub fn total_units(&self) -> usize {
        self.units
    }

    pub fn units(&self) -> usize {
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
        let elapsed_beats = (frames as f64 / sample_rate.value() as f64) * tempo.bps();
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
    pub fn empty_range() -> Range<Self> {
        Range {
            start: Self::TIME_MAX,
            end: Self::TIME_MAX,
        }
    }

    pub fn as_frames(&self, tempo: Tempo, sample_rate: SampleRate) -> usize {
        Self::units_to_frames(tempo, sample_rate, self.units)
    }

    // Actually just chopped to nearest part for now
    pub fn quantized(&self) -> MusicalTime {
        MusicalTime::new_with_parts(self.total_parts())
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

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
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
impl Add<f64> for Seconds {
    type Output = Seconds;

    fn add(self, rhs: f64) -> Self::Output {
        Seconds(self.0 + rhs)
    }
}
impl Add<Seconds> for Seconds {
    type Output = Seconds;

    fn add(self, rhs: Seconds) -> Self::Output {
        Seconds(self.0 + rhs.0)
    }
}

/// Samples per second. Always a positive integer; cannot be zero.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Display, PartialEq, Eq)]
pub struct SampleRate(pub usize);
#[allow(missing_docs)]
impl SampleRate {
    pub const DEFAULT_SAMPLE_RATE: usize = 44100;
    pub const DEFAULT: SampleRate = SampleRate::new(Self::DEFAULT_SAMPLE_RATE);

    pub const fn value(&self) -> usize {
        self.0
    }

    pub const fn new(value: usize) -> Self {
        if value != 0 {
            Self(value)
        } else {
            Self(Self::DEFAULT_SAMPLE_RATE)
        }
    }
}
impl Default for SampleRate {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SAMPLE_RATE)
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

/// Parts of [Transport] that shouldn't be serialized.
#[derive(Debug, Clone, Default)]
pub struct TransportEphemerals {
    /// The global time pointer within the song.
    current_time: MusicalTime,

    current_frame: usize,

    sample_rate: SampleRate,

    is_performing: bool,
}

/// [Transport] is the global clock. It keeps track of the current position in
/// the song, and how time should advance.

#[derive(Serialize, Deserialize, Clone, Control, IsController, Debug, Default, Uid, Builder)]
pub struct Transport {
    /// The entity's uid.
    uid: Uid,

    /// The current global time signature.
    #[builder(default)]
    time_signature: TimeSignature,

    /// The current beats per minute.
    #[builder(default)]
    pub(crate) tempo: Tempo,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: TransportEphemerals,
}
impl HandlesMidi for Transport {}
impl Transport {
    /// Returns the current [Tempo].
    pub fn tempo(&self) -> Tempo {
        self.tempo
    }

    /// Sets a new [Tempo].
    pub fn set_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo;
    }

    /// Advances the clock by the given number of frames. Returns the time range
    /// from the prior time to now.
    pub fn advance(&mut self, frames: usize) -> Range<MusicalTime> {
        // Calculate the work time range. Note that the range can be zero, which
        // will happen if frames advance faster than MusicalTime units.
        let new_frames = self.e.current_frame + frames;
        let new_time = MusicalTime::new_with_frames(self.tempo, self.e.sample_rate, new_frames);
        let length = new_time - self.e.current_time;
        let range = self.e.current_time..self.e.current_time + length;

        // If we aren't performing, then we don't advance the clock, but we do
        // give devices the appearance of time moving forward by providing them
        // a (usually) nonzero time range.
        //
        // This is another reason why devices will sometimes get the same time
        // range twice. It's also why very high sample rates will make
        // MusicalTime inaccurate for devices like an arpeggiator that depend on
        // this time source *and* are supposed to operate interactively while
        // not performing (performance is stopped, but a MIDI track is selected,
        // and the user expects to hear the arp respond normally to MIDI
        // keyboard events). TODO: define a better way for these kinds of
        // devices; maybe they need a different clock that genuinely moves
        // forward (except when the performance starts). It should share the
        // same origin as the real clock, but increases regardless of
        // performance status.
        if self.is_performing() {
            self.e.current_frame = new_frames;
            self.e.current_time = new_time;
        }
        range
    }

    #[allow(missing_docs)]
    pub fn current_time(&self) -> MusicalTime {
        self.e.current_time
    }
}
impl Displays for Transport {
    fn ui(&mut self, _ui: &mut Ui) -> eframe::egui::Response {
        unimplemented!("use transport widget instead")
    }
}
impl Serializable for Transport {}
impl Configurable for Transport {
    fn sample_rate(&self) -> SampleRate {
        self.e.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.e.sample_rate = sample_rate;
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo;
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
    }
}
impl Controls for Transport {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        // Nothing - we calculated the range, so we don't need to do anything with it.
        debug_assert!(
            self.e.current_time == range.end,
            "Transport::update_time() was called with the range ..{} but current_time is {}",
            range.end,
            self.e.current_time
        );
    }

    fn work(&mut self, _control_events_fn: &mut ControlEventsFn) {
        // nothing, but in the future we might want to propagate a tempo or time-sig change
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.e.is_performing = true;
    }

    fn stop(&mut self) {
        self.e.is_performing = false;
    }

    fn skip_to_start(&mut self) {
        self.e.current_time = MusicalTime::default();
        self.e.current_frame = Default::default();
    }

    fn is_performing(&self) -> bool {
        self.e.is_performing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tempo() {
        let t = Tempo::default();
        assert_eq!(t.value(), 128.0);
    }

    #[test]
    fn sample_rate_default_is_sane() {
        let sr = SampleRate::default();
        assert_eq!(sr.value(), 44100);
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
        t += MusicalTime::new_with_beats(1);
        assert_eq!(t.beats(&ts), 1);
        let mut t = MusicalTime::new(&ts, 0, ts.top - 1, 0, 0);
        t += MusicalTime::new_with_beats(1);
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
        t += MusicalTime::new_with_units(1);
        assert_eq!(t.bars(&ts), 0);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.units(), 1);
        let mut t = MusicalTime::new(&ts, 0, 0, 0, MusicalTime::UNITS_IN_PART - 1);
        t += MusicalTime::new_with_units(1);
        assert_eq!(t.bars(&ts), 0);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.parts(), 1);
        assert_eq!(t.units(), 0);

        // One more big rollover to be sure
        let mut t = MusicalTime::new(&ts, 0, 3, 15, MusicalTime::UNITS_IN_PART - 1);
        t += MusicalTime::new_with_units(1);
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

        let t = time + MusicalTime::new_with_beats(1);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);

        let t = time + MusicalTime::new_with_parts(1);
        assert_eq!(t.parts(), 0);
        assert_eq!(t.beats(&ts), 0);
        assert_eq!(t.bars(&ts), 1);

        let t = time + MusicalTime::new_with_units(1);
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
                let delta_units = (range.end - range.start).total_units();
                time_range_covered += delta_units;
            }
            assert_eq!(time_range_covered, MusicalTime::UNITS_IN_BEAT,
            "Sample rate {} Hz: after advancing one second of frames at 60 BPM, we should have covered {} MusicalTime units",
            sample_rate, MusicalTime::UNITS_IN_BEAT);

            assert_eq!(
                transport.current_time(),
                MusicalTime::new_with_beats(1),
                "Transport should be exactly on the one-beat mark."
            );

            // We put this at the end of the loop rather than the start because
            // we'd like to test that the initial post-new state is correct
            // without first calling skip_to_start().
            transport.skip_to_start();
        }
    }
}