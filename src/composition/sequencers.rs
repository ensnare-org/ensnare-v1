// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{prelude::*, rng::Rng};
use delegate::delegate;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Builder, PartialEq)]
//#[serde(rename_all = "kebab-case")]
pub struct MidiSequencer {
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
            let note_start = MusicalTime::new_with_beats(rng.rand_range(beat_range) as usize);
            self.note(Note::new_with(
                rng.rand_range(16..100) as u8,
                note_start,
                MusicalTime::DURATION_QUARTER,
            ));
        }
        self
    }
}

#[derive(Debug, Default, Builder, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(build_fn(private, name = "build_from_builder"))]
pub struct NoteSequencer {
    #[builder(default, setter(each(name = "note", into)))]
    notes: Vec<Note>,

    #[serde(skip)]
    #[builder(setter(skip))]
    pub e: NoteSequencerEphemerals,
}
#[derive(Debug, Default)]
pub struct NoteSequencerEphemerals {
    pub inner: MidiSequencer,
    pub extent: TimeRange,
}
impl Sequences for NoteSequencer {
    type MU = Note;

    fn record(
        &mut self,
        channel: MidiChannel,
        note: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let note = note.shift_right(position);
        let events: Vec<MidiEvent> = note.clone().into();
        events.iter().for_each(|e| {
            let _ = self.e.inner.record_midi_event(channel, *e);
        });
        self.e
            .extent
            .expand_with_range(&note.extent().shift_right(position));
        self.notes.push(note);
        Ok(())
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        note: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let note = note.shift_right(position);
        let _ = self.e.inner.remove_midi_message(
            channel,
            MidiMessage::NoteOn {
                key: u7::from(note.key),
                vel: u7::from(127),
            },
            note.extent.0.start,
        );
        let _ = self.e.inner.remove_midi_message(
            channel,
            MidiMessage::NoteOff {
                key: u7::from(note.key),
                vel: u7::from(127),
            },
            note.extent.0.end,
        );
        self.notes.retain(|n| *n != note);
        self.recalculate_extent();
        Ok(())
    }

    fn clear(&mut self) {
        self.notes.clear();
        self.e.inner.clear();
    }
}
impl Controls for NoteSequencer {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.e.inner.update_time_range(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.e.inner.work(control_events_fn)
    }

    fn is_finished(&self) -> bool {
        self.e.inner.is_finished()
    }

    fn play(&mut self) {
        self.e.inner.play()
    }

    fn stop(&mut self) {
        self.e.inner.stop()
    }

    fn skip_to_start(&mut self) {
        self.e.inner.skip_to_start()
    }

    fn is_performing(&self) -> bool {
        self.e.inner.is_performing()
    }
}
impl Serializable for NoteSequencer {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.notes.iter().for_each(|note| {
            let events: Vec<MidiEvent> = note.clone().into();
            events.iter().for_each(|e| {
                let _ = self.e.inner.record_midi_event(MidiChannel::default(), *e);
            });
        });
        self.recalculate_extent();
    }
}
impl NoteSequencer {
    fn recalculate_extent(&mut self) {
        self.e.extent = Default::default();
        self.notes.iter().for_each(|note| {
            self.e.extent.expand_with_range(&note.extent());
        });
    }
}
impl HasExtent for NoteSequencer {
    fn extent(&self) -> TimeRange {
        self.e.extent.clone()
    }

