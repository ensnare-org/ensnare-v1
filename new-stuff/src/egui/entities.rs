// Copyright (c) 2024 Mike Tsao. All rights reserved.

use eframe::egui::Widget;
use ensnare_drag_drop::{DragDropManager, DragSource};
use ensnare_entity::factory::EntityKey;

/// Wraps an [EntityPaletteWidget] as a [Widget](eframe::egui::Widget).
pub fn entity_palette<'a>(keys: &'a [EntityKey]) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| EntityPaletteWidget::new_with(keys).ui(ui)
}

/// A tree view of devices that can be placed in tracks.
#[derive(Debug)]
struct EntityPaletteWidget<'a> {
    keys: &'a [EntityKey],
}
impl<'a> EntityPaletteWidget<'a> {
    pub fn new_with(keys: &'a [EntityKey]) -> Self {
        Self { keys }
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
