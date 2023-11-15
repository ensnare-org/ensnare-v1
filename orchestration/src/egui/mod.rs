// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use orchestrator::{old_orchestrator, orchestrates_trait_widget, orchestrator};
pub use palette::entity_palette;
pub use signal_chain::new_signal_chain_widget;
pub use track::{make_title_bar_galley, new_track_widget, title_bar};

mod orchestrator;
mod palette;
mod signal_chain;
mod track;
