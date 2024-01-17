// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// TODO: this is a pretty lame bitcrusher. It is hardly noticeable for values
/// below 13, and it destroys the waveform at 15. It doesn't do any simulation
/// of sample-rate reduction, either.
#[derive(Debug, Control, Serialize, Deserialize)]
pub struct Bitcrusher {
    /// The number of bits to preserve
    #[control]
    bits: u8,

    /// A cached representation of `bits` for optimization.
    #[serde(skip)]
    c: SampleType,
}
impl Default for Bitcrusher {
    fn default() -> Self {
        Self::new_with(8)
    }
}
impl TransformsAudio for Bitcrusher {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        const I16_SCALE: SampleType = i16::MAX as SampleType;
        let sign = input_sample.0.signum();
        let input = (input_sample * I16_SCALE).0.abs();
        (((input / self.c).floor() * self.c / I16_SCALE) * sign).into()
    }
}
impl Configurable for Bitcrusher {}
#[allow(missing_docs)]
impl Bitcrusher {
    pub fn new_with(bits: u8) -> Self {
        let mut r = Self {
            bits,
            c: Default::default(),
        };
        r.update_c();
        r
    }

    pub fn bits(&self) -> u8 {
        self.bits
    }

    pub fn set_bits(&mut self, n: u8) {
        self.bits = n;
        self.update_c();
    }

    fn update_c(&mut self) {
        self.c = 2.0f64.powi(self.bits() as i32);
    }

    // TODO - write a custom type for range 0..16

    pub fn bits_range() -> std::ops::RangeInclusive<u8> {
        0..=16
    }
}
impl Serializable for Bitcrusher {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.update_c();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const CRUSHED_PI: SampleType = 0.14062929166539506;

    #[test]
    fn bitcrusher_basic() {
        let mut fx = Bitcrusher::new_with(8);
        assert_eq!(
            fx.transform_channel(0, Sample(PI - 3.0)),
            Sample(CRUSHED_PI)
        );
    }

    #[test]
    fn bitcrusher_no_bias() {
        let mut fx = Bitcrusher::new_with(8);
        assert_eq!(
            fx.transform_channel(0, Sample(-(PI - 3.0))),
            Sample(-CRUSHED_PI)
        );
    }
}
