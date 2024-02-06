// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::{DragDropManager, DragSource};
use crate::prelude::*;
use eframe::egui::Widget;

/// A tree view of devices that can be placed in tracks.
#[derive(Debug)]
pub struct EntityPaletteWidget<'a> {
    keys: &'a [EntityKey],
}
impl<'a> EntityPaletteWidget<'a> {
    fn new_with(keys: &'a [EntityKey]) -> Self {
        Self { keys }
    }
    pub fn widget(keys: &'a [EntityKey]) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| EntityPaletteWidget::new_with(keys).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for EntityPaletteWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            for key in self.keys {
                DragDropManager::drag_source(
                    ui,
                    eframe::egui::Id::new(key),
                    DragSource::NewDevice(key.to_string()),
                    |ui| ui.label(key.to_string()),
                );
            }
        })
        .response
    }
}
