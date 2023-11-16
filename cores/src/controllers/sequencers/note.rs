// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::midi::MidiSequencer;
use derive_builder::Builder;
use ensnare_core::{piano_roll::Note, prelude::*, rng::Rng, traits::Sequences};
use ensnare_proc_macros::InnerConfigurable;

impl NoteSequencerBuilder {
    /// Builds the [NoteSequencer].
    pub fn build(&self) -> anyhow::Result<NoteSequencer, NoteSequencerBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    /// Produces a random sequence of quarter-note notes. For debugging.
    pub fn random(&mut self, range: TimeRange) -> &mut Self {
        let mut rng = Rng::default();

        for _ in 0..32 {
            let beat_range = range.0.start.total_beats() as u64..range.0.end.total_beats() as u64;
            let note_start = MusicalTime::new_with_beats(rng.0.rand_range(beat_range) as usize);
            self.note(Note {
                key: rng.0.rand_range(16..100) as u8,
                range: TimeRange(note_start..note_start + MusicalTime::DURATION_QUARTER),
            });
        }
        self
    }
}

#[derive(Debug, Default, Builder, InnerConfigurable)]
#[builder(build_fn(private, name = "build_from_builder"))]
pub struct NoteSequencer {
    #[builder(setter(skip))]
    pub inner: MidiSequencer,

