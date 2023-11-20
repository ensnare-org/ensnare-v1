// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use serde::{Deserialize, Serialize};

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
}
