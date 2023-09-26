// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{prelude::*, traits::prelude::*};
use eframe::egui::Ui;
use ensnare_proc_macros::{Control, IsEffect, Params, Uid};
use serde::{Deserialize, Serialize};

pub(crate) trait Delays {
    fn peek_output(&self, apply_decay: bool) -> Sample;
    fn peek_indexed_output(&self, index: isize) -> Sample;
    fn pop_output(&mut self, input: Sample) -> Sample;
}

#[derive(Debug, Default)]
pub(crate) struct DelayLine {
    sample_rate: SampleRate,
    delay_seconds: ParameterType,
    decay_factor: SignalType,

    buffer_size: usize,
    buffer_pointer: usize,
    buffer: Vec<Sample>,
}
impl Configurable for DelayLine {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.resize_buffer();
    }
}
impl DelayLine {
    /// decay_factor: 1.0 = no decay
    pub(super) fn new_with(delay_seconds: ParameterType, decay_factor: SignalType) -> Self {
        Self {
            sample_rate: Default::default(),
            delay_seconds,
            decay_factor,

            buffer_size: Default::default(),
            buffer_pointer: 0,
            buffer: Default::default(),
        }
    }

    pub(super) fn delay_seconds(&self) -> ParameterType {
        self.delay_seconds
    }

    pub(super) fn set_delay_seconds(&mut self, delay_seconds: ParameterType) {
        if delay_seconds != self.delay_seconds {
            self.delay_seconds = delay_seconds;
            self.resize_buffer();
        }
    }

    fn resize_buffer(&mut self) {
        self.buffer_size =
            (self.sample_rate.value() as ParameterType * self.delay_seconds) as usize;
        self.buffer = Vec::with_capacity(self.buffer_size);
        self.buffer.resize(self.buffer_size, Sample::SILENCE);
        self.buffer_pointer = 0;
    }

    pub(super) fn decay_factor(&self) -> SignalType {
        self.decay_factor
    }
}
impl Delays for DelayLine {
    fn peek_output(&self, apply_decay: bool) -> Sample {
        if self.buffer_size == 0 {
            Sample::SILENCE
        } else if apply_decay {
            self.buffer[self.buffer_pointer] * self.decay_factor()
        } else {
            self.buffer[self.buffer_pointer]
        }
    }

    fn peek_indexed_output(&self, index: isize) -> Sample {
        if self.buffer_size == 0 {
            Sample::SILENCE
        } else {
            let mut index = -index;
            while index < 0 {
                index += self.buffer_size as isize;
            }
            self.buffer[self.buffer_pointer]
        }
    }

    fn pop_output(&mut self, input: Sample) -> Sample {
        if self.buffer_size == 0 {
            input
        } else {
            let out = self.peek_output(true);
            self.buffer[self.buffer_pointer] = input;
            self.buffer_pointer += 1;
            if self.buffer_pointer >= self.buffer_size {
                self.buffer_pointer = 0;
            }
            out
        }
    }
}

#[derive(Debug, Default)]
pub struct RecirculatingDelayLine {
    delay: DelayLine,
}
impl RecirculatingDelayLine {
    pub(crate) fn new_with(
        delay_seconds: ParameterType,
        decay_seconds: ParameterType,
        final_amplitude: Normal,
        peak_amplitude: Normal,
    ) -> Self {
        Self {
            delay: DelayLine::new_with(
                delay_seconds,
                (peak_amplitude.value() * final_amplitude.value())
                    .powf(delay_seconds / decay_seconds) as SignalType,
            ),
        }
    }

    pub(super) fn decay_factor(&self) -> SignalType {
        self.delay.decay_factor()
    }
}
impl Delays for RecirculatingDelayLine {
    fn peek_output(&self, apply_decay: bool) -> Sample {
        self.delay.peek_output(apply_decay)
    }

    fn peek_indexed_output(&self, index: isize) -> Sample {
        self.delay.peek_indexed_output(index)
    }

    fn pop_output(&mut self, input: Sample) -> Sample {
        let output = self.peek_output(true);
        self.delay.pop_output(input + output);
        output
    }
}
impl Configurable for RecirculatingDelayLine {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.delay.update_sample_rate(sample_rate);
    }
}

#[derive(Debug, Default)]
pub(crate) struct AllPassDelayLine {
    delay: RecirculatingDelayLine,
}
impl AllPassDelayLine {
    pub(crate) fn new_with(
        delay_seconds: ParameterType,
        decay_seconds: ParameterType,
        final_amplitude: Normal,
        peak_amplitude: Normal,
    ) -> Self {
        Self {
            delay: RecirculatingDelayLine::new_with(
                delay_seconds,
                decay_seconds,
                final_amplitude,
                peak_amplitude,
            ),
        }
    }
}
impl Delays for AllPassDelayLine {
    fn peek_output(&self, _apply_decay: bool) -> Sample {
        panic!("AllPassDelay doesn't allow peeking")
    }

    fn peek_indexed_output(&self, _: isize) -> Sample {
        panic!("AllPassDelay doesn't allow peeking")
    }

