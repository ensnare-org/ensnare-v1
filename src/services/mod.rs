// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Long-running services that are useful to a music application.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{ProjectService, ProjectServiceEvent, ProjectServiceInput};
}
pub use project::{ProjectService, ProjectServiceEvent, ProjectServiceInput};

mod project;
