// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::delay::{AllPassDelayLine, Delays};
use crate::RecirculatingDelayLine;
use ensnare_core::{prelude::*, time::Seconds};
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Schroeder reverb. Uses four parallel recirculating delay lines feeding into
/// a series of two all-pass delay lines.
#[derive(Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Reverb {
    /// How much the effect should attenuate the input.
    #[control]
    attenuation: Normal,

    #[control]
    seconds: Seconds,

    #[serde(skip)]
    sample_rate: SampleRate,

    #[serde(skip)]
    channels: [ReverbChannel; 2],
}
impl Serializable for Reverb {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.channels = [
            ReverbChannel::new_with(self.attenuation, self.seconds),
            ReverbChannel::new_with(self.attenuation, self.seconds),
        ];
    }
}
impl Configurable for Reverb {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.channels[0].update_sample_rate(sample_rate);
        self.channels[1].update_sample_rate(sample_rate);
    }
}
impl TransformsAudio for Reverb {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.channels[channel].transform_channel(channel, input_sample)
    }
}
impl Reverb {
    pub fn new_with(attenuation: Normal, seconds: Seconds) -> Self {
        // Thanks to https://basicsynth.com/ (page 133 of paperback) for
        // constants.
        let mut r = Self {
            attenuation,
            seconds,
            ..Default::default()
        };
        r.after_deser();
        r
    }

    pub fn attenuation(&self) -> Normal {
        self.attenuation
    }

    pub fn set_attenuation(&mut self, attenuation: Normal) {
        self.attenuation = attenuation;
        self.channels
            .iter_mut()
            .for_each(|c| c.set_attenuation(attenuation));
    }

    pub fn seconds(&self) -> Seconds {
        self.seconds
    }

    pub fn set_seconds(&mut self, seconds: Seconds) {
        self.seconds = seconds;
        self.channels
            .iter_mut()
            .for_each(|c| c.set_seconds(seconds));
    }
}

#[derive(Debug, Default)]
struct ReverbChannel {
    attenuation: Normal,

    recirc_delay_lines: Vec<RecirculatingDelayLine>,
    allpass_delay_lines: Vec<AllPassDelayLine>,
}
impl TransformsAudio for ReverbChannel {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let input_attenuated = input_sample * self.attenuation.0;
        let recirc_output = self.recirc_delay_lines[0].pop_output(input_attenuated)
            + self.recirc_delay_lines[1].pop_output(input_attenuated)
            + self.recirc_delay_lines[2].pop_output(input_attenuated)
            + self.recirc_delay_lines[3].pop_output(input_attenuated);
        let adl_0_out = self.allpass_delay_lines[0].pop_output(recirc_output);
        self.allpass_delay_lines[1].pop_output(adl_0_out)
    }
}
impl Serializable for ReverbChannel {}
impl Configurable for ReverbChannel {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.recirc_delay_lines
            .iter_mut()
            .for_each(|r| r.update_sample_rate(sample_rate));
        self.allpass_delay_lines
            .iter_mut()
            .for_each(|r| r.update_sample_rate(sample_rate));
    }
}
impl ReverbChannel {
    pub fn new_with(attenuation: Normal, seconds: Seconds) -> Self {
        // Thanks to https://basicsynth.com/ (page 133 of paperback) for
        // constants.
        Self {
            attenuation,
            recirc_delay_lines: Self::instantiate_recirc_delay_lines(seconds),
            allpass_delay_lines: Self::instantiate_allpass_delay_lines(),
        }
    }

    fn set_attenuation(&mut self, attenuation: Normal) {
        self.attenuation = attenuation;
    }

    fn set_seconds(&mut self, seconds: Seconds) {
        self.recirc_delay_lines = Self::instantiate_recirc_delay_lines(seconds);
    }

    fn instantiate_recirc_delay_lines(seconds: Seconds) -> Vec<RecirculatingDelayLine> {
        vec![
            RecirculatingDelayLine::new_with(
                0.0297.into(),
                seconds,
                Normal::from(0.001),
                Normal::from(1.0),
            ),
            RecirculatingDelayLine::new_with(
                0.0371.into(),
                seconds,
                Normal::from(0.001),
                Normal::from(1.0),
            ),
            RecirculatingDelayLine::new_with(
                0.0411.into(),
                seconds,
                Normal::from(0.001),
                Normal::from(1.0),
            ),
            RecirculatingDelayLine::new_with(
                0.0437.into(),
                seconds,
                Normal::from(0.001),
                Normal::from(1.0),
            ),
        ]
    }

    fn instantiate_allpass_delay_lines() -> Vec<AllPassDelayLine> {
        vec![
            AllPassDelayLine::new_with(
                0.09683.into(),
                0.0050.into(),
                Normal::from(0.001),
                Normal::from(1.0),
            ),
            AllPassDelayLine::new_with(
                0.03292.into(),
                0.0017.into(),
                Normal::from(0.001),
                Normal::from(1.0),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_SAMPLE_RATE: usize = 44100;

    #[test]
    fn reverb_does_anything_at_all() {
        // This test is lame, because I can't think of a programmatic way to
        // test that reverb works. I observed that with the Schroeder reverb set
        // to 0.5 seconds, we start getting back nonzero samples (first
        // 0.47767496) at samples: 29079, seconds: 0.65938777. This doesn't look
        // wrong, but I couldn't have predicted that exact number.
        let mut fx = Reverb::new_with(Normal::from(0.9), 0.5.into());
        fx.update_sample_rate(SampleRate::DEFAULT);
        assert_eq!(fx.transform_channel(0, Sample::from(0.8)), Sample::SILENCE);
        let mut s = Sample::default();
        for _ in 0..DEFAULT_SAMPLE_RATE {
            s += fx.transform_channel(0, Sample::SILENCE);
        }
        assert!(s != Sample::SILENCE);

        // TODO: this test might not do anything. I refactored it in a hurry and
        // took something that looked critical (skipping the clock to 0.5
        // seconds) out of it, but it still passed. I might not actually be
        // testing anything useful.
    }
}
