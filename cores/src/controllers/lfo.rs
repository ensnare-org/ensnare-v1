// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{generators::Oscillator, prelude::*};
use ensnare_proc_macros::Control;

/// Uses an internal LFO as a control source.
#[derive(Debug, Default, Control)]
pub struct LfoController {
    #[control]
    pub oscillator: Oscillator,

    is_performing: bool,

    time_range: TimeRange,

    last_frame: usize,
}
impl Serializable for LfoController {}
impl Configurable for LfoController {
    fn sample_rate(&self) -> SampleRate {
        self.oscillator.sample_rate()
    }
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.oscillator.update_sample_rate(sample_rate);
    }
}
impl Controls for LfoController {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.time_range = range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        let frames = self
            .time_range
            .0
            .start
            .as_frames(Tempo::from(120), self.oscillator.sample_rate());

        if frames != self.last_frame {
            let tick_count = if frames >= self.last_frame {
                // normal case; oscillator should advance the calculated number
                // of frames
                //
                // TODO: this is unlikely to be frame-accurate, because
                // Orchestrator is currently going from frames -> beats
                // (inaccurate), and then we're going from beats -> frames. We
                // could include frame count in update_time(), as discussed in
                // #132, which would mean we don't have to be smart at all about
                // it.
                frames - self.last_frame
            } else {
                self.last_frame = frames;
                0
            };
            self.last_frame += tick_count;
            self.oscillator.tick(tick_count);
        }
        control_events_fn(WorkEvent::Control(self.oscillator.value().into()));
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn skip_to_start(&mut self) {
        // TODO: think how important it is for LFO oscillator to start at zero
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl HandlesMidi for LfoController {}
impl LfoController {
    pub fn new_with(oscillator: Oscillator) -> Self {
        Self {
            oscillator,
            is_performing: false,

            time_range: Default::default(),
            last_frame: Default::default(),
        }
    }

    pub fn notify_change_oscillator(&mut self) {}

    pub const fn frequency_range() -> std::ops::RangeInclusive<ParameterType> {
        0.0..=100.0
    }
}
