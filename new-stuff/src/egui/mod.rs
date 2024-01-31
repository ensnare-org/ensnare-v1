// Copyright (c) 2024 Mike Tsao. All rights reserved.

pub use composer::ComposerWidget;
pub use cursor::CursorWidget;
pub use entities::EntityPaletteWidget;
pub use grid::GridWidget;
pub use legend::LegendWidget;
pub use project::{ProjectAction, ProjectWidget};
pub use timeline::{TimelineIconStripAction, TimelineIconStripWidget};
pub use track::{make_title_bar_galley, TitleBarWidget, TrackWidget};

mod composer;
mod cursor;
mod entities;
mod grid;
mod legend;
mod project;
mod signal_chain;
mod timeline;
mod track;
