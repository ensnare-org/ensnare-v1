// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// [Transport] is the global clock. It keeps track of the current position in
/// the song, and how time should advance.
#[derive(Clone, Control, Debug, Default, Builder, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Transport {
    /// The current global time signature.
    #[builder(default)]
    pub time_signature: TimeSignature,

    /// The current beats per minute.
    #[builder(default)]
    #[control]
    pub tempo: Tempo,

    #[builder(setter(skip))]
    #[serde(skip)]
    e: TransportEphemerals,
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

impl PartialEq for Transport {
    fn eq(&self, other: &Self) -> bool {
        self.time_signature == other.time_signature && self.tempo == other.tempo
    }
}
impl HandlesMidi for Transport {}
impl Transport {
    /// Advances the clock by the given number of frames. Returns the time range
    /// from the prior time to now.
    pub fn advance(&mut self, frames: usize) -> TimeRange {
        // Calculate the work time range. Note that the range can be zero, which
        // will happen if frames advance faster than MusicalTime units.
        let new_frames = self.e.current_frame + frames;
        let new_time = MusicalTime::new_with_frames(self.tempo, self.e.sample_rate, new_frames);
        let length = if new_time >= self.e.current_time {
            new_time - self.e.current_time
        } else {
            MusicalTime::DURATION_ZERO
        };
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
        TimeRange(range)
    }

    #[allow(missing_docs)]
    pub fn current_time(&self) -> MusicalTime {
        self.e.current_time
    }

    pub fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    // Don't delete this! The #[derive(Control)] macro expects exactly this
    // method name.
    pub fn set_tempo(&mut self, tempo: Tempo) {
        self.update_tempo(tempo)
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

    fn tempo(&self) -> Tempo {
        self.tempo
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo;
    }

    fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }
    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
    }
}
impl Controls for Transport {
    fn update_time_range(&mut self, range: &TimeRange) {
        // Nothing - we calculated the range, so we don't need to do anything with it.
        debug_assert!(
            self.e.current_time == range.0.end,
            "Transport::update_time() was called with the range ..{} but current_time is {}",
            range.0.end,
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
        // Stopping when already stopped resets the time to start, which gives
        // the stop button a convenient dual function.
        if self.e.is_performing {
            self.e.is_performing = false;
        } else {
            self.skip_to_start();
        }
    }

    fn skip_to_start(&mut self) {
        self.e.current_time = MusicalTime::default();
        self.e.current_frame = Default::default();
    }

    fn is_performing(&self) -> bool {
        self.e.is_performing
    }
}
