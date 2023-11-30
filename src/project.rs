// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Representation of a whole music project, including support for serialization.

use crate::{all_entities::EntityParams, prelude::*, types::TrackTitle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod prelude {
    pub use super::{Project, ProjectTitle, TrackInfo};
}

/// A user-visible project title.
#[derive(Clone, Debug, derive_more::Display, PartialEq, Serialize, Deserialize)]
pub struct ProjectTitle(String);
impl Default for ProjectTitle {
    fn default() -> Self {
        Self("Untitled".to_string())
    }
}
impl From<ProjectTitle> for String {
    fn from(value: ProjectTitle) -> Self {
        value.0
    }
}
impl From<&str> for ProjectTitle {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// A serializable representation of a track's metadata.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[allow(missing_docs)]
pub struct TrackInfo {
    pub uid: TrackUid,
    pub title: TrackTitle,
}

/// A serializable representation of a project. Most applications that use
/// [Project] will need to create `From` implementations to/from their own
/// custom representation of the data contained within it.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Project {
    /// The user-visible title of this project. Used only for display.
    pub title: ProjectTitle,

    /// The project's global [Tempo].
    pub tempo: Tempo,

    /// The project's global [TimeSignature].
    pub time_signature: TimeSignature,

    /// The next Uid that EntityUidFactory should assign.
    pub entity_uid_factory_next_uid: usize,

    /// The next Uid that TrackUidFactory should assign.
    pub track_uid_factory_next_uid: usize,

    /// An ordered list of tracks in the order they appear in the UI.
    pub tracks: Vec<TrackInfo>,

    /// The entities in each track.
    pub entities: HashMap<TrackUid, Vec<(Uid, Box<EntityParams>)>>,
}
