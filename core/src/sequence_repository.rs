// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, ops::Deref, sync::atomic::AtomicUsize};

/// Identifies a [Sequence].
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SequenceUid(pub usize);
impl IsUid for SequenceUid {
    fn as_usize(&self) -> usize {
        self.0
    }
}
impl From<usize> for SequenceUid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for SequenceUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
pub type SequenceUidFactory = UidFactory<SequenceUid>;
impl UidFactory<SequenceUid> {
    pub const FIRST_UID: AtomicUsize = AtomicUsize::new(1);
}
impl Default for UidFactory<SequenceUid> {
    fn default() -> Self {
        Self {
            next_uid_value: Self::FIRST_UID,
            _phantom: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sequence {
    Pattern(PatternUid),
    Note(Vec<Note>),
}

/// [SequenceRepository] stores data that sequencers turn into MIDI events.
#[derive(Debug, Derivative, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct SequenceRepository {
    uid_factory: SequenceUidFactory,
    pub sequences: HashMap<SequenceUid, (TrackUid, MusicalTime, Box<Sequence>)>,
    pub track_to_sequence_uids: HashMap<TrackUid, Vec<SequenceUid>>,

    // Each time something changes in the repo, this number will change. Use the
    // provided methods to manage a local copy of it and decide whether to act.
    //
    // We start at something other than usize::default() so that
    // everyone else can use the default value and fire their update
    // code on the first call to has_changed().
    #[derivative(Default(value = "1000"))]
    mod_serial: usize,
}
impl SequenceRepository {
    pub fn add(
        &mut self,
        sequence: Sequence,
        start_time: MusicalTime,
        track_uid: TrackUid,
    ) -> anyhow::Result<SequenceUid> {
        let uid = self.uid_factory.mint_next();
        self.add_with_uid(uid, sequence, start_time, track_uid)
    }

    pub fn add_with_uid(
        &mut self,
        uid: SequenceUid,
        sequence: Sequence,
        start_time: MusicalTime,
        track_uid: TrackUid,
    ) -> anyhow::Result<SequenceUid> {
        self.sequences
            .insert(uid.clone(), (track_uid, start_time, Box::new(sequence)));
        self.track_to_sequence_uids
            .entry(track_uid)
            .or_default()
            .push(uid.clone());
        self.notify_change();
        self.uid_factory.notify_externally_minted_uid(uid.clone());
        Ok(uid)
    }

    pub fn remove(&mut self, sequence_uid: SequenceUid) {
        if let Some((track_uid, _, _)) = self.sequences.get(&sequence_uid) {
            if let Some(sequence_uids) = self.track_to_sequence_uids.get_mut(&track_uid) {
                sequence_uids.retain(|uid| sequence_uid != *uid);
            }
            self.notify_change();
            self.sequences.remove(&sequence_uid);
        }
    }

    pub fn notify_change(&mut self) {
        self.mod_serial += 1;
    }

    /// Use like this:
    ///
    /// ```no_run
    /// use ensnare_core::sequence_repository::SequenceRepository;
    ///
    /// let repo = SequenceRepository::default();
    /// let mut repo_serial = 0;
    ///
    /// if repo.has_changed(&mut repo_serial) {
    ///     // Update local data
    /// } else {
    ///     // We're up to date, nothing to do    
    /// }
    /// ```
    pub fn has_changed(&self, last_known: &mut usize) -> bool {
        let has_changed = self.mod_serial != *last_known;
        *last_known = self.mod_serial;
        has_changed
    }
}

/// A serializable representation of a track's arrangements.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct ArrangementInfo {
    pub track_uid: TrackUid,
    pub arranged_sequences: Vec<ArrangedSequenceInfo>,
}

/// A serializable representation of an arrangement.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct ArrangedSequenceInfo {
    pub sequence_uid: SequenceUid,
    pub channel: MidiChannel,
    pub position: MusicalTime,
    pub pattern_uid: PatternUid,
}

