// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A digital audio workstation.

use ::ensnare::all_entities::EntityWrapper;
use anyhow::anyhow;
use ensnare::Ensnare;
use ensnare_drag_drop::DragDropManager;
use ensnare_entity::factory::EntityFactory;
use env_logger;
use factory::EnsnareEntities;

mod ensnare;
mod factory;
mod menu;
mod project;
mod settings;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions::default();

    let factory =
        EnsnareEntities::register(EntityFactory::<dyn EntityWrapper>::default()).finalize();

    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        panic!("Couldn't set DragDropManager once_cell");
    }

    if let Err(e) = eframe::run_native(
        Ensnare::NAME,
        options,
        Box::new(|cc| Box::new(Ensnare::new(cc, factory))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
