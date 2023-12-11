// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{LivePatternSequencer, LivePatternSequencerParams};
use delegate::delegate;
use ensnare_core::{
    piano_roll::{PatternUid, PianoRoll},
    prelude::*,
    sequence_repository::{Sequence, SequenceRepository},
    traits::Sequences,
};
use ensnare_proc_macros::{InnerConfigurable, InnerHandlesMidi, InnerSerializable, Params};
use std::sync::{Arc, RwLock};

#[derive(Debug, Default, Params, InnerConfigurable, InnerHandlesMidi, InnerSerializable)]
pub struct ThinSequencer {
    repository: Arc<RwLock<SequenceRepository>>,
    repo_serial: usize,
    pub inner: LivePatternSequencer,
    track_uid: TrackUid,
    is_refreshed: bool,
}
impl ThinSequencer {
    pub fn new_with(
        _params: &ThinSequencerParams,
        track_uid: TrackUid,
        repository: &Arc<RwLock<SequenceRepository>>,
        piano_roll: &Arc<RwLock<PianoRoll>>,
    ) -> Self {
        Self {
            repository: Arc::clone(repository),
            inner: LivePatternSequencer::new_with(
                &LivePatternSequencerParams::default(),
                piano_roll,
            ),
            track_uid,
            ..Default::default()
        }
    }

    fn rebuild_sequencer(&mut self) {
        let repository = self.repository.read().unwrap();
        self.inner.clear();

        if let Some(sequence_uids) = repository.track_to_sequence_uids.get(&self.track_uid) {
            sequence_uids.iter().for_each(|sequence_uid| {
                if let Some((_, position, sequence)) = repository.sequences.get(sequence_uid) {
                    match sequence.as_ref() {
                        Sequence::Pattern(pattern_uid) => {
                            let _ =
                                self.inner
                                    .record(MidiChannel::default(), pattern_uid, *position);
                        }
                        Sequence::Note(_notes) => {
                            todo!("Design issue: must every sequencer handle all Sequence types?")
                        }
                    }
                }
            });
        }
    }

    // TODO: for Displays as well
    fn check_repo_for_changes(&mut self) {
        if self
            .repository
            .read()
            .unwrap()
            .has_changed(&mut self.repo_serial)
        {
            self.is_refreshed = false;
        }
    }
}
impl Controls for ThinSequencer {
    delegate! {
        to self.inner {
            fn time_range(&self) -> Option<TimeRange>;
            fn work(&mut self, control_events_fn: &mut ControlEventsFn);
            fn is_finished(&self) -> bool;
            fn play(&mut self);
            fn stop(&mut self);
            fn skip_to_start(&mut self);
            fn is_performing(&self) -> bool;
        }
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.check_repo_for_changes();
        if !self.is_refreshed {
            self.rebuild_sequencer();
            self.is_refreshed = true;
        }
        self.inner.update_time_range(time_range)
    }
}
impl Sequences for ThinSequencer {
    type MU = PatternUid;

    delegate! {
            to self.inner {
        fn record(
            &mut self,
            channel: MidiChannel,
            unit: &Self::MU,
            position: MusicalTime,
        ) -> anyhow::Result<()>;
        fn remove(
            &mut self,
            channel: MidiChannel,
            unit: &Self::MU,
            position: MusicalTime,
        ) -> anyhow::Result<()>;
        fn clear(&mut self);
    }
        }
}

#[cfg(test)]
mod tests {

    #[test]
    fn thin_sequencer() {}
}