impl From<&Vec<ArrangementInfo>> for SequenceRepository {
    fn from(value: &Vec<ArrangementInfo>) -> Self {
        let mut r = Self::default();
        value.iter().for_each(|arrangement| {
            arrangement.arranged_sequences.iter().for_each(|sequence| {
                if let Ok(_) = r.add_with_uid(
                    sequence.sequence_uid.clone(),
                    Sequence::Pattern(sequence.pattern_uid),
                    sequence.position,
                    arrangement.track_uid,
                ) {
                    // is it done at this point? Concern that ThinSequencer
                    // doesn't know the sequence IDs, so how can it allow
                    // the user to edit one?
                }
            });
        });
        r
    }
}
impl From<&SequenceRepository> for Vec<ArrangementInfo> {
    fn from(value: &SequenceRepository) -> Self {
        let mut v_tracks = Vec::default();
        value.track_to_sequence_uids.keys().for_each(|track_uid| {
            if let Some(sequences) = value.track_to_sequence_uids.get(track_uid) {
                let mut v_sequences = Vec::default();
                sequences.iter().for_each(|sequence_uid| {
                    if let Some((_, position, sequence)) = value.sequences.get(sequence_uid) {
                        if let Sequence::Pattern(pattern_uid) = sequence.deref() {
                            v_sequences.push(ArrangedSequenceInfo {
                                sequence_uid: sequence_uid.clone(),
                                channel: MidiChannel::default(),
                                position: *position,
                                pattern_uid: *pattern_uid,
                            });
                        }
                    };
                });
                v_tracks.push(ArrangementInfo {
                    track_uid: *track_uid,
                    arranged_sequences: v_sequences,
                });
            }
        });
        v_tracks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::TimeRange;

    #[test]
    fn sequence_repository() {
        let track_1 = TrackUid(1);
        let track_2 = TrackUid(2);
        let mut sr = SequenceRepository::default();
        assert!(
            sr.sequences.is_empty(),
            "New SequenceRepository should be empty"
        );

        let pattern_sequence = Sequence::Pattern(PatternUid(1));
        let pattern_sequence_uid = sr
            .add(pattern_sequence, MusicalTime::DURATION_EIGHTH, track_1)
            .unwrap();
        assert_eq!(
            sr.sequences.len(),
            1,
            "Adding a sequence should increase count"
        );

        let note_sequence_1 = Sequence::Note(vec![
            Note {
                key: 1,
                range: TimeRange::new_with_start_and_duration(
                    MusicalTime::START,
                    MusicalTime::DURATION_QUARTER,
                ),
            },
            Note {
                key: 2,
                range: TimeRange::new_with_start_and_duration(
                    MusicalTime::START + MusicalTime::DURATION_HALF,
                    MusicalTime::DURATION_QUARTER,
                ),
            },
        ]);
        let note_sequence_2 = Sequence::Note(vec![]);
        let note_sequence_uid_1 = sr
            .add(note_sequence_1, MusicalTime::START, track_2)
            .unwrap();
        let note_sequence_uid_2 = sr
            .add(note_sequence_2, MusicalTime::START, track_2)
            .unwrap();
        assert_ne!(
            note_sequence_uid_1, note_sequence_uid_2,
            "Factory should mint unique IDs"
        );
        assert_eq!(
            sr.sequences.len(),
            3,
            "Adding a sequence should increase count"
        );

        let track_1_sequences = sr.track_to_sequence_uids.get(&track_1).unwrap();
        assert_eq!(
            track_1_sequences.len(),
            1,
            "Retrieval of one track should match count"
        );
        assert_eq!(
            track_1_sequences[0], pattern_sequence_uid,
            "Retrieval of track Uids should match expected"
        );
        let track_2_sequences = sr.track_to_sequence_uids.get(&track_2).unwrap();
        assert_eq!(
            track_2_sequences.len(),
            2,
            "Retrieval of one track should match count"
        );
        assert_eq!(
            track_2_sequences[0], note_sequence_uid_1,
            "Retrieval of track Uids should match expected numbers and order"
        );
        assert_eq!(
            track_2_sequences[1], note_sequence_uid_2,
            "Retrieval of track Uids should match expected numbers and order"
        );

        assert_eq!(sr.sequences.len(), 3);
        sr.remove(pattern_sequence_uid);
        assert_eq!(sr.sequences.len(), 2);
        sr.remove(note_sequence_uid_1);
        assert_eq!(sr.sequences.len(), 1);
        sr.remove(note_sequence_uid_2);
        assert!(sr.sequences.is_empty());
    }

    #[test]
    fn change_tracking() {
        let mut repo = SequenceRepository::default();
        let mut my_repo_tracker = 0;

        assert!(repo.has_changed(&mut my_repo_tracker));
        assert!(!repo.has_changed(&mut my_repo_tracker));

        assert!(repo
            .add(
                Sequence::Pattern(PatternUid(123)),
                MusicalTime::default(),
                TrackUid(456)
            )
            .is_ok());
        assert!(repo.has_changed(&mut my_repo_tracker));
    }
}
