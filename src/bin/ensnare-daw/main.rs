// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A digital audio workstation.

use anyhow::anyhow;
use ensnare::Ensnare;
use ensnare_drag_drop::DragDropManager;
use env_logger;
use factory::EnsnareEntityFactory;

mod ensnare;
mod factory;
mod menu;
mod project;
mod settings;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions::default();

    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        panic!("Couldn't set DragDropManager once_cell");
    }

    if let Err(e) = eframe::run_native(
        Ensnare::NAME,
        options,
        Box::new(|cc| Box::new(Ensnare::new(cc, EnsnareEntityFactory::register_entities()))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
