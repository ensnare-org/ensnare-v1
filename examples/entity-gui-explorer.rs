// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `entity-gui-explorer` example is a sandbox for developing the GUI part
//! of Ensnare entities.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use eframe::{
    egui::{self, warn_if_debug_build, CollapsingHeader, Layout, ScrollArea, Style},
    emath::Align,
    CreationContext,
};
use ensnare::{app_version, prelude::*};
use rustc_hash::FxHashMap;
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
    factory: EntityFactory<dyn EntityBounds>,
    sorted_keys: Vec<EntityKey>,
    selected_key: Option<EntityKey>,
    uid_factory: EntityUidFactory,
    display_mode: DisplayMode,
    entities: FxHashMap<EntityKey, Box<dyn EntityBounds>>,
}
impl EntityGuiExplorer {
    pub const NAME: &'static str = "Entity GUI Explorer";

    pub fn new(_cc: &CreationContext, factory: EntityFactory<dyn EntityBounds>) -> Self {
        let sorted_keys = Self::generate_entity_key_list(&factory);
        Self {
            factory,
            sorted_keys,
            ..Default::default()
        }
    }

    fn generate_entity_key_list(factory: &EntityFactory<dyn EntityBounds>) -> Vec<EntityKey> {
        // let skips = vec![EntityKey::from(ControlTrip::ENTITY_KEY)];
        let skips = [];

        let mut keys: Vec<String> = factory
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
                    if let Some(entity) = self.factory.new_entity(key, uid) {
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
    let options = eframe::NativeOptions::default();

    // We want to add internal entities here, so we do it here and then hand the
    // result to register_factory_entities().
    let factory = EntityFactory::<dyn EntityBounds>::default();
    let factory = BuiltInEntities::register(factory).finalize();

    if let Err(e) = eframe::run_native(
        EntityGuiExplorer::NAME,
        options,
        Box::new(|cc| Box::new(EntityGuiExplorer::new(cc, factory))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
