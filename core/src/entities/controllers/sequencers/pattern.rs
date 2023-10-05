// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::MidiSequencer;
use crate::{
    midi::{MidiChannel, MidiEvent},
    piano_roll::{Pattern, PatternUid, PianoRoll},
    time::MusicalTime,
    traits::{Configurable, ControlEventsFn, Controls, Sequences, SequencesMidi},
};
use anyhow::anyhow;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default)]
pub struct PatternSequencer {
    inner: MidiSequencer,
    patterns: Vec<Pattern>,
}
impl Sequences for PatternSequencer {
    type MU = Pattern;

    fn record(
        &mut self,
        channel: MidiChannel,
        pattern: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let pattern = pattern.clone() + position;
        let events: Vec<MidiEvent> = pattern.clone().into();
        events.iter().for_each(|&e| {
            let _ = self.inner.record_midi_event(channel, e);
        });
        self.patterns.push(pattern);
        Ok(())
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        pattern: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let pattern = pattern.clone() + position;
        let events: Vec<MidiEvent> = pattern.clone().into();
        events.iter().for_each(|&e| {
            let _ = self.inner.remove_midi_event(channel, e);
        });
        self.patterns.retain(|p| p != &pattern);
        Ok(())
    }

    fn clear(&mut self) {
        self.patterns.clear();
        self.inner.clear();
    }
}
impl Controls for PatternSequencer {
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
impl Configurable for PatternSequencer {}

#[derive(Debug, Default)]
pub struct LivePatternSequencer {
    inner: PatternSequencer,
    pattern_uids: Vec<(MusicalTime, PatternUid)>,
    piano_roll: Arc<RwLock<PianoRoll>>,
}
impl Sequences for LivePatternSequencer {
    type MU = PatternUid;

    fn record(
        &mut self,
        channel: MidiChannel,
        pattern_uid: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let piano_roll = self.piano_roll.read().unwrap();
        if let Some(pattern) = piano_roll.get_pattern(pattern_uid) {
            let _ = self.inner.record(channel, &pattern, position);
            self.pattern_uids.push((position, *pattern_uid));
            Ok(())
        } else {
            Err(anyhow!("couldn't find pattern {pattern_uid}"))
        }
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        pattern_uid: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        self.pattern_uids
            .retain(|(pos, uid)| *pos != position || *uid != *pattern_uid);
        self.inner.clear();
        self.replay();
        Ok(())
    }

    fn clear(&mut self) {
        self.pattern_uids.clear();
        self.inner.clear();
    }
}
impl Controls for LivePatternSequencer {
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
impl Configurable for LivePatternSequencer {}
impl LivePatternSequencer {
    pub fn new_with(piano_roll: Arc<RwLock<PianoRoll>>) -> Self {
        Self {
            inner: Default::default(),
            pattern_uids: Default::default(),
            piano_roll,
        }
    }

    fn replay(&mut self) {
        let piano_roll = self.piano_roll.read().unwrap();
        self.pattern_uids
            .iter()
            .for_each(|(position, pattern_uid)| {
                if let Some(pattern) = piano_roll.get_pattern(&pattern_uid) {
                    let _ = self.inner.record(MidiChannel(0), pattern, *position);
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::tests::{
        validate_sequences_live_patterns_trait, validate_sequences_patterns_trait,
    };
    use std::sync::{Arc, RwLock};

    #[test]
    fn sequencer_works() {
        let mut s = PatternSequencer::default();

        validate_sequences_patterns_trait(&mut s);
    }

    #[test]
    fn live_sequencer_works() {
        let piano_roll = Arc::new(RwLock::new(PianoRoll::default()));
        let mut s = LivePatternSequencer::new_with(Arc::clone(&piano_roll));

        validate_sequences_live_patterns_trait(piano_roll, &mut s);
    }
}
