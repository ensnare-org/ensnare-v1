// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{prelude::*, util::Rng};
use core::ops::{Add, Mul, Range, Sub};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use synonym::Synonym;

/// A human-readable description of the parameter being controlled. Not suitable
/// for end-user viewing, but it's good for debugging.
#[derive(Synonym, Serialize, Deserialize)]
pub struct ControlName(pub String);

/// A zero-based index of the entity parameter being controlled. The index is
/// specific to the entity type.
#[derive(Synonym, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ControlIndex(pub usize);
impl Add<usize> for ControlIndex {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

/// A standardized value range (0..=1.0) for Controls/Controllable traits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Display, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ControlValue(pub f64);
#[allow(missing_docs)]
impl ControlValue {
    pub const MIN: Self = Self(0.0);
    pub const MAX: Self = Self(1.0);
}
impl Mul<f64> for ControlValue {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl From<Normal> for ControlValue {
    fn from(value: Normal) -> Self {
        Self(value.0)
    }
}
impl From<ControlValue> for Normal {
    fn from(value: ControlValue) -> Self {
        Self::from(value.0)
    }
}
impl From<BipolarNormal> for ControlValue {
    fn from(value: BipolarNormal) -> Self {
        Self(Normal::from(value).into())
    }
}
impl From<ControlValue> for BipolarNormal {
    fn from(value: ControlValue) -> Self {
        Self::from(Normal::from(value))
    }
}
impl From<usize> for ControlValue {
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}
impl From<ControlValue> for usize {
    fn from(value: ControlValue) -> Self {
        value.0 as usize
    }
}
impl From<u8> for ControlValue {
    fn from(value: u8) -> Self {
        Self(value as f64 / u8::MAX as f64)
    }
}
impl From<ControlValue> for u8 {
    fn from(value: ControlValue) -> Self {
        (value.0 * u8::MAX as f64) as u8
    }
}
impl From<f32> for ControlValue {
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}
impl From<ControlValue> for f32 {
    fn from(value: ControlValue) -> Self {
        value.0 as f32
    }
}
impl From<f64> for ControlValue {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl From<ControlValue> for f64 {
    fn from(value: ControlValue) -> Self {
        value.0
    }
}
impl From<FrequencyHz> for ControlValue {
    fn from(value: FrequencyHz) -> Self {
        FrequencyHz::frequency_to_percent(value.0).into()
    }
}
impl From<ControlValue> for FrequencyHz {
    fn from(value: ControlValue) -> Self {
        Self::percent_to_frequency(Normal::from(value)).into()
    }
}
impl From<bool> for ControlValue {
    fn from(value: bool) -> Self {
        ControlValue(if value { 1.0 } else { 0.0 })
    }
}
impl From<ControlValue> for bool {
    fn from(value: ControlValue) -> Self {
        value.0 != 0.0
    }
}
impl From<Ratio> for ControlValue {
    fn from(value: Ratio) -> Self {
        ControlValue(Normal::from(value).0)
    }
}
impl From<ControlValue> for Ratio {
    fn from(value: ControlValue) -> Self {
        Self::from(Normal::from(value))
    }
}
impl From<Tempo> for ControlValue {
    fn from(value: Tempo) -> Self {
        Self(value.0 / Tempo::MAX_VALUE)
    }
}
impl From<ControlValue> for Tempo {
    fn from(value: ControlValue) -> Self {
        Self(value.0 * Tempo::MAX_VALUE)
    }
}
impl From<StereoSample> for ControlValue {
    fn from(value: StereoSample) -> Self {
        let sample: Sample = value.into();
        sample.into()
    }
}
impl From<Sample> for ControlValue {
    fn from(value: Sample) -> Self {
        Self(value.0)
    }
}
impl From<Seconds> for ControlValue {
    fn from(value: Seconds) -> Self {
        Self(value.0 / 30.0)
    }
}
impl From<ControlValue> for Seconds {
    fn from(value: ControlValue) -> Self {
        Self(value.0 * 30.0)
    }
}
impl Add<ControlValue> for ControlValue {
    type Output = Self;

    fn add(self, rhs: ControlValue) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl Sub<ControlValue> for ControlValue {
    type Output = Self;

    fn sub(self, rhs: ControlValue) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// Represents a target of a source of control events. For example, if the user
/// wanted Lfo 1 to control Synth 2's pan parameter, then Lfo 1 might have a
/// ControlLink(2, 33) (assume that #33 represents the Synth's pan parameter).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ControlLink {
    /// The [Uid] of the entity to be controlled.
    pub uid: Uid,
    /// The index of the entity parameter to be controlled.
    pub param: ControlIndex,
}

/// A newtype that represents how a value should change, usually over time.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ControlRange(pub Range<ControlValue>);
impl ControlRange {
    /// For testing/prototyping
    pub fn random(rng: &mut Rng) -> Self {
        Self(ControlValue(rng.rand_float())..ControlValue(rng.rand_float()))
    }
}
impl From<Range<f32>> for ControlRange {
    fn from(range: Range<f32>) -> Self {
        Self(range.start.into()..range.end.into())
    }
}
impl From<Range<ControlValue>> for ControlRange {
    fn from(range: Range<ControlValue>) -> Self {
        Self(range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usize_ok() {
        let a = usize::MAX;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<usize>>::into(cv));

        let a = usize::MIN;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<usize>>::into(cv));
    }

    #[test]
    fn u8_ok() {
        let a = u8::MAX;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<u8>>::into(cv));

        let a = u8::MIN;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<u8>>::into(cv));
    }

    #[test]
    fn f32_ok() {
        let a = f32::MAX;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<f32>>::into(cv));

        let a = f32::MIN;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<f32>>::into(cv));
    }

    #[test]
    fn f64_ok() {
        let a = 1000000.0f64;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<f64>>::into(cv));

        let a = -1000000.0f64;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<f64>>::into(cv));
    }

    #[test]
    fn normal_ok() {
        let a = Normal::maximum();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<Normal>>::into(cv));

        let a = Normal::minimum();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<Normal>>::into(cv));
    }

    #[test]
    fn bipolar_normal_ok() {
        let a = BipolarNormal::maximum();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<BipolarNormal>>::into(cv));

        let a = BipolarNormal::minimum();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<BipolarNormal>>::into(cv));

        let a = BipolarNormal::zero();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<BipolarNormal>>::into(cv));
    }

    #[test]
    fn bool_ok() {
        let a = true;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<bool>>::into(cv));

        let a = false;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<bool>>::into(cv));
    }

    #[test]
    fn ratio_ok() {
        assert_eq!(Ratio::from(ControlValue(0.0)).0, 0.125);
        assert_eq!(Ratio::from(ControlValue(0.5)).0, 1.0);
        assert_eq!(Ratio::from(ControlValue(1.0)).0, 8.0);

        assert_eq!(ControlValue::from(Ratio::from(0.125)).0, 0.0);
        assert_eq!(ControlValue::from(Ratio::from(1.0)).0, 0.5);
        assert_eq!(ControlValue::from(Ratio::from(8.0)).0, 1.0);

        assert_eq!(Ratio::from(BipolarNormal::from(-1.0)).0, 0.125);
        assert_eq!(Ratio::from(BipolarNormal::from(0.0)).0, 1.0);
        assert_eq!(Ratio::from(BipolarNormal::from(1.0)).0, 8.0);

        assert_eq!(BipolarNormal::from(Ratio::from(0.125)).0, -1.0);
        assert_eq!(BipolarNormal::from(Ratio::from(1.0)).0, 0.0);
        assert_eq!(BipolarNormal::from(Ratio::from(8.0)).0, 1.0);
    }
}
