// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{Composer, LivePatternSequencer};
use delegate::delegate;
use ensnare_core::{piano_roll::PatternUid, prelude::*, traits::Sequences};
use ensnare_proc_macros::{InnerConfigurable, InnerHandlesMidi, InnerSerializable, Params};
use std::sync::{Arc, RwLock};

#[derive(Debug, Default, Params, InnerConfigurable, InnerHandlesMidi, InnerSerializable)]
pub struct ThinSequencer {
    composer: Arc<RwLock<Composer>>,
    repo_serial: usize,
    pub inner: LivePatternSequencer,
    track_uid: TrackUid,
    is_refreshed: bool,
}
impl ThinSequencer {
    pub fn new_with(track_uid: TrackUid, composer: &Arc<RwLock<Composer>>) -> Self {
        Self {
            composer: Arc::clone(composer),
            inner: LivePatternSequencer::new_with(composer),
            track_uid,
            ..Default::default()
        }
    }

    fn rebuild_sequencer(&mut self) {
        let composer = self.composer.read().unwrap();
        self.inner.clear();

        // Run through all the arrangements for this track.
        if let Some(arrangements) = composer.arrangements.get(&self.track_uid) {
            arrangements.iter().for_each(|arrangement| {
                let _ = self.inner.record(
                    MidiChannel::default(),
                    &arrangement.pattern_uid,
                    arrangement.position,
                );
            });
        }
    }

    // TODO: for Displays as well
    fn check_repo_for_changes(&mut self) {
        if self
            .composer
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
