// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Representation of a whole music project, including support for serialization.

use crate::{all_entities::EntityParams, composition::Pattern, prelude::*, types::TrackTitle};
use ensnare_core::sequence_repository::ArrangementInfo;
use ensnare_orchestration::midi_router::MidiConnectionInfo;
use serde::{Deserialize, Serialize};

/// The most commonly used imports.
pub mod prelude {
    pub use super::{DiskProject, ProjectTitle, TrackInfo};
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

/// A serializable representation of an entity's metadata.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(missing_docs)]
pub struct EntityInfo {
    pub uid: Uid,
    pub params: Box<EntityParams>,
}

/// A serializable representation of an collection of a track's entities.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(missing_docs)]
pub struct TrackEntities {
    pub track_uid: TrackUid,
    pub entities: Vec<EntityInfo>,
}

/// A serializable representation of a pattern.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(missing_docs)]
pub struct PatternInfo {
    pub pattern_uid: PatternUid,
    pub pattern: Pattern,
}

/// A serializable representation of a project. Most applications that use
/// [DiskProject] will need to create `From` implementations to/from their own
/// custom representation of the data contained within it.
///
/// Note that we use Vec<(key, value)> when it seems like HashMap<key, value>
/// would be a more natural choice. That's because DiskProject is not intended
/// to function as a live database, but rather as a static, ordered list of
/// things that are read and written sequentially. We certainly could have made
/// it a HashMap, but we'd lose the implicit ordering of Vecs, and we might
/// someday write code that expects the struct to be smarter than it should be.
///
/// We use the ...Info structs rather than tuples so that the serialized JSON
/// will have named fields, which increases readability.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DiskProject {
    /// The user-visible title of this project. Used only for display.
    pub title: ProjectTitle,

    /// The project's global [Tempo].
    pub tempo: Tempo,

    /// The project's global [TimeSignature].
    pub time_signature: TimeSignature,

    /// An ordered list of tracks in the order they appear in the UI.
    pub tracks: Vec<TrackInfo>,

    /// The entities in each track.
    pub entities: Vec<TrackEntities>,

    /// The MIDI connections for this project.
    pub midi_connections: Vec<MidiConnectionInfo>,

    /// Sequences of notes that can be reused elsewhere in the project.
    pub patterns: Vec<PatternInfo>,

    /// Patterns that have been arranged in tracks.
    pub arrangements: Vec<ArrangementInfo>,
}