    fn set_extent(&mut self, extent: TimeRange) {
        self.e.extent = extent;
    }
}
impl Configurable for NoteSequencer {
    delegate! {
        to self.e.inner {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}

impl PatternSequencerBuilder {
    /// Builds the [PatternSequencer].
    pub fn build(&self) -> Result<PatternSequencer, PatternSequencerBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}

/// A sequencer that works in terms of static copies of [Pattern]s. Recording a
/// [Pattern] and then later changing it won't change what's recorded in this
/// sequencer.
///
/// This makes remove() a little weird. You can't remove a pattern that you've
/// changed, because the sequencer won't recognize that the new pattern was
/// meant to refer to the old pattern.
///
/// This sequencer is nice for certain test cases, but I don't think it's useful
/// in a production environment. [LivePatternSequencer] is better.
#[derive(Debug, Default, Builder, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[builder(build_fn(private, name = "build_from_builder"))]
pub struct PatternSequencer {
    #[builder(default, setter(each(name = "pattern", into)))]
    pub patterns: Vec<(MidiChannel, Pattern)>,

    #[serde(skip)]
    #[builder(setter(skip))]
    pub e: PatternSequencerEphemerals,
}
#[derive(Debug, Default, PartialEq)]
pub struct PatternSequencerEphemerals {
    pub inner: MidiSequencer,
    pub extent: TimeRange,
}
impl PatternSequencerEphemerals {
    fn clear(&mut self) {
        self.inner.clear();
        self.extent = Default::default();
    }
}
impl Sequences for PatternSequencer {
    type MU = Pattern;

    fn record(
        &mut self,
        channel: MidiChannel,
        pattern: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let pattern = pattern.shift_right(position);
        let events: Vec<MidiEvent> = pattern.clone().into();
        events.iter().for_each(|&e| {
            let _ = self.e.inner.record_midi_event(channel, e);
        });
        self.e
            .extent
            .expand_with_range(&pattern.extent().shift_right(position));
        self.patterns.push((channel, pattern));
        Ok(())
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        pattern: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let pattern = pattern.shift_right(position);
        let events: Vec<MidiEvent> = pattern.clone().into();
        events.iter().for_each(|&e| {
            let _ = self.e.inner.remove_midi_event(channel, e);
        });
        self.patterns
            .retain(|(c, p)| *c != channel || *p != pattern);
        self.recalculate_extent();
        Ok(())
    }

    fn clear(&mut self) {
        self.patterns.clear();
        self.e.clear();
    }
}
impl HasExtent for PatternSequencer {
    fn extent(&self) -> TimeRange {
        self.e.extent.clone()
    }

    fn set_extent(&mut self, extent: TimeRange) {
        self.e.extent = extent;
    }
}
impl Controls for PatternSequencer {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.e.inner.update_time_range(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.e.inner.work(control_events_fn)
    }

    fn is_finished(&self) -> bool {
        self.e.inner.is_finished()
    }

    fn play(&mut self) {
        self.e.inner.play()
    }

    fn stop(&mut self) {
        self.e.inner.stop()
    }

    fn skip_to_start(&mut self) {
        self.e.inner.skip_to_start()
    }

    fn is_performing(&self) -> bool {
        self.e.inner.is_performing()
    }
}
impl Serializable for PatternSequencer {
    fn after_deser(&mut self) {
        for (channel, pattern) in &self.patterns {
            let events: Vec<MidiEvent> = pattern.clone().into();
            events.iter().for_each(|&e| {
                let _ = self.e.inner.record_midi_event(*channel, e);
            });
        }
        self.recalculate_extent();
    }
}
impl PatternSequencer {
    fn recalculate_extent(&mut self) {
        self.e.extent = Default::default();
        self.patterns.iter().for_each(|(_channel, pattern)| {
            self.e.extent.expand_with_range(&pattern.extent());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    use super::PatternSequencerBuilder;
    use crate::{
        midi::MidiChannel,
        prelude::{MusicalTime, PatternBuilder, TimeRange, TimeSignature},
        traits::{HasExtent, Sequences},
    };

    #[test]
    fn pattern_sequencer_handles_extents() {
        let mut s = PatternSequencerBuilder::default().build().unwrap();

        assert_eq!(
            s.extent(),
            TimeRange::default(),
            "Empty sequencer should have empty extent"
        );

        let pattern = PatternBuilder::default().build().unwrap();
        assert_eq!(pattern.time_signature(), TimeSignature::default());
        assert!(s
            .record(MidiChannel::default(), &pattern, MusicalTime::START)
            .is_ok());
        assert_eq!(
            s.extent(),
            TimeRange(MusicalTime::START..MusicalTime::new_with_beats(4)),
            "Adding an empty 4/4 pattern to a sequencer should update the extent to one measure"
        );

        assert!(s
            .remove(MidiChannel::default(), &pattern, MusicalTime::START)
            .is_ok());
        assert_eq!(
            s.extent(),
            TimeRange::default(),
            "After removing last pattern from sequencer, its extent should return to empty"
        );

        assert!(s
            .record(MidiChannel::default(), &pattern, MusicalTime::ONE_BEAT * 16)
            .is_ok());
        assert_eq!(
            s.extent(),
            TimeRange(MusicalTime::START..(MusicalTime::new_with_beats(4) + MusicalTime::ONE_BEAT * 16)),
            "Adding a 4/4 pattern later in a 4/4 score should update the extent to one measure starting at the 16th measure"
        );
    }

    #[cfg(obsolete)]
    mod obsolete {
        use super::*;
        use crate::core::{
            midi::MidiNote,
            piano_roll::{Note, PatternBuilder},
        };
        use crate::Composer;
        use std::sync::{Arc, RwLock};

        #[test]
        fn live_sequencer_can_find_patterns() {
            let composer = Arc::new(RwLock::new(Composer::default()));
            let pattern_uid = composer
                .write()
                .unwrap()
                .add_pattern(
                    PatternBuilder::default()
                        .note(Note::new_with_midi_note(
                            MidiNote::C0,
                            MusicalTime::new_with_beats(0),
                            MusicalTime::DURATION_WHOLE,
                        ))
                        .note(Note::new_with_midi_note(
                            MidiNote::C0,
                            MusicalTime::ONE_BEAT,
                            MusicalTime::DURATION_WHOLE,
                        ))
                        .note(Note::new_with_midi_note(
                            MidiNote::C0,
                            MusicalTime::new_with_beats(2),
                            MusicalTime::DURATION_WHOLE,
                        ))
                        .note(Note::new_with_midi_note(
                            MidiNote::C0,
                            MusicalTime::new_with_beats(3),
                            MusicalTime::DURATION_WHOLE,
                        ))
                        .build()
                        .unwrap(),
                    None,
                )
                .unwrap();

            let mut s = LivePatternSequencer::new_with(&composer);
            let _ = s.record(
                MidiChannel::default(),
                &pattern_uid,
                MusicalTime::new_with_beats(20),
            );

            assert!(s.pattern_uid_for_position(MusicalTime::START).is_none());
            assert!(s.pattern_uid_for_position(MusicalTime::TIME_MAX).is_none());
            assert!(s
                .pattern_uid_for_position(MusicalTime::new_with_beats(19))
                .is_none());
            // I manually counted the length of the pattern to figure out that it was four beats long.
            assert!(s
                .pattern_uid_for_position(
                    MusicalTime::new_with_beats(20) + MusicalTime::new_with_beats(4)
                )
                .is_none());

            assert!(s
                .pattern_uid_for_position(MusicalTime::new_with_beats(20))
                .is_some());
            assert!(s
                .pattern_uid_for_position(MusicalTime::new_with_beats(24) - MusicalTime::ONE_UNIT)
                .is_some());

            s.clear();
        }
    }

    fn replay_units<MU>(
        sequences: &mut dyn Sequences<MU = MU>,
        start_time: MusicalTime,
        duration: MusicalTime,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        let mut v = Vec::default();
        sequences.update_time_range(&TimeRange(start_time..start_time + duration));
        sequences.work(&mut |event| match event {
            WorkEvent::Midi(channel, message) => v.push((channel, message)),
            WorkEvent::MidiForTrack(_, channel, message) => v.push((channel, message)),
            WorkEvent::Control(_) => panic!(),
        });
        v
    }

    fn replay_all_units<MU>(
        sequences: &mut dyn Sequences<MU = MU>,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        replay_units(sequences, MusicalTime::TIME_ZERO, MusicalTime::TIME_MAX)
    }

    /// Validates the provided implementation of [Sequences] for a [Note].
    pub fn validate_sequences_notes_trait(s: &mut dyn Sequences<MU = Note>) {
        const SAMPLE_NOTE: Note =
            Note::new_with(60, MusicalTime::START, MusicalTime::DURATION_QUARTER);
        const SAMPLE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);

        s.clear();

        assert!(replay_all_units(s).is_empty());
        assert!(s
            .record(SAMPLE_MIDI_CHANNEL, &SAMPLE_NOTE, MusicalTime::START)
            .is_ok());
        let message_count = replay_all_units(s).len();
        assert_eq!(
            message_count, 2,
            "After recording a Note, two new messages should be recorded."
        );

        assert!(s
            .remove(
                SAMPLE_MIDI_CHANNEL,
                &SAMPLE_NOTE,
                MusicalTime::START + MusicalTime::ONE_UNIT
            )
            .is_ok());
        assert_eq!(
            replay_all_units(s).len(),
            message_count,
            "Number of messages should remain unchanged after removing nonexistent Note"
        );

        assert!(s
            .remove(SAMPLE_MIDI_CHANNEL, &SAMPLE_NOTE, MusicalTime::START)
            .is_ok());
        assert!(
            replay_all_units(s).is_empty(),
            "Sequencer should be empty after removing last note"
        );
    }

    /// Validates the provided implementation of [Sequences] for a [Pattern].
    pub(crate) fn validate_sequences_patterns_trait(s: &mut dyn Sequences<MU = Pattern>) {
        const SAMPLE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);

        s.clear();

        {
            let pattern = PatternBuilder::default().build().unwrap();

            assert!(replay_all_units(s).is_empty());
            assert!(s
                .record(SAMPLE_MIDI_CHANNEL, &pattern, MusicalTime::START)
                .is_ok());
            let message_count = replay_all_units(s).len();
            assert_eq!(
                message_count, 0,
                "After recording an empty pattern, no new messages should be recorded."
            );
        }
        {
            let pattern = PatternBuilder::default()
                .note(Note::new_with_midi_note(
                    MidiNote::C0,
                    MusicalTime::new_with_beats(0),
                    MusicalTime::DURATION_WHOLE,
                ))
                .note(Note::new_with_midi_note(
                    MidiNote::C0,
                    MusicalTime::ONE_BEAT,
                    MusicalTime::DURATION_WHOLE,
                ))
                .note(Note::new_with_midi_note(
                    MidiNote::C0,
                    MusicalTime::new_with_beats(2),
                    MusicalTime::DURATION_WHOLE,
                ))
                .note(Note::new_with_midi_note(
                    MidiNote::C0,
                    MusicalTime::new_with_beats(3),
                    MusicalTime::DURATION_WHOLE,
                ))
                .build()
                .unwrap();

            assert!(s
                .record(SAMPLE_MIDI_CHANNEL, &pattern, MusicalTime::START)
                .is_ok());
            let message_count = replay_all_units(s).len();
            assert_eq!(
                message_count, 8,
                "After recording an pattern with four notes, eight new messages should be recorded."
            );

            assert!(s
                .remove(
                    SAMPLE_MIDI_CHANNEL,
                    &pattern,
                    MusicalTime::START + MusicalTime::ONE_UNIT
                )
                .is_ok());
            assert_eq!(
                replay_all_units(s).len(),
                message_count,
                "Number of messages should remain unchanged after removing nonexistent item"
            );

            assert!(s
                .remove(SAMPLE_MIDI_CHANNEL, &pattern, MusicalTime::START)
                .is_ok());
            assert!(
                replay_all_units(s).is_empty(),
                "Sequencer should be empty after removing pattern"
            );
        }
    }

    #[test]
    fn midi_sequencer_passes_trait_validation() {
        let mut s = MidiSequencer::default();

        validate_sequences_midi_trait(&mut s);
    }

    #[test]
    fn pattern_sequencer_passes_trait_validation() {
        let mut s = PatternSequencer::default();

        validate_sequences_patterns_trait(&mut s);
    }

    fn replay_messages(
        sequences_midi: &mut dyn SequencesMidi,
        start_time: MusicalTime,
        duration: MusicalTime,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        let mut v = Vec::default();
        sequences_midi.update_time_range(&TimeRange(start_time..start_time + duration));
        sequences_midi.work(&mut |event| match event {
            WorkEvent::Midi(channel, message) => v.push((channel, message)),
            WorkEvent::MidiForTrack(_, channel, message) => v.push((channel, message)),
            WorkEvent::Control(_) => panic!(),
        });
        v
    }

    fn replay_all_messages(
        sequences_midi: &mut dyn SequencesMidi,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        replay_messages(
            sequences_midi,
            MusicalTime::TIME_ZERO,
            MusicalTime::TIME_MAX,
        )
    }

    /// Validates the provided implementation of [SequencesMidi].
    pub fn validate_sequences_midi_trait(sequences: &mut dyn SequencesMidi) {
        const SAMPLE_NOTE_ON_MESSAGE: MidiMessage = MidiMessage::NoteOn {
            key: u7::from_int_lossy(60),
            vel: u7::from_int_lossy(100),
        };
        const SAMPLE_NOTE_OFF_MESSAGE: MidiMessage = MidiMessage::NoteOff {
            key: u7::from_int_lossy(60),
            vel: u7::from_int_lossy(100),
        };
        const SAMPLE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);

        assert!(replay_all_messages(sequences).is_empty());
        assert!(sequences
            .record_midi_message(
                SAMPLE_MIDI_CHANNEL,
                SAMPLE_NOTE_OFF_MESSAGE,
                MusicalTime::START
            )
            .is_ok());
        assert_eq!(
            replay_all_messages(sequences).len(),
            1,
            "sequencer should contain one recorded message"
        );
        sequences.clear();
        assert!(replay_all_messages(sequences).is_empty());

        assert!(
            sequences.is_finished(),
            "An empty sequencer should always be finished."
        );
        assert!(
            !sequences.is_performing(),
            "A sequencer should not be performing before play()"
        );

        let mut do_nothing = |_, _| {};

        assert!(!sequences.is_recording());
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_ON_MESSAGE,
            &mut do_nothing,
        );
        assert!(
            replay_all_messages(sequences).is_empty(),
            "sequencer should ignore incoming messages when not recording"
        );

        sequences.start_recording();
        assert!(sequences.is_recording());
        sequences.update_time_range(&TimeRange(
            MusicalTime::ONE_BEAT..MusicalTime::DURATION_QUARTER,
        ));
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_ON_MESSAGE,
            &mut do_nothing,
        );
        sequences.update_time_range(&TimeRange(
            MusicalTime::new_with_beats(2)..MusicalTime::DURATION_QUARTER,
        ));
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_OFF_MESSAGE,
            &mut do_nothing,
        );
        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should reflect recorded messages even while recording"
        );
        sequences.stop();
        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should reflect recorded messages after recording"
        );

        assert!(
            replay_messages(
                sequences,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_QUARTER,
            )
            .is_empty(),
            "sequencer should replay no events for time slice before recorded events"
        );

        assert_eq!(
            replay_messages(
                sequences,
                MusicalTime::ONE_BEAT,
                MusicalTime::DURATION_QUARTER,
            )
            .len(),
            1,
            "sequencer should produce appropriate messages for time slice"
        );

        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should produce appropriate messages for time slice"
        );
    }
}

