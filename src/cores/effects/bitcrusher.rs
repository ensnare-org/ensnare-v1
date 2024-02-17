// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derivative::Derivative;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// TODO: this is a pretty lame bitcrusher. It is hardly noticeable for values
/// below 13, and it destroys the waveform at 15. It doesn't do any simulation
/// of sample-rate reduction, either.
#[derive(Debug, Builder, Derivative, Control, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BitcrusherCore {
    /// The number of bits to preserve
    #[control]
    #[derivative(Default(value = "8"))]
    bits: u8,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BitcrusherCoreEphemerals,
}
#[derive(Debug, Default)]
pub struct BitcrusherCoreEphemerals {
    /// A cached representation of `bits` for optimization.
    bits_cached: SampleType,

    c: Configurables,
}
impl BitcrusherCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<BitcrusherCore, BitcrusherCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl TransformsAudio for BitcrusherCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        const I16_SCALE: SampleType = i16::MAX as SampleType;
        let sign = input_sample.0.signum();
        let input = (input_sample * I16_SCALE).0.abs();
        (((input / self.e.bits_cached).floor() * self.e.bits_cached / I16_SCALE) * sign).into()
    }
}
impl Configurable for BitcrusherCore {
    delegate! {
        to self.e.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
#[allow(missing_docs)]
impl BitcrusherCore {
    pub fn bits(&self) -> u8 {
        self.bits
    }

    pub fn set_bits(&mut self, n: u8) {
        self.bits = n;
        self.update_cache();
    }

    fn update_cache(&mut self) {
        self.e.bits_cached = 2.0f64.powi(self.bits() as i32);
    }

    // TODO - write a custom type for range 0..16

    pub fn bits_range() -> core::ops::RangeInclusive<u8> {
        0..=16
    }
}
impl Serializable for BitcrusherCore {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.update_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::PI;

    const CRUSHED_PI: SampleType = 0.14062929166539506;

    #[test]
    fn bitcrusher_basic() {
        let mut fx = BitcrusherCoreBuilder::default().build().unwrap();
        assert_eq!(
            fx.transform_channel(0, Sample(PI - 3.0)),
            Sample(CRUSHED_PI)
        );
    }

    #[test]
    fn bitcrusher_no_bias() {
        let mut fx = BitcrusherCoreBuilder::default().build().unwrap();
        assert_eq!(
            fx.transform_channel(0, Sample(-(PI - 3.0))),
            Sample(-CRUSHED_PI)
        );
    }
}
