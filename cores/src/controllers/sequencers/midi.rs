// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    midi::{MidiChannel, MidiEvent, MidiMessage},
    time::{MusicalTime, SampleRate, Tempo, TimeSignature},
    traits::{
        Configurable, ControlEventsFn, Controls, HandlesMidi, MidiMessagesFn, SequencesMidi,
        TimeRange, WorkEvent,
    },
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MidiSequencer {
    #[serde(skip)]
    events: Vec<(MidiChannel, MidiEvent)>,
    pub time_range: TimeRange,
    is_recording: bool,
    is_performing: bool,
    max_event_time: MusicalTime,
}
impl SequencesMidi for MidiSequencer {
    fn clear(&mut self) {
        self.events.clear();
        self.max_event_time = MusicalTime::default();
    }

    fn record_midi_event(&mut self, channel: MidiChannel, event: MidiEvent) -> anyhow::Result<()> {
        self.events.push((channel, event));
        if event.time > self.max_event_time {
            self.max_event_time = event.time;
        }
        Ok(())
    }

    fn remove_midi_event(&mut self, channel: MidiChannel, event: MidiEvent) -> anyhow::Result<()> {
        self.events.retain(|e| *e != (channel, event));
        self.recalculate_max_time();
        Ok(())
    }

    fn start_recording(&mut self) {
        self.is_recording = true;
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}
impl Configurable for MidiSequencer {
    fn sample_rate(&self) -> SampleRate {
        // I was too lazy to add this everywhere when I added this to the trait,
        // but I didn't want unexpected usage to go undetected.
        panic!("Someone asked for a SampleRate but we provided default");
    }

    fn update_sample_rate(&mut self, _sample_rate: SampleRate) {}

    fn update_tempo(&mut self, _tempo: Tempo) {}

    fn update_time_signature(&mut self, _time_signature: TimeSignature) {}
}
impl Controls for MidiSequencer {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.time_range = range.clone();
    }

    //    #[deprecated = "FIX THE CHANNEL!"]
    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        // OMG this is O(n^2)
        self.events.iter().for_each(|(channel, event)| {
            if self.time_range.0.contains(&event.time) {
                control_events_fn(WorkEvent::Midi(*channel, event.message))
            }
        });
    }

    fn is_finished(&self) -> bool {
        self.time_range.0.end >= self.max_event_time
    }

    fn play(&mut self) {
        self.is_performing = true;
        self.is_recording = false;
    }

    fn stop(&mut self) {
        self.is_performing = false;
        self.is_recording = false;
    }

    fn skip_to_start(&mut self) {
        self.time_range = TimeRange(MusicalTime::default()..MusicalTime::default())
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl HandlesMidi for MidiSequencer {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        if self.is_recording {
            let _ = self.record_midi_message(channel, message, self.time_range.0.start);
        }
    }
}
impl MidiSequencer {
    fn recalculate_max_time(&mut self) {
        if let Some(max_event_time) = self.events.iter().map(|(_, event)| event.time).max() {
            self.max_event_time = max_event_time;
        } else {
            self.max_event_time = MusicalTime::default();
        }
    }
}
