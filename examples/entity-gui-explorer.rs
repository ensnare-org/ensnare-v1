// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `entity-gui-explorer` example is a sandbox for developing the GUI part
//! of Ensnare [Entities](Entity).

use anyhow::anyhow;
use eframe::{
    egui::{self, warn_if_debug_build, CollapsingHeader, Layout, ScrollArea, Style},
    emath::Align,
    CreationContext,
};
use ensnare::{app_version, prelude::*};
use ensnare_core::uid::EntityUidFactory;
use ensnare_entity::traits::Entity; // TODO clean up
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Default, EnumIter, Display, PartialEq)]
enum DisplayMode {
    #[default]
    Normal,
    WithHeader,
}
#[derive(Debug, Default)]
struct EntityGuiExplorer {
    sorted_keys: Vec<EntityKey>,
    selected_key: Option<EntityKey>,
    uid_factory: EntityUidFactory,
    display_mode: DisplayMode,
    entities: HashMap<EntityKey, Box<dyn Entity>>,
}
impl EntityGuiExplorer {
    pub const NAME: &'static str = "Entity GUI Explorer";

    pub fn new(_cc: &CreationContext) -> Self {
        Self {
            sorted_keys: Self::generate_entity_key_list(),
            ..Default::default()
        }
    }

    fn generate_entity_key_list() -> Vec<EntityKey> {
        // let skips = vec![EntityKey::from(ControlTrip::ENTITY_KEY)];
        let skips = vec![];

        let mut keys: Vec<String> = EntityFactory::global()
            .keys()
            .iter()
            .filter(|k| !skips.contains(k))
            .map(|k| k.to_string())
            .collect();
        keys.sort();
        keys.into_iter().map(EntityKey::from).collect()
    }

    fn show_top(&mut self, ui: &mut eframe::egui::Ui) {
        self.debug_ui(ui);
    }

    fn show_bottom(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            warn_if_debug_build(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(app_version())
            });
        });
    }

    fn show_left(&mut self, ui: &mut eframe::egui::Ui) {
        for key in self.sorted_keys.iter() {
            if ui.button(key.to_string()).clicked() && self.selected_key != Some(key.clone()) {
                if !self.entities.contains_key(key) {
                    let uid = self.uid_factory.mint_next();
                    if let Some(entity) = EntityFactory::global().new_entity(key, uid) {
                        self.entities.insert(key.clone(), entity);
                    } else {
                        panic!("Couldn't create new entity {key}")
                    }
                }
                self.selected_key = Some(key.clone());
            }
        }
    }

    fn debug_ui(&mut self, ui: &mut eframe::egui::Ui) {
        #[cfg(debug_assertions)]
        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "🐛 Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }
        let style: Style = (*ui.ctx().style()).clone();
        let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
        if let Some(visuals) = new_visuals {
            ui.ctx().set_visuals(visuals);
        }
    }

    fn show_right(&mut self, ui: &mut eframe::egui::Ui) {
        for mode in DisplayMode::iter() {
            let s = mode.to_string();
            ui.radio_value(&mut self.display_mode, mode, s);
        }
    }

    fn show_center(&mut self, ui: &mut eframe::egui::Ui) {
        let available_height = ui.available_height();
        ScrollArea::vertical().show(ui, |ui| {
            ui.set_max_height(available_height / 2.0);
            if let Some(key) = self.selected_key.as_ref() {
                if let Some(entity) = self.entities.get_mut(key) {
                    ui.with_layout(Layout::default().with_cross_align(Align::Center), |ui| {
                        match self.display_mode {
                            DisplayMode::Normal => {
                                ui.vertical(|ui| {
                                    ui.group(|ui| entity.ui(ui));
                                });
                            }
                            DisplayMode::WithHeader => {
                                CollapsingHeader::new(entity.name())
                                    .default_open(true)
                                    .show_unindented(ui, |ui| entity.ui(ui));
                            }
                        }
                    });
                }
            } else {
                ui.with_layout(Layout::default().with_cross_align(Align::Center), |ui| {
                    ui.label("Click an entity in the sidebar");
                });
            }

            ui.allocate_space(ui.available_size_before_wrap());
        });
        ui.separator();
        if let Some(key) = self.selected_key.as_ref() {
            if let Some(entity) = self.entities.get_mut(key) {
                ui.label(format!("{entity:?}"));
            }
        }
    }
}
impl eframe::App for EntityGuiExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let top = egui::TopBottomPanel::top("top-panel")
            .resizable(false)
            .exact_height(64.0);
        let bottom = egui::TopBottomPanel::bottom("bottom-panel")
            .resizable(false)
            .exact_height(24.0);
        let left = egui::SidePanel::left("left-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0);
        let right = egui::SidePanel::right("right-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0);
        let center = egui::CentralPanel::default();

        top.show(ctx, |ui| {
            self.show_top(ui);
        });
        bottom.show(ctx, |ui| {
            self.show_bottom(ui);
        });
        left.show(ctx, |ui| {
            self.show_left(ui);
        });
        right.show(ctx, |ui| {
            self.show_right(ui);
        });
        center.show(ctx, |ui| {
            self.show_center(ui);
        });
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1366.0, 768.0)),
        ..Default::default()
    };

    // We want to add internal entities here, so we do it here and then hand the
    // result to register_factory_entities().
    let mut factory = EntityFactory::default();
    factory.register_entity_with_str_key(
        ensnare_factory_entities::piano_roll::PianoRoll::ENTITY_KEY,
        |uid| Box::new(ensnare_factory_entities::piano_roll::PianoRoll::new(uid)),
    );
    register_factory_entities(&mut factory);
    factory.complete_registration();
    if EntityFactory::initialize(factory).is_err() {
        return Err(anyhow!("Couldn't initialize EntityFactory"));
    }
    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        return Err(anyhow!("Couldn't set DragDropManager once_cell"));
    }

    if let Err(e) = eframe::run_native(
        EntityGuiExplorer::NAME,
        options,
        Box::new(|cc| Box::new(EntityGuiExplorer::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
