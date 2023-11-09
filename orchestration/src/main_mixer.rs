// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{types::Normal, uid::TrackUid};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct MainMixer {
    pub track_output: HashMap<TrackUid, Normal>,
    pub track_mute: HashMap<TrackUid, bool>,
    pub solo_track: Option<TrackUid>,
}
impl MainMixer {
    pub fn set_track_output(&mut self, track_uid: TrackUid, output: Normal) {
        self.track_output.insert(track_uid, output);
    }

    pub fn mute_track(&mut self, track_uid: TrackUid, muted: bool) {
        self.track_mute.insert(track_uid, muted);
    }

    pub fn solo_track(&self) -> Option<TrackUid> {
        self.solo_track
    }

    pub fn set_solo_track(&mut self, track_uid: TrackUid) {
        self.solo_track = Some(track_uid)
    }

    pub fn end_solo(&mut self) {
        self.solo_track = None
    }
}