    #[builder(default, setter(each(name = "note", into)))]
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
            note.range.0.start,
        );
        let _ = self.inner.remove_midi_message(
            channel,
            MidiMessage::NoteOff {
                key: u7::from(note.key),
                vel: u7::from(127),
            },
            note.range.0.end,
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
    fn update_time_range(&mut self, range: &TimeRange) {
        self.inner.update_time_range(range)
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
impl Serializable for NoteSequencer {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.notes.iter().for_each(|note| {
            let events: Vec<MidiEvent> = note.clone().into();
            events.iter().for_each(|e| {
                let _ = self.inner.record_midi_event(MidiChannel::default(), *e);
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controllers::sequencers::tests::validate_sequences_notes_trait;

    #[test]
    fn note_sequencer_passes_trait_validation() {
        let mut s = NoteSequencer::default();

        validate_sequences_notes_trait(&mut s);
    }

    /////////////////////////////////////////////////////////////////////////
    /// BEGIN tests taken from the old sequencer. These are here to scavenge
    /// good testing ideas.

    #[cfg(tired)]
    impl MidiTickSequencer {
        #[allow(dead_code)]
        pub(crate) fn debug_events(&self) -> &MidiTickEventsMap {
            &self.events
        }
    }

    #[cfg(tired)]
    impl MidiTickSequencer {
        pub(crate) fn tick_for_beat(&self, clock: &Clock, beat: usize) -> MidiTicks {
            //            let tpb = self.midi_ticks_per_second.0 as f32 /
            //            (clock.bpm() / 60.0);
            let tpb = 960.0 / (clock.bpm() / 60.0); // TODO: who should own the number of ticks/second?
            MidiTicks::from(tpb * beat as f64)
        }
    }

    // fn advance_to_next_beat(
    //     clock: &mut Clock,
    //     sequencer: &mut dyn IsController<Message = EntityMessage>,
    // ) {
    //     let next_beat = clock.beats().floor() + 1.0;
    //     while clock.beats() < next_beat {
    //         // TODO: a previous version of this utility function had
    //         // clock.tick() first, meaning that the sequencer never got the 0th
    //         // (first) tick. No test ever cared, apparently. Fix this.
    //         let _ = sequencer.work(1);
    //         clock.tick(1);
    //     }
    // }

    // // We're papering over the issue that MIDI events are firing a little late.
    // // See Clock::next_slice_in_midi_ticks().
    // fn advance_one_midi_tick(
    //     clock: &mut Clock,
    //     sequencer: &mut dyn IsController<Message = EntityMessage>,
    // ) {
    //     let next_midi_tick = clock.midi_ticks() + 1;
    //     while clock.midi_ticks() < next_midi_tick {
    //         let _ = sequencer.work(1);
    //         clock.tick(1);
    //     }
    // }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    #[test]
    fn sequencer_mainline() {
        const DEVICE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);
        #[cfg(obsolete)]
        let clock = Clock::new_with(
            DEFAULT_BPM,
            DEFAULT_MIDI_TICKS_PER_SECOND,
            TimeSignature::default(),
        );
        // let mut o = Orchestrator::new_with(DEFAULT_BPM);
        // let mut sequencer = Box::new(MidiTickSequencer::new_with(
        //     DEFAULT_SAMPLE_RATE,
        //     DEFAULT_MIDI_TICKS_PER_SECOND,
        // ));
        // let instrument = Box::new(ToyInstrument::new_with(clock.sample_rate()));
        // let device_uid = o.add(None, Entity::ToyInstrument(instrument));

        // sequencer.insert(
        //     sequencer.tick_for_beat(&clock, 0),
        //     DEVICE_MIDI_CHANNEL,
        //     new_note_on(MidiNote::C4 as u8, 127),
        // );
        // sequencer.insert(
        //     sequencer.tick_for_beat(&clock, 1),
        //     DEVICE_MIDI_CHANNEL,
        //     new_note_off(MidiNote::C4 as u8, 0),
        // );
        // const SEQUENCER_ID: &'static str = "seq";
        // let _sequencer_uid = o.add(Some(SEQUENCER_ID), Entity::MidiTickSequencer(sequencer));
        // o.connect_midi_downstream(device_uid, DEVICE_MIDI_CHANNEL);

        // // TODO: figure out a reasonable way to test these things once they're
        // // inside Store, and their type information has been erased. Maybe we
        // // can send messages asking for state. Maybe we can send things that the
        // // entities themselves assert.
        // if let Some(entity) = o.get_mut(SEQUENCER_ID) {
        //     if let Some(sequencer) = entity.as_is_controller_mut() {
        //         advance_one_midi_tick(&mut clock, sequencer);
        //         {
        //             // assert!(instrument.is_playing);
        //             // assert_eq!(instrument.received_count, 1);
        //             // assert_eq!(instrument.handled_count, 1);
        //         }
        //     }
        // }

        // if let Some(entity) = o.get_mut(SEQUENCER_ID) {
        //     if let Some(sequencer) = entity.as_is_controller_mut() {
        //         advance_to_next_beat(&mut clock, sequencer);
        //         {
        //             // assert!(!instrument.is_playing);
        //             // assert_eq!(instrument.received_count, 2);
        //             // assert_eq!(&instrument.handled_count, &2);
        //         }
        //     }
        // }
    }

    // TODO: re-enable later.......................................................................
    // #[test]
    // fn sequencer_multichannel() {
    //     let mut clock = Clock::default();
    //     let mut sequencer = MidiTickSequencer::<TestMessage>::default();

    //     let device_1 = rrc(TestMidiSink::default());
    //     assert!(!device_1.borrow().is_playing);
    //     device_1.borrow_mut().set_midi_channel(0);
    //     sequencer.add_midi_sink(0, rrc_downgrade::<TestMidiSink<TestMessage>>(&device_1));

    //     let device_2 = rrc(TestMidiSink::default());
    //     assert!(!device_2.borrow().is_playing);
    //     device_2.borrow_mut().set_midi_channel(1);
    //     sequencer.add_midi_sink(1, rrc_downgrade::<TestMidiSink<TestMessage>>(&device_2));

    //     sequencer.insert(
    //         sequencer.tick_for_beat(&clock, 0),
    //         0,
    //         new_note_on(60, 0),
    //     );
    //     sequencer.insert(
    //         sequencer.tick_for_beat(&clock, 1),
    //         1,
    //         new_note_on(60, 0),
    //     );
    //     sequencer.insert(
    //         sequencer.tick_for_beat(&clock, 2),
    //         0,
    //         new_note_off(MidiNote::C4 as u8, 0),
    //     );
    //     sequencer.insert(
    //         sequencer.tick_for_beat(&clock, 3),
    //         1,
    //         new_note_off(MidiNote::C4 as u8, 0),
    //     );
    //     assert_eq!(sequencer.debug_events().len(), 4);

    //     // Let the tick #0 event(s) fire.
    //     assert_eq!(clock.samples(), 0);
    //     assert_eq!(clock.midi_ticks(), 0);
    //     advance_one_midi_tick(&mut clock, &mut sequencer);
    //     {
    //         let dp_1 = device_1.borrow();
    //         assert!(dp_1.is_playing);
    //         assert_eq!(dp_1.received_count, 1);
    //         assert_eq!(dp_1.handled_count, 1);

    //         let dp_2 = device_2.borrow();
    //         assert!(!dp_2.is_playing);
    //         assert_eq!(dp_2.received_count, 0);
    //         assert_eq!(dp_2.handled_count, 0);
    //     }

    //     advance_to_next_beat(&mut clock, &mut sequencer);
    //     assert_eq!(clock.beats().floor(), 1.0); // TODO: these floor() calls are a smell
    //     {
    //         let dp = device_1.borrow();
    //         assert!(dp.is_playing);
    //         assert_eq!(dp.received_count, 1);
    //         assert_eq!(dp.handled_count, 1);

    //         let dp_2 = device_2.borrow();
    //         assert!(dp_2.is_playing);
    //         assert_eq!(dp_2.received_count, 1);
    //         assert_eq!(dp_2.handled_count, 1);
    //     }

    //     advance_to_next_beat(&mut clock, &mut sequencer);
    //     assert_eq!(clock.beats().floor(), 2.0);
    //     // assert_eq!(sequencer.tick_sequencer.events.len(), 1);
    //     {
    //         let dp = device_1.borrow();
    //         assert!(!dp.is_playing);
    //         assert_eq!(dp.received_count, 2);
    //         assert_eq!(dp.handled_count, 2);

    //         let dp_2 = device_2.borrow();
    //         assert!(dp_2.is_playing);
    //         assert_eq!(dp_2.received_count, 1);
    //         assert_eq!(dp_2.handled_count, 1);
    //     }

    //     advance_to_next_beat(&mut clock, &mut sequencer);
    //     assert_eq!(clock.beats().floor(), 3.0);
    //     // assert_eq!(sequencer.tick_sequencer.events.len(), 0);
    //     {
    //         let dp = device_1.borrow();
    //         assert!(!dp.is_playing);
    //         assert_eq!(dp.received_count, 2);
    //         assert_eq!(dp.handled_count, 2);

    //         let dp_2 = device_2.borrow();
    //         assert!(!dp_2.is_playing);
    //         assert_eq!(dp_2.received_count, 2);
    //         assert_eq!(dp_2.handled_count, 2);
    //     }
    // }
}