#[cfg(obsolete)]
mod obsolete {
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct LivePatternArrangement {
        pattern_uid: PatternUid,
        range: Range<MusicalTime>,
    }

    #[derive(Debug, Default)]
    pub struct LivePatternSequencer {
        arrangements: Vec<LivePatternArrangement>,

        pub inner: PatternSequencer,
        composer: Arc<RwLock<Composer>>,
    }
    impl Sequences for LivePatternSequencer {
        type MU = PatternUid;

        fn record(
            &mut self,
            channel: MidiChannel,
            pattern_uid: &Self::MU,
            position: MusicalTime,
        ) -> anyhow::Result<()> {
            let composer = self.composer.read().unwrap();
            if let Some(pattern) = composer.pattern(*pattern_uid) {
                let _ = self.e.inner.record(channel, &pattern, position);
                self.arrangements.push(LivePatternArrangement {
                    pattern_uid: *pattern_uid,
                    range: position..position + pattern.duration(),
                });
                Ok(())
            } else {
                Err(anyhow!("couldn't find pattern {pattern_uid}"))
            }
        }

        fn remove(
            &mut self,
            _channel: MidiChannel,
            pattern_uid: &Self::MU,
            position: MusicalTime,
        ) -> anyhow::Result<()> {
            // Someday I will get https://en.wikipedia.org/wiki/De_Morgan%27s_laws right
            self.arrangements
                .retain(|a| a.pattern_uid != *pattern_uid || a.range.start != position);
            self.e.inner.clear();
            self.replay();
            Ok(())
        }

