// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use crate::traits::prelude::*;
use ensnare_proc_macros::{Control, IsController, Uid};
use serde::{Deserialize, Serialize};

/// [Timer] runs for a specified amount of time, then indicates that it's done.
/// It is useful when you need something to happen after a certain amount of
/// wall-clock time, rather than musical time.
#[derive(Debug, Control, IsController, Uid, Serialize, Deserialize)]
pub struct Timer {
    uid: Uid,

    duration: MusicalTime,

    #[serde(skip)]
    is_performing: bool,

    #[serde(skip)]
    is_finished: bool,

    #[serde(skip)]
    end_time: Option<MusicalTime>,
}
impl Serializable for Timer {}
#[allow(missing_docs)]
impl Timer {
    pub fn new_with(duration: MusicalTime) -> Self {
        Self {
            uid: Default::default(),
            duration,
            is_performing: false,
            is_finished: false,
            end_time: Default::default(),
        }
    }

    pub fn duration(&self) -> MusicalTime {
        self.duration
    }

    pub fn set_duration(&mut self, duration: MusicalTime) {
        self.duration = duration;
    }
}
impl HandlesMidi for Timer {}
impl Configurable for Timer {}
impl Controls for Timer {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {
        if self.is_performing {
            if self.duration == MusicalTime::default() {
                // Zero-length timers fire immediately.
                self.is_finished = true;
            } else {
                if let Some(end_time) = self.end_time {
                    if range.contains(&end_time) {
                        self.is_finished = true;
                    }
                } else {
                    // The first time we're called with an update_time() while
                    // performing, we take that as the start of the timer.
                    self.end_time = Some(range.start + self.duration);
                }
            }
        }
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Displays for Timer {}

// TODO: needs tests!
/// [Trigger] issues a control signal after a specified amount of time.
#[derive(Debug, Control, IsController, Uid, Serialize, Deserialize)]
pub struct Trigger {
    uid: Uid,

    timer: Timer,

    value: ControlValue,

    has_triggered: bool,
    is_performing: bool,
}
impl Serializable for Trigger {}
impl Controls for Trigger {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {
        self.timer.update_time(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.timer.is_finished() && self.is_performing && !self.has_triggered {
            self.has_triggered = true;
            control_events_fn(self.uid, EntityEvent::Control(self.value));
        }
    }

    fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }

    fn play(&mut self) {
        self.is_performing = true;
        self.timer.play();
    }

    fn stop(&mut self) {
        self.is_performing = false;
        self.timer.stop();
    }

    fn skip_to_start(&mut self) {
        self.has_triggered = false;
        self.timer.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Configurable for Trigger {
    fn sample_rate(&self) -> SampleRate {
        self.timer.sample_rate()
    }
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.timer.update_sample_rate(sample_rate)
    }
}
impl HandlesMidi for Trigger {}
impl Trigger {
    pub fn new_with(timer: Timer, value: ControlValue) -> Self {
        Self {
            uid: Default::default(),
            timer,
            value,
            has_triggered: false,
            is_performing: false,
        }
    }

    pub fn value(&self) -> ControlValue {
        self.value
    }

    pub fn set_value(&mut self, value: ControlValue) {
        self.value = value;
    }
}
impl Displays for Trigger {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    #[test]
    fn instantiate_trigger() {
        let ts = TimeSignature::default();
        let mut trigger = Trigger::new_with(
            Timer::new_with(MusicalTime::new_with_bars(&ts, 1)),
            ControlValue::from(0.5),
        );
        trigger.update_sample_rate(SampleRate::DEFAULT);
        trigger.play();

        trigger.update_time(&Range {
            start: MusicalTime::default(),
            end: MusicalTime::new_with_parts(1),
        });
        let mut count = 0;
        trigger.work(&mut |_, _| {
            count += 1;
        });
        assert_eq!(count, 0);
        assert!(!trigger.is_finished());

        trigger.update_time(&Range {
            start: MusicalTime::new_with_bars(&ts, 1),
            end: MusicalTime::new(&ts, 1, 0, 0, 1),
        });
        let mut count = 0;
        trigger.work(&mut |_, _| {
            count += 1;
        });
        assert!(count != 0);
        assert!(trigger.is_finished());
    }
}