    fn pop_output(&mut self, input: Sample) -> Sample {
        let decay_factor = self.delay.decay_factor();
        let vm = self.delay.peek_output(false);
        let vn = input - (vm * decay_factor);
        self.delay.pop_output(vn);
        vm + vn * decay_factor
    }
}
impl Configurable for AllPassDelayLine {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.delay.update_sample_rate(sample_rate)
    }
}

#[derive(Debug, Default, Control, IsEffect, Params, Uid, Serialize, Deserialize)]
pub struct Delay {
    uid: Uid,

    #[control]
    #[params]
    seconds: ParameterType,

    #[serde(skip)]
    delay: DelayLine,
}
impl Serializable for Delay {}
impl Configurable for Delay {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.delay.update_sample_rate(sample_rate);
    }
}
impl TransformsAudio for Delay {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        self.delay.pop_output(input_sample)
    }
}
impl Delay {
    #[allow(dead_code)]
    fn new() -> Self {
        Self::default()
    }

    pub fn new_with(params: &DelayParams) -> Self {
        Self {
            seconds: params.seconds(),
            delay: DelayLine::new_with(params.seconds(), 1.0),
            ..Default::default()
        }
    }

    pub fn seconds(&self) -> ParameterType {
        self.delay.delay_seconds()
    }

    pub fn set_seconds(&mut self, seconds: ParameterType) {
        self.delay.set_delay_seconds(seconds);
    }
}
impl Displays for Delay {
    fn ui(&mut self, ui: &mut Ui) -> eframe::egui::Response {
        ui.label(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;
    use more_asserts::{assert_gt, assert_lt};

    // This small rate allows us to observe expected behavior after a small
    // number of iterations.
    const CURIOUSLY_SMALL_SAMPLE_RATE: SampleRate = SampleRate::new(3);

    #[test]
    fn basic_delay() {
        let mut fx = Delay::new_with(&DelayParams { seconds: 1.0 });
        fx.update_sample_rate(SampleRate::DEFAULT);

        // Add a unique first sample.
        assert_eq!(fx.transform_channel(0, Sample::from(0.5)), Sample::SILENCE);

        // Push a whole bunch more.
        for i in 0..SampleRate::DEFAULT_SAMPLE_RATE - 1 {
            assert_eq!(
                fx.transform_channel(0, Sample::MAX),
                Sample::SILENCE,
                "unexpected value at sample {}",
                i
            );
        }

        // We should get back our first sentinel sample.
        assert_eq!(fx.transform_channel(0, Sample::SILENCE), Sample::from(0.5));

        // And the next should be one of the bunch.
        assert_eq!(fx.transform_channel(0, Sample::SILENCE), Sample::MAX);
    }

    #[test]
    fn delay_zero() {
        let mut fx = Delay::new_with(&DelayParams { seconds: 0.0 });
        fx.update_sample_rate(SampleRate::DEFAULT);

        // We should keep getting back what we put in.
        let mut rng = oorandom::Rand32::new(0);
        for i in 0..SampleRate::DEFAULT_SAMPLE_RATE {
            let random_bipolar_normal = rng.rand_float() * 2.0 - 1.0;
            let sample = Sample::from(random_bipolar_normal);
            assert_eq!(
                fx.transform_channel(0, sample),
                sample,
                "unexpected value at sample {}",
                i
            );
        }
    }

    #[test]
    fn delay_line() {
        // It's very simple: it should return an input sample, attenuated, after
        // the specified delay.
        let mut delay = DelayLine::new_with(1.0, 0.3);
        delay.update_sample_rate(CURIOUSLY_SMALL_SAMPLE_RATE);

        assert_eq!(delay.pop_output(Sample::from(0.5)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.4)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.3)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.2)), Sample::from(0.5 * 0.3));
    }

    #[test]
    fn recirculating_delay_line() {
        // Recirculating means that the input value is added to the value at the
        // back of the buffer, rather than replacing that value. So if we put in
        // a single value, we should expect to get it back, usually quieter,
        // each time it cycles through the buffer.
        let mut delay =
            RecirculatingDelayLine::new_with(1.0, 1.5, Normal::from(0.001), Normal::from(1.0));
        delay.update_sample_rate(CURIOUSLY_SMALL_SAMPLE_RATE);

        assert_eq!(delay.pop_output(Sample::from(0.5)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert!(approx_eq!(
            SampleType,
            delay.pop_output(Sample::from(0.0)).0,
            Sample::from(0.5 * 0.01).0,
            epsilon = 0.001
        ));
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert!(approx_eq!(
            SampleType,
            delay.pop_output(Sample::from(0.0)).0,
            Sample::from(0.5 * 0.01 * 0.01).0,
            epsilon = 0.001
        ));
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
    }

    #[test]
    fn allpass_delay_line() {
        // TODO: I'm not sure what this delay line is supposed to do.
        let mut delay =
            AllPassDelayLine::new_with(1.0, 1.5, Normal::from(0.001), Normal::from(1.0));
        delay.update_sample_rate(CURIOUSLY_SMALL_SAMPLE_RATE);

        assert_lt!(delay.pop_output(Sample::from(0.5)), Sample::from(0.5));
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_gt!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE); // Note! > not =
    }
}