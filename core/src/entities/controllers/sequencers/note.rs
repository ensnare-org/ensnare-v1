// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    midi::{u7, MidiChannel, MidiEvent, MidiMessage},
    piano_roll::Note,
    time::MusicalTime,
    traits::{Configurable, ControlEventsFn, Controls, Sequences, SequencesMidi},
};

use super::MidiSequencer;

#[derive(Debug, Default)]
pub struct NoteSequencer {
    inner: MidiSequencer,
    notes: Vec<Note>,
}
impl Sequences for NoteSequencer {
    type MU = Note;

    fn record(
        &mut self,
        channel: MidiChannel,
        note: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let note = note.clone() + position;
        let events: Vec<MidiEvent> = note.clone().into();
        events.iter().for_each(|e| {
            let _ = self.inner.record_midi_event(channel, *e);
        });
        self.notes.push(note);
        Ok(())
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        note: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let note = note.clone() + position;
        let _ = self.inner.remove_midi_message(
            channel,
            MidiMessage::NoteOn {
                key: u7::from(note.key),
                vel: u7::from(127),
            },
            note.range.start,
        );
        let _ = self.inner.remove_midi_message(
            channel,
            MidiMessage::NoteOff {
                key: u7::from(note.key),
                vel: u7::from(127),
            },
            note.range.end,
        );
        self.notes.retain(|n| *n != note);
        Ok(())
    }

    fn clear(&mut self) {
        self.notes.clear();
        self.inner.clear();
    }
}
impl Controls for NoteSequencer {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {
        self.inner.update_time(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.inner.work(control_events_fn)
    }

    fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }

    fn play(&mut self) {
        self.inner.play()
    }

    fn stop(&mut self) {
        self.inner.stop()
    }

    fn skip_to_start(&mut self) {
        self.inner.skip_to_start()
    }

    fn is_performing(&self) -> bool {
        self.inner.is_performing()
    }
}
impl Configurable for NoteSequencer {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::tests::validate_sequences_notes_trait;

    #[test]
    fn sequencer_works() {
        let mut s = NoteSequencer::default();

        validate_sequences_notes_trait(&mut s);
    }
}
