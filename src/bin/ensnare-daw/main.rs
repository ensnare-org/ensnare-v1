// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A digital audio workstation.

use anyhow::anyhow;
use ensnare::Ensnare;
use env_logger;

mod ensnare;
mod menu;
mod project;
mod settings;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions::default();

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