        fn clear(&mut self) {
            self.arrangements.clear();
            self.e.inner.clear();
        }
    }
    impl Controls for LivePatternSequencer {
        fn update_time_range(&mut self, range: &TimeRange) {
            self.e.inner.update_time_range(range)
        }

        fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
            // TODO: when you make the Entity wrapper for this, this code is where
            // you'll substitute in the real uid.
            let mut inner_control_events_fn = |event| {
                control_events_fn(event);
            };

            self.e.inner.work(&mut inner_control_events_fn)
        }

        fn is_finished(&self) -> bool {
            self.e.inner.is_finished()
        }

        fn play(&mut self) {
            self.e.inner.play()
        }

        fn stop(&mut self) {
            self.e.inner.stop()
        }

        fn skip_to_start(&mut self) {
            self.e.inner.skip_to_start()
        }

        fn is_performing(&self) -> bool {
            self.e.inner.is_performing()
        }
    }
    impl Serializable for LivePatternSequencer {
        fn after_deser(&mut self) {
            self.replay();
        }
    }
    impl Configurable for LivePatternSequencer {}
    impl HandlesMidi for LivePatternSequencer {}
    impl LivePatternSequencer {
        #[allow(unused_variables)]
        pub fn new_with(composer: &Arc<RwLock<Composer>>) -> Self {
            Self {
                composer: Arc::clone(composer),
                ..Default::default()
            }
        }

        fn replay(&mut self) {
            let composer = self.composer.read().unwrap();
            self.arrangements.iter().for_each(|arrangement| {
                if let Some(pattern) = composer.pattern(arrangement.pattern_uid) {
                    let _ = self.e.inner.record(
                        MidiChannel::default(),
                        pattern,
                        arrangement.range.start,
                    );
                }
            });
        }

        pub fn pattern_uid_for_position(&self, position: MusicalTime) -> Option<PatternUid> {
            if let Some(arrangement) = self
                .arrangements
                .iter()
                .find(|a| a.range.contains(&position))
            {
                Some(arrangement.pattern_uid)
            } else {
                None
            }
        }
    }
}
