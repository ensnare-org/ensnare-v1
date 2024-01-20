// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::parts::{BusStation, EntityRepository, Humidifier, Mixer, TrackRepository};
use anyhow::Result;
use delegate::delegate;
use ensnare_core::{
    prelude::*,
    traits::{ControlProxyEventsFn, ControlsAsProxy},
};
use ensnare_entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, option::Option};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Orchestrator {
    pub track_repo: TrackRepository,
    pub entity_repo: EntityRepository,

    pub aux_track_uids: Vec<TrackUid>,
    pub bus_station: BusStation,
    pub humidifier: Humidifier,
    pub mixer: Mixer,
}
impl Orchestrator {
    delegate! {
        to self.track_repo {
            pub fn create_track(&mut self, uid: Option<TrackUid>) -> Result<TrackUid>;
            #[call(uids)]
            pub fn track_uids(&self) -> &[TrackUid];
            pub fn set_track_position(&mut self, uid: TrackUid, new_position: usize) -> Result<()>;
            pub fn mint_track_uid(&self) -> TrackUid;
        }
        to self.entity_repo {
            pub fn add_entity(
                &mut self,
                track_uid: TrackUid,
                entity: Box<dyn EntityBounds>,
                uid: Option<Uid>,
            ) -> Result<Uid>;
            pub fn move_entity(
                &mut self,
                uid: Uid,
                new_track_uid: Option<TrackUid>,
                new_position: Option<usize>,
            ) -> Result<()>;
            pub fn delete_entity(&mut self, uid: Uid) -> Result<()>;
            pub fn remove_entity(&mut self, uid: Uid) -> Result<Box<dyn EntityBounds>>;
            pub fn mint_entity_uid(&self) -> Uid;
        }
        to self.entity_repo.entities {
            #[call(get_mut)]
            pub fn get_entity_mut(&mut self, uid: &Uid) -> Option<&mut Box<(dyn EntityBounds)>>;
        }
        to self.bus_station {
            pub fn add_send(&mut self, src_uid: TrackUid, dst_uid: TrackUid, amount: Normal) -> anyhow::Result<()>;
            pub fn remove_send(&mut self, send_track_uid: TrackUid, aux_track_uid: TrackUid);
        }
        to self.humidifier {
            pub fn get_humidity(&self, uid: &Uid) -> Normal;
            pub fn set_humidity(&mut self, uid: Uid, humidity: Normal);
            pub fn transform_batch(
                &mut self,
                humidity: Normal,
                effect: &mut Box<dyn EntityBounds>,
                samples: &mut [StereoSample],
            );
        }
        to self.mixer {
            pub fn track_output(&mut self, track_uid: TrackUid) -> Normal;
            pub fn set_track_output(&mut self, track_uid: TrackUid, output: Normal);
            pub fn mute_track(&mut self, track_uid: TrackUid, muted: bool);
            pub fn is_track_muted(&mut self, track_uid: TrackUid) -> bool;
            pub fn solo_track(&self) -> Option<TrackUid>;
            pub fn set_solo_track(&mut self, track_uid: Option<TrackUid>);
        }
    }

    pub fn delete_track(&mut self, uid: TrackUid) -> Result<()> {
        self.bus_station.remove_sends_for_track(uid);
        self.track_repo.delete_track(uid)
    }

    pub fn entities_for_track(&self, uid: TrackUid) -> Option<&Vec<Uid>> {
        self.entity_repo.uids_for_track.get(&uid)
    }

    pub fn track_for_entity(&self, uid: Uid) -> Option<TrackUid> {
        self.entity_repo.track_for_uid.get(&uid).copied()
    }
}
impl Controls for Orchestrator {
    fn time_range(&self) -> Option<TimeRange> {
        self.entity_repo.time_range()
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.entity_repo.update_time_range(time_range)
    }

    fn is_finished(&self) -> bool {
        self.entity_repo.is_finished()
    }

    fn play(&mut self) {
        self.entity_repo.play();
    }

    fn stop(&mut self) {
        self.entity_repo.stop();
    }

    fn skip_to_start(&mut self) {
        self.entity_repo.skip_to_start()
    }

