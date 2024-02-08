// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Builder, Control, Serialize, Deserialize)]
#[builder(default)]
#[serde(rename_all = "kebab-case")]
pub struct CompressorCore {
    /// The level above which compression takes effect. Range is 0.0..=1.0, 0.0
    /// corresponds to quietest, and 1.0 corresponds to 0dB.
    #[control]
    threshold: Normal,

    /// How much to compress the audio above the threshold. For example, 2:1
    /// means that a 2dB input increase leads to a 1dB output increase. Note
    /// that this value is actually the inverted ratio, so that 2:1 is 0.5 (1
    /// divided by 2), and 1:4 is 0.25 (1 divided by 4). Thus, 1.0 means no
    /// compression, and 0.0 is infinite compression (the output remains a
    /// constant amplitude no matter what).
    #[control]
    ratio: Ratio,

    /// How soon the compressor activates after the level exceeds the threshold.
    /// Expressed as a [Normal] that is scaled to an amount of time.
    #[control]
    attack: Normal,

    /// How soon the compressor deactivates after the level drops below the
    /// Expressed as a [Normal] that is scaled to an amount of time.
    #[control]
    release: Normal,

    #[serde(skip)]
    #[builder(setter(skip))]
    c: Configurables,
}
impl Serializable for CompressorCore {}
impl TransformsAudio for CompressorCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let input_sample_positive = input_sample.0.abs();
        let threshold = self.threshold.0;
        if input_sample_positive > threshold {
            // TODO: this expression is (a + b - a) * c * d, which is just b * c
            // * d, which is clearly wrong. Fix it. (Too tired right now to look
            //   into how compression should work)
            Sample::from(
                (threshold + (input_sample_positive - threshold) * self.ratio)
                    * input_sample.0.signum(),
            )
        } else {
            input_sample
        }
    }
}
impl Configurable for CompressorCore {
    delegate! {
        to self.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl CompressorCore {
    pub fn threshold(&self) -> Normal {
        self.threshold
    }

    pub fn set_threshold(&mut self, threshold: Normal) {
        self.threshold = threshold;
    }

    pub fn ratio(&self) -> Ratio {
        self.ratio
    }

    pub fn set_ratio(&mut self, ratio: Ratio) {
        self.ratio = ratio;
    }

    pub fn attack(&self) -> Normal {
        self.attack
    }

    pub fn set_attack(&mut self, attack: Normal) {
        self.attack = attack;
    }

    pub fn release(&self) -> Normal {
        self.release
    }

    pub fn set_release(&mut self, release: Normal) {
        self.release = release;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_compressor() {
        const THRESHOLD: SampleType = 0.25;
        let mut fx = CompressorCoreBuilder::default()
            .threshold(THRESHOLD.into())
            .ratio(0.5.into())
            .attack(0.0.into())
            .release(0.0.into())
            .build()
            .unwrap();
        assert_eq!(
            fx.transform_channel(0, Sample::from(0.35)),
            Sample::from((0.35 - THRESHOLD) * 0.5 + THRESHOLD)
        );
    }

    #[test]
    fn nothing_compressor() {
        let mut fx = CompressorCoreBuilder::default()
            .threshold(0.25.into())
            .ratio(1.0.into())
            .attack(0.0.into())
            .release(0.0.into())
            .build()
            .unwrap();
        assert_eq!(
            fx.transform_channel(0, Sample::from(0.35f32)),
            Sample::from(0.35f32)
        );
    }

    #[test]
    fn infinite_compressor() {
        let mut fx = CompressorCoreBuilder::default()
            .threshold(0.25.into())
            .ratio(0.0.into())
            .attack(0.0.into())
            .release(0.0.into())
            .build()
            .unwrap();
        assert_eq!(
            fx.transform_channel(0, Sample::from(0.35)),
            Sample::from(0.25)
        );
    }
}
