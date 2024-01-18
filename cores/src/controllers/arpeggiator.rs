// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{composition::sequencers::NoteSequencer, prelude::*, traits::Sequences};
use serde::{Deserialize, Serialize};

use std::option::Option;
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

/// [Arpeggiator] creates [arpeggios](https://en.wikipedia.org/wiki/Arpeggio),
/// which "is a type of broken chord in which the notes that compose a chord are
/// individually and quickly sounded in a progressive rising or descending
/// order." You can also think of it as a hybrid MIDI instrument and MIDI
/// controller; you play it with MIDI, but instead of producing audio, it
/// produces more MIDI.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Arpeggiator {
    midi_channel_out: MidiChannel,
    sequencer: NoteSequencer,
    is_sequencer_enabled: bool,

    pub bpm: Tempo,

    pub mode: ArpeggioMode,

    // A poor-man's semaphore that allows note-off events to overlap with the
    // current note without causing it to shut off. Example is a legato
    // playing-style of the MIDI instrument that controls the arpeggiator. If we
    // turned on and off solely by the last note-on/off we received, then the
    // arpeggiator would frequently get clipped.
    note_semaphore: i16,
}
impl Configurable for Arpeggiator {
    fn sample_rate(&self) -> SampleRate {
        self.sequencer.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sequencer.update_sample_rate(sample_rate);
    }
}
impl Controls for Arpeggiator {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.sequencer.update_time_range(range);
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.sequencer.work(control_events_fn)
    }

    fn is_finished(&self) -> bool {
        self.sequencer.is_finished()
    }

    fn play(&mut self) {
        self.sequencer.play();
    }

    fn stop(&mut self) {
        self.sequencer.stop();
    }

    fn skip_to_start(&mut self) {
        self.sequencer.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.sequencer.is_performing()
    }
}
impl HandlesMidi for Arpeggiator {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        match message {
            MidiMessage::NoteOff { key: _, vel: _ } => {
                self.note_semaphore -= 1;
                if self.note_semaphore < 0 {
                    self.note_semaphore = 0;
                }
                self.is_sequencer_enabled = self.note_semaphore > 0;
            }
            MidiMessage::NoteOn { key, vel } => {
                self.note_semaphore += 1;
                self.rebuild_sequence(key.as_int(), vel.as_int());
                self.is_sequencer_enabled = true;

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
                // self.sequencer.generate_midi_messages_for_current_frame(midi_messages_fn);
                //
                // TODO 10-2023: I don't understand the prior comment. I should
                // have just written a unit test. I think that
                // generate_midi_messages_for_current_frame() was just the same
                // as work() for the current time slice, which we can assume
                // will be called. We'll see.
            }
            MidiMessage::Aftertouch { key: _, vel: _ } => todo!(),
            MidiMessage::Controller {
                controller: _,
                value: _,
            } => todo!(),
            MidiMessage::ProgramChange { program: _ } => todo!(),
            MidiMessage::ChannelAftertouch { vel: _ } => todo!(),
            MidiMessage::PitchBend { bend: _ } => todo!(),
        }
    }
}
impl Arpeggiator {
    pub fn new_with(bpm: Tempo, midi_channel_out: MidiChannel) -> Self {
        Self {
            bpm,
            midi_channel_out,
            ..Default::default()
        }
    }

    fn insert_one_note(&mut self, when: &MusicalTime, duration: &MusicalTime, key: u8, _vel: u8) {
        let _ = self.sequencer.record(
            self.midi_channel_out,
            &Note::new_with(key, MusicalTime::START, *duration),
            *when,
        );
    }

    fn rebuild_sequence(&mut self, key: u8, vel: u8) {
        self.sequencer.clear();

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

    pub fn bpm(&self) -> Tempo {
        self.bpm
    }

    pub fn set_bpm(&mut self, bpm: Tempo) {
        self.bpm = bpm;
    }

    pub fn mode(&self) -> ArpeggioMode {
        self.mode
    }
}