    fn is_performing(&self) -> bool {
        self.entity_repo.is_performing()
    }
}
impl ControlsAsProxy for Orchestrator {
    fn work_as_proxy(&mut self, control_events_fn: &mut ControlProxyEventsFn) {
        self.entity_repo.work_as_proxy(control_events_fn)
    }
}
impl Generates<StereoSample> for Orchestrator {
    fn generate(&mut self, values: &mut [StereoSample]) {
        let buffer_len = values.len();
        let solo_track_uid = self.solo_track();

        // First handle all non-aux tracks. As a side effect, we also create empty buffers for the aux tracks.
        let (track_buffers, mut aux_track_buffers): (
            HashMap<TrackUid, Vec<StereoSample>>,
            HashMap<TrackUid, Vec<StereoSample>>,
        ) = self.track_repo.uids.iter().fold(
            (HashMap::default(), HashMap::default()),
            |(mut h, mut aux_h), track_uid| {
                let mut track_buffer = Vec::default();
                track_buffer.resize(buffer_len, StereoSample::SILENCE);
                if self.aux_track_uids.contains(track_uid) {
                    aux_h.insert(*track_uid, track_buffer);
                } else {
                    let should_work = !self.mixer.is_track_muted(*track_uid)
                        && (solo_track_uid.is_none() || solo_track_uid == Some(*track_uid));
                    if should_work {
                        if let Some(entity_uids) = self.entity_repo.uids_for_track.get(track_uid) {
                            entity_uids.iter().for_each(|uid| {
                                if let Some(entity) = self.entity_repo.entities.get_mut(uid) {
                                    entity.generate(&mut track_buffer);
                                    let humidity = self.humidifier.get_humidity(uid);
                                    if humidity != Normal::zero() {
                                        self.humidifier.transform_batch(
                                            humidity,
                                            entity,
                                            &mut track_buffer,
                                        );
                                    }
                                }
                            });
                        }
                    }
                    h.insert(*track_uid, track_buffer);
                }
                (h, aux_h)
            },
        );

        // Then send audio to the aux tracks.
        for (track_uid, routes) in self.bus_station.sends() {
            // We have a source track_uid and the aux tracks that should receive it.
            if let Some(source_track_buffer) = track_buffers.get(track_uid) {
                // Mix the source into the destination aux track.
                for route in routes {
                    if let Some(aux) = aux_track_buffers.get_mut(&route.aux_track_uid) {
                        for (src, dst) in source_track_buffer.iter().zip(aux.iter_mut()) {
                            *dst += *src * route.amount;
                        }
                    }
                }
            }
        }

        // Let the aux tracks do their processing.
        aux_track_buffers
            .iter_mut()
            .for_each(|(track_uid, track_buffer)| {
                let should_work = !self.mixer.is_track_muted(*track_uid)
                    && (solo_track_uid.is_none() || solo_track_uid == Some(*track_uid));
                if should_work {
                    if let Some(entity_uids) = self.entity_repo.uids_for_track.get(track_uid) {
                        entity_uids.iter().for_each(|uid| {
                            if let Some(entity) = self.entity_repo.entities.get_mut(uid) {
                                entity.transform_batch(track_buffer);
                            }
                        });
                    }
                }
            });

        // Mix all the tracks into the final buffer.
        track_buffers
            .iter()
            .chain(aux_track_buffers.iter())
            .for_each(|(track_uid, buffer)| {
                let should_mix = !self.mixer.is_track_muted(*track_uid)
                    && (solo_track_uid.is_none() || solo_track_uid == Some(*track_uid));
                if should_mix {
                    let output = self.track_output(*track_uid);
                    for (dst, src) in values.iter_mut().zip(buffer) {
                        *dst += *src * output;
                    }
                }
            });
    }
}
impl Ticks for Orchestrator {}
impl Configurable for Orchestrator {
    fn sample_rate(&self) -> SampleRate {
        self.entity_repo.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.entity_repo.update_sample_rate(sample_rate)
    }

    fn tempo(&self) -> Tempo {
        self.entity_repo.tempo()
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.entity_repo.update_tempo(tempo)
    }

    fn time_signature(&self) -> TimeSignature {
        self.entity_repo.time_signature()
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.entity_repo.update_time_signature(time_signature)
    }
}
impl Serializable for Orchestrator {
    fn before_ser(&mut self) {
        self.track_repo.before_ser();
        self.entity_repo.before_ser();
    }

    fn after_deser(&mut self) {
        self.track_repo.after_deser();
        self.entity_repo.after_deser();
    }
}

#[cfg(test)]
mod tests {
    use ensnare_entities::instruments::TestInstrument;

    use super::*;

    #[test]
    fn orchestrator_mainline() {
        let mut orchestrator = Orchestrator::default();

        let nonexistent_track_uid = TrackUid(12345);
        assert!(
            orchestrator
                .entities_for_track(nonexistent_track_uid)
                .is_none(),
            "Getting track entities for nonexistent track should return None"
        );

        let track_uid = orchestrator.create_track(None).unwrap();
        assert!(
            orchestrator.entities_for_track(track_uid).is_none(),
            "Getting track entries for a track that exists but is empty should return None"
        );
        let target_uid = orchestrator
            .add_entity(track_uid, Box::new(TestInstrument::default()), None)
            .unwrap();
        assert_eq!(
            orchestrator.track_for_entity(target_uid).unwrap(),
            track_uid,
            "Added entity's track uid should be retrievable"
        );
        let track_entities = orchestrator.entities_for_track(track_uid).unwrap();
        assert_eq!(track_entities.len(), 1);
        assert!(track_entities.contains(&target_uid));

        assert!(
            orchestrator.get_entity_mut(&Uid(99999)).is_none(),
            "getting nonexistent entity should return None"
        );
        assert!(
            orchestrator.get_entity_mut(&target_uid).is_some(),
            "getting an entity should return it"
        );
    }
}
