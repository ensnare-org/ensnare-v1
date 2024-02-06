// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Add, Mul, MulAssign, RangeInclusive, Sub},
};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
