// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Pocket Calculator is a streamlined groovebox.

use anyhow::anyhow;
use calculator::Calculator;
use eframe::CreationContext;
use ensnare::{
    panels::{AudioPanel, NeedsAudioFn},
    prelude::*,
    traits::prelude::*,
};
use std::sync::{Arc, Mutex};

mod calculator;

struct CalculatorApp {
    calculator: Arc<Mutex<Calculator>>,
    audio_interface: AudioPanel,
}
impl CalculatorApp {
    const APP_NAME: &'static str = "Pocket Calculator";

    fn new(_cc: &CreationContext) -> Self {
        let calculator = Arc::new(Mutex::new(Calculator::default()));
        let c4na = Arc::clone(&calculator);
        let needs_audio_fn: NeedsAudioFn = {
            Box::new(move |audio_queue, _| {
                let mut buffer = [StereoSample::SILENCE; 64];
                if let Ok(mut calculator) = c4na.lock() {
                    let range = MusicalTime::START..MusicalTime::DURATION_EIGHTH;
                    calculator.update_time(&range);
                    calculator.work(&mut |_, _| {});
                    calculator.generate_batch_values(&mut buffer);
                    for sample in buffer {
                        let _ = audio_queue.push(sample);
                    }
                }
            })
        };
        Self {
            calculator,
            audio_interface: AudioPanel::new_with(needs_audio_fn),
        }
    }
}
impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let center = eframe::egui::CentralPanel::default();

        center.show(ctx, |ui| {
            if let Ok(mut calculator) = self.calculator.lock() {
                calculator.ui(ui);
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.audio_interface.exit();
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
