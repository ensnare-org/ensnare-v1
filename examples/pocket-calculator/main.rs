// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Pocket Calculator is a streamlined groovebox.

mod calculator;

struct CalculatorApp {
    calculator: Calculator,
}
impl CalculatorApp {
    const APP_NAME: &'static str = "Pocket Calculator";

    fn new(cc: &CreationContext) -> Self {
        Self {
            calculator: Default::default(),
        }
    }
}
impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        todo!()
    }
}

use anyhow::anyhow;
use calculator::Calculator;
use eframe::CreationContext;
fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(1366.0, 768.0)),
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
