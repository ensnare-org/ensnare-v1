// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Pocket Calculator is a streamlined groovebox.

use anyhow::anyhow;
use calculator::Calculator;
use eframe::CreationContext;
use ensnare::traits::prelude::*;

mod calculator;

struct CalculatorApp {
    calculator: Calculator,
}
impl CalculatorApp {
    const APP_NAME: &'static str = "Pocket Calculator";

    fn new(_cc: &CreationContext) -> Self {
        Self {
            calculator: Default::default(),
        }
    }
}
impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let center = eframe::egui::CentralPanel::default();

        center.show(ctx, |ui| self.calculator.ui(ui));
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(320.0, 560.0)),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        CalculatorApp::APP_NAME,
        options,
        Box::new(|cc| Box::new(CalculatorApp::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
