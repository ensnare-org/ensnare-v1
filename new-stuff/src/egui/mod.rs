// Copyright (c) 2024 Mike Tsao. All rights reserved.

pub use composer::ComposerWidget;
pub use cursor::CursorWidget;
pub use entities::entity_palette;
pub use grid::GridWidget;
pub use legend::LegendWidget;
pub use project::{ProjectAction, ProjectWidget};
pub use timeline::{timeline_icon_strip, TimelineIconStrip, TimelineIconStripAction};
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
