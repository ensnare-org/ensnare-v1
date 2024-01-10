// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use palette::entity_palette;
pub use project::{orchestrator, project_widget, DescribesProject, ProjectAction};
pub use signal_chain::new_signal_chain_widget;
pub use track::{make_title_bar_galley, new_track_widget, title_bar};

mod palette;
mod project;
mod signal_chain;
mod track;
