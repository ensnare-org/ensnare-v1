// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, option::Option};

/// A [BusRoute] represents a signal connection between two tracks.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BusRoute {
    /// The [TrackUid] of the receiving track.
    pub aux_track_uid: TrackUid,
    /// How much gain should be applied to this connection.
    pub amount: Normal,
}

/// A [BusStation] manages how signals move between tracks and aux tracks. These
/// collections of signals are sometimes called buses.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BusStation {
    pub routes: HashMap<TrackUid, Vec<BusRoute>>,
}

impl BusStation {
    pub fn add_send(
        &mut self,
        track_uid: TrackUid,
        dst_uid: TrackUid,
        amount: Normal,
    ) -> anyhow::Result<()> {
        self.routes.entry(track_uid).or_default().push(BusRoute {
            aux_track_uid: dst_uid,
            amount,
        });
        Ok(())
    }

    pub fn remove_send(&mut self, track_uid: TrackUid, aux_track_uid: TrackUid) {
        if let Some(routes) = self.routes.get_mut(&track_uid) {
            routes.retain(|route| route.aux_track_uid != aux_track_uid);
        }
    }

    pub fn sends(&self) -> impl Iterator<Item = (&TrackUid, &Vec<BusRoute>)> {
        self.routes.iter()
    }

    // If we want this method to be immutable and cheap, then we can't guarantee
    // that it will return a Vec. Such is life.
    #[allow(dead_code)]
    pub fn sends_for_track(&self, track_uid: &TrackUid) -> Option<&Vec<BusRoute>> {
        self.routes.get(track_uid)
    }

    pub(crate) fn remove_sends_for_track(&mut self, track_uid: TrackUid) {
        self.routes.remove(&track_uid);
        self.routes
            .values_mut()
            .for_each(|routes| routes.retain(|route| route.aux_track_uid != track_uid));
    }
}
