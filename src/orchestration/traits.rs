// Copyright (c) 2024 Mike Tsao

use super::TrackUid;

/// The [Projects] trait specifies the common behavior of an Ensnare project,
/// which is everything that makes up a single musical work -- the tempo, the
/// time signature, the musical notes, the tracks, the instrument layouts and
/// configurations, etc. [Projects] is a trait because we have different
/// implementations of project behavior, depending on the use case.
///
/// Incidentally, the name "Projects" sounds awkward, but I looked up the
/// etymology of the word "project," and it originally meant "to cause to move
/// forward" in the sense of making an idea transform into reality. So saying
/// that a project projects is not totally strange.
pub trait Projects {
    /// Creates a new track, optionally assigning the given [TrackUid]. Returns
    /// the [TrackUid] of the new track. Specified [TrackUid]s must not
    /// duplicate one that already exists in the project.
    fn create_track(&mut self, uid: Option<TrackUid>) -> anyhow::Result<TrackUid>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::TrackUidFactory;
    use anyhow::anyhow;
    use std::collections::HashSet;

    /// [TestProject] is a harness that helps make the [Projects] trait
    /// ergonomic.
    #[derive(Default)]
    struct TestProject {
        track_uid_factory: TrackUidFactory,
        track_uids: HashSet<TrackUid>,
    }
    impl Projects for TestProject {
        fn create_track(&mut self, uid: Option<TrackUid>) -> anyhow::Result<TrackUid> {
            let uid = if let Some(uid) = uid {
                if self.track_uids.contains(&uid) {
                    return Err(anyhow!("Duplicate TrackUid"));
                }
                uid
            } else {
                self.track_uid_factory.mint_next()
            };

            self.track_uids.insert(uid);
            Ok(uid)
        }
    }

    #[test]
    fn trait_mainline() {
        let mut p = TestProject::default();
        let track_uid_1 = p.create_track(None).unwrap();
        let track_uid_2 = p.create_track(None).unwrap();
        assert_ne!(
            track_uid_1, track_uid_2,
            "create_track should generate unique IDs"
        );

        assert!(
            p.create_track(Some(track_uid_1)).is_err(),
            "create_track should disallow assignment of duplicate uids."
        )
    }
}
