// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::midi::MidiSequencer;
use derive_builder::Builder;
use ensnare_core::{piano_roll::Pattern, prelude::*, traits::Sequences};
use serde::{Deserialize, Serialize};

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
    #[builder(setter(skip))]
    pub inner: MidiSequencer,

    #[builder(default, setter(each(name = "pattern", into)))]
    pub patterns: Vec<(MidiChannel, Pattern)>,
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
        self.patterns.push((channel, pattern));
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
        self.patterns
            .retain(|(c, p)| *c != channel || *p != pattern);
        Ok(())
    }

    fn clear(&mut self) {
        self.patterns.clear();
        self.inner.clear();
    }
}
impl Controls for PatternSequencer {
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
impl Serializable for PatternSequencer {
    fn after_deser(&mut self) {
        for (channel, pattern) in &self.patterns {
            let events: Vec<MidiEvent> = pattern.clone().into();
            events.iter().for_each(|&e| {
                let _ = self.inner.record_midi_event(*channel, e);
            });
        }
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
                let _ = self.inner.record(channel, &pattern, position);
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
            self.inner.clear();
            self.replay();
            Ok(())
        }

        fn clear(&mut self) {
            self.arrangements.clear();
            self.inner.clear();
        }
    }
    impl Controls for LivePatternSequencer {
        fn update_time_range(&mut self, range: &TimeRange) {
            self.inner.update_time_range(range)
        }

        fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
            // TODO: when you make the Entity wrapper for this, this code is where
            // you'll substitute in the real uid.
            let mut inner_control_events_fn = |event| {
                control_events_fn(event);
            };

            self.inner.work(&mut inner_control_events_fn)
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
                    let _ =
                        self.inner
                            .record(MidiChannel::default(), pattern, arrangement.range.start);
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
#[cfg(test)]
mod tests {
    #[cfg(obsolete)]
    mod obsolete {
        use super::*;
        use crate::Composer;
        use ensnare_core::{
            midi::MidiNote,
            piano_roll::{Note, PatternBuilder},
        };
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
                            MusicalTime::new_with_beats(1),
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
                .pattern_uid_for_position(
                    MusicalTime::new_with_beats(24) - MusicalTime::new_with_units(1)
                )
                .is_some());

            s.clear();
        }
    }
}
