// Copyright (c) 2024 Mike Tsao. All rights reserved.

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

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(keys: &'a [EntityKey]) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| EntityPaletteWidget::new_with(keys).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for EntityPaletteWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.vertical(|ui| {
            for key in self.keys {
                ui.dnd_drag_source(eframe::egui::Id::new(key), key.clone(), |ui| {
                    ui.label(key.to_string())
                });
            }
        })
        .response
    }
}
