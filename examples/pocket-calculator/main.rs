// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `pocket-calculator` example is a simple groovebox. It demonstrates using
//! the `ensnare` crate without an [Orchestrator].

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
            Box::new(move |audio_queue, samples_requested| {
                let mut buffer = [StereoSample::SILENCE; 64];

                for _ in 0..(samples_requested / buffer.len()) + 1 {
                    if let Ok(mut calculator) = c4na.lock() {
                        // This is a lot of redundant calculation for something that
                        // doesn't change much, but it's cheap.
                        let range = MusicalTime::START
                            ..MusicalTime::new_with_units(MusicalTime::frames_to_units(
                                calculator.tempo(),
                                calculator.sample_rate(),
                                buffer.len(),
                            ));

                        calculator.update_time(&range);
                        calculator.work(&mut |_, _| {});
                        calculator.generate_batch_values(&mut buffer);
                        for sample in buffer {
                            let _ = audio_queue.push(sample);
                        }
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

        // We're being lazy and always requesting a repaint, even though we
        // don't know whether anything changed on-screen.
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.audio_interface.exit();
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(348.0, 576.0)),
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