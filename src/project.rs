// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::Orchestrator;
use serde::{Deserialize, Serialize};

/// A user-visible project title.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// A serializable representation of a project.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Project {
    title: ProjectTitle,
}
impl Project {
    /// Returns the in-memory components that make up a [Project].
    pub fn deserialize(&self) -> anyhow::Result<(Orchestrator, ProjectTitle)> {
        anyhow::Ok((Orchestrator::default(), self.title.clone()))
    }

    /// Creates a new [Project] from the given components.
    pub fn serialize(_orchestrator: &Orchestrator, title: &ProjectTitle) -> anyhow::Result<Self> {
        let r = Self {
            title: title.clone(),
        };
        Ok(r)
    }
}
