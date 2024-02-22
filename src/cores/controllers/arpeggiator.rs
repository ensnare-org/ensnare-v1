// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{composition::NoteSequencer, prelude::*};
use delegate::delegate;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, FromRepr, IntoStaticStr};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    EnumCount,
    EnumIter,
    FromRepr,
    IntoStaticStr,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum ArpeggioMode {
    #[default]
    Major,
    Minor,
}

/// [ArpeggiatorCore] creates
/// [arpeggios](https://en.wikipedia.org/wiki/Arpeggio), which "is a type of
/// broken chord in which the notes that compose a chord are individually and
/// quickly sounded in a progressive rising or descending order." You can also
/// think of it as a hybrid MIDI instrument and MIDI controller; you play it
/// with MIDI, but instead of producing audio, it produces more MIDI.
#[derive(Clone, Builder, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default)]
pub struct ArpeggiatorCore {
    pub mode: ArpeggioMode,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: ArpeggiatorEphemerals,
}
#[derive(Clone, Debug, Default)]
pub struct ArpeggiatorEphemerals {
    sequencer: NoteSequencer,
    is_sequencer_enabled: bool,

    // A poor-man's semaphore that allows note-off events to overlap with the
    // current note without causing it to shut off. Example is a legato
    // playing-style of the MIDI instrument that controls the arpeggiator. If we
    // turned on and off solely by the last note-on/off we received, then the
    // arpeggiator would frequently get clipped.
    note_semaphore: i16,
}
impl Configurable for ArpeggiatorEphemerals {
    delegate! {
        to self.sequencer {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl Configurable for ArpeggiatorCore {
    delegate! {
        to self.e {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl Controls for ArpeggiatorCore {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.e.sequencer.update_time_range(range);
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.e.sequencer.work(control_events_fn)
    }

    fn is_finished(&self) -> bool {
        self.e.sequencer.is_finished()
    }

    fn play(&mut self) {
        self.e.sequencer.play();
    }

    fn stop(&mut self) {
        self.e.sequencer.stop();
    }

    fn skip_to_start(&mut self) {
        self.e.sequencer.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.e.sequencer.is_performing()
    }
}
impl HandlesMidi for ArpeggiatorCore {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        match message {
            MidiMessage::NoteOff { key: _, vel: _ } => {
                self.e.note_semaphore -= 1;
                if self.e.note_semaphore < 0 {
                    self.e.note_semaphore = 0;
                }
                self.e.is_sequencer_enabled = self.e.note_semaphore > 0;
            }
            MidiMessage::NoteOn { key, vel } => {
                self.e.note_semaphore += 1;
                self.rebuild_sequence(key.as_int(), vel.as_int());
                self.e.is_sequencer_enabled = true;

                // TODO: this scratches the itch of needing to respond to a
                // note-down with a note *during this slice*, but it also has an
                // edge condition where we need to cancel a different note that
                // was might have been supposed to be sent instead during this
                // slice, or at least immediately shut it off. This seems to
                // require a two-phase Tick handler (one to decide what we're
                // going to send, and another to send it), and an internal
                // memory of which notes we've asked the downstream to play.
                // TODO TODO TODO
                //
                // self.e.sequencer.generate_midi_messages_for_current_frame(midi_messages_fn);
                //
                // TODO 10-2023: I don't understand the prior comment. I should
                // have just written a unit test. I think that
                // generate_midi_messages_for_current_frame() was just the same
                // as work() for the current time slice, which we can assume
                // will be called. We'll see.
            }
            MidiMessage::Aftertouch { key: _, vel: _ } => todo!(),
            MidiMessage::Controller {
                controller,
                value: _,
            } => match controller.as_int() {
                123 => {
                    self.e.note_semaphore = 0;
                    self.e.is_sequencer_enabled = false;
                }
                _ => {}
            },
            MidiMessage::ProgramChange { program: _ } => todo!(),
            MidiMessage::ChannelAftertouch { vel: _ } => todo!(),
            MidiMessage::PitchBend { bend: _ } => todo!(),
        }
    }
}
impl ArpeggiatorCore {
    fn insert_one_note(&mut self, when: &MusicalTime, duration: &MusicalTime, key: u8, _vel: u8) {
        let _ = self.e.sequencer.record(
            MidiChannel::default(),
            &Note::new_with(key, MusicalTime::START, *duration),
            *when,
        );
    }

    fn rebuild_sequence(&mut self, key: u8, vel: u8) {
        self.e.sequencer.clear();

        let start_beat = MusicalTime::START; // TODO: this is wrong, but I'm just trying to get this code to build for now
        let duration = MusicalTime::new_with_parts(4); // TODO: we're ignoring time signature!
        let scale_notes = match self.mode {
            ArpeggioMode::Major => [0, 2, 4, 5, 7, 9, 11], // W W H W W W H
            ArpeggioMode::Minor => [0, 2, 3, 5, 7, 8, 10], // W H W W H W W
        };
        for (index, offset) in scale_notes.iter().enumerate() {
            // TODO - more examples of needing wider range for smaller parts
            let when = start_beat + MusicalTime::new_with_parts(4 * index);
            self.insert_one_note(&when, &duration, key + offset, vel);
        }
    }

    pub fn mode(&self) -> ArpeggioMode {
        self.mode
    }
}
