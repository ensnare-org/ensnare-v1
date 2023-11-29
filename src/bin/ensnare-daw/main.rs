// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A digital audio workstation.

use anyhow::anyhow;
use ensnare::Ensnare;
use ensnare_drag_drop::DragDropManager;
use ensnare_entities::register_factory_entities;
use ensnare_entity::factory::EntityFactory;
use env_logger;

mod ensnare;
mod menu;
mod project;
mod settings;
mod entities;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions::default();

    let mut factory = EntityFactory::default();
    register_factory_entities(&mut factory);
    if EntityFactory::initialize(factory).is_err() {
        panic!("Couldn't set EntityFactory once_cell");
    }
    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        panic!("Couldn't set DragDropManager once_cell");
    }

    if let Err(e) = eframe::run_native(
        Ensnare::NAME,
        options,
        Box::new(|cc| Box::new(Ensnare::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
