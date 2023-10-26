// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `entity-explorer` example is a sandbox for developing Ensnare [Entities](Entity).

use anyhow::anyhow;
use eframe::{
    egui::{self, warn_if_debug_build, CollapsingHeader, Layout, ScrollArea, Style, Ui},
    emath::Align,
    CreationContext,
};
use ensnare::{app_version, prelude::*};
use std::sync::atomic::{AtomicUsize, Ordering};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Default, EnumIter, Display, PartialEq)]
enum DisplayMode {
    #[default]
    Normal,
    WithHeader,
}
#[derive(Debug, Default)]
struct EntityExplorer {
    sorted_keys: Vec<EntityKey>,
    selected_key: Option<EntityKey>,
    entity: Option<Box<dyn Entity>>,
    next_uid: AtomicUsize,
    display_mode: DisplayMode,
}
impl EntityExplorer {
    pub const NAME: &'static str = "Entity Explorer";

    pub fn new(_cc: &CreationContext) -> Self {
        Self {
            sorted_keys: Self::generate_entity_key_list(),
            ..Default::default()
        }
    }

    fn generate_entity_key_list() -> Vec<EntityKey> {
        let mut keys: Vec<String> = EntityFactory::global()
            .keys()
            .into_iter()
            .map(|k| k.to_string())
            .collect();
        keys.sort();
        keys.into_iter().map(|k| EntityKey::from(k)).collect()
    }

    fn show_top(&mut self, ui: &mut Ui) {
        self.debug_ui(ui);
    }

    fn show_bottom(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            warn_if_debug_build(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(app_version())
            });
        });
    }

    fn show_left(&mut self, ui: &mut Ui) {
        for key in self.sorted_keys.iter() {
            if ui.button(key.to_string()).clicked() {
                if self.selected_key != Some(key.clone()) {
                    let uid = Uid(self.next_uid.fetch_add(1, Ordering::Relaxed));
                    self.entity = EntityFactory::global().new_entity(key, uid);
                    if self.entity.is_some() {
                        self.selected_key = Some(key.clone());
                    } else {
                        self.selected_key = None;
                    }
                }
            }
        }
    }

    fn debug_ui(&mut self, ui: &mut Ui) {
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

    fn show_right(&mut self, ui: &mut Ui) {
        for mode in DisplayMode::iter() {
            let s = mode.to_string();
            ui.radio_value(&mut self.display_mode, mode, s);
        }
        ScrollArea::horizontal().show(ui, |ui| ui.label("Under Construction"));
    }

    fn show_center(&mut self, ui: &mut Ui) {
        let available_height = ui.available_height();
        ScrollArea::vertical().show(ui, |ui| {
            ui.set_max_height(available_height / 2.0);
            if let Some(entity) = self.entity.as_mut() {
                ui.with_layout(
                    Layout::default().with_cross_align(Align::Center),
                    |ui| match self.display_mode {
                        DisplayMode::Normal => {
                            ui.group(|ui| entity.ui(ui));
                        }
                        DisplayMode::WithHeader => {
                            CollapsingHeader::new(entity.name())
                                .default_open(true)
                                .show_unindented(ui, |ui| entity.ui(ui));
                        }
                    },
                );
            } else {
                ui.with_layout(Layout::default().with_cross_align(Align::Center), |ui| {
                    ui.label("Click an entity in the sidebar");
                });
            }

            ui.allocate_space(ui.available_size_before_wrap());
        });
        ui.separator();
        if let Some(entity) = self.entity.as_mut() {
            ui.label(format!("{entity:?}"));
        }
    }
}
impl eframe::App for EntityExplorer {
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

    if EntityFactory::initialize(register_factory_entities(EntityFactory::default())).is_err() {
        return Err(anyhow!("Couldn't initialize EntityFactory"));
    }
    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        return Err(anyhow!("Couldn't set DragDropManager once_cell"));
    }

    if let Err(e) = eframe::run_native(
        EntityExplorer::NAME,
        options,
        Box::new(|cc| Box::new(EntityExplorer::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
