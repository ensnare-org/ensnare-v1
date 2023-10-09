// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    drag_drop::{DragDropManager, DragDropSource},
    entities::prelude::*,
    traits::prelude::*,
};

/// A tree view of devices that can be placed in tracks.
#[derive(Debug, Default)]
pub struct PalettePanel {}
impl Displays for PalettePanel {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            for key in EntityFactory::global().sorted_keys() {
                DragDropManager::drag_source(
                    ui,
                    eframe::egui::Id::new(key),
                    DragDropSource::NewDevice(key.clone()),
                    |ui| {
                        ui.label(key.to_string());
                    },
                );
            }
        })
        .response
    }
}
