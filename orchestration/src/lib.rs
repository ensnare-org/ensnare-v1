// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use egui::project_widget;

pub mod egui;
pub mod track;

pub mod prelude {
    pub use super::egui::{project_widget, ProjectAction};
}
