// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A digital audio workstation.

use anyhow::anyhow;
use eframe::{
    egui::{CentralPanel, Context, FontData, FontDefinitions, ScrollArea, TextStyle},
    epaint::{Color32, FontFamily, FontId},
    App, CreationContext,
};
use ensnare::prelude::*;
use env_logger;

#[derive(Debug, Default)]
struct Application {}
impl Application {
    const NAME: &'static str = "Ensnare";

    pub const FONT_REGULAR: &'static str = "font-regular";
    pub const FONT_BOLD: &'static str = "font-bold";
    pub const FONT_MONO: &'static str = "font-mono";

    fn new(cc: &CreationContext) -> Self {
        Self::initialize_fonts(cc);
        Self::initialize_visuals(&cc.egui_ctx);
        Self::initialize_style(&cc.egui_ctx);
        Self {}
    }

    fn initialize_fonts(cc: &CreationContext) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            Self::FONT_REGULAR.to_owned(),
            FontData::from_static(include_bytes!(
                "../../../res/fonts/jost/static/Jost-Regular.ttf"
            )),
        );
        fonts.font_data.insert(
            Self::FONT_BOLD.to_owned(),
            FontData::from_static(include_bytes!(
                "../../../res/fonts/jost/static/Jost-Bold.ttf"
            )),
        );
        fonts.font_data.insert(
            Self::FONT_MONO.to_owned(),
            FontData::from_static(include_bytes!(
                "../../../res/fonts/cousine/Cousine-Regular.ttf"
            )),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, Self::FONT_REGULAR.to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, Self::FONT_MONO.to_owned());
        fonts
            .families
            .entry(FontFamily::Name(Self::FONT_BOLD.into()))
            .or_default()
            .insert(0, Self::FONT_BOLD.to_owned());

        cc.egui_ctx.set_fonts(fonts);
    }

    /// Sets the default visuals.
    fn initialize_visuals(ctx: &Context) {
        let mut visuals = ctx.style().visuals.clone();

        // It's better to set text color this way than to change
        // Visuals::override_text_color because override_text_color overrides
        // dynamic highlighting when hovering over interactive text.
        visuals.widgets.noninteractive.fg_stroke.color = Color32::LIGHT_GRAY;
        ctx.set_visuals(visuals);
    }

    fn initialize_style(ctx: &Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(16.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
            (
                TextStyle::Monospace,
                FontId::new(14.0, FontFamily::Monospace),
            ),
            (
                TextStyle::Button,
                FontId::new(16.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(14.0, FontFamily::Proportional),
            ),
        ]
        .into();

        ctx.set_style(style);
    }
}
impl App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let center = CentralPanel::default();
        center.show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.label("Ensnare");
            });
        });
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(1366.0, 768.0)),
        ..Default::default()
    };

    if EntityFactory::initialize(register_factory_entities(EntityFactory::default())).is_err() {
        return Err(anyhow!("Couldn't set EntityFactory once_cell"));
    }
    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        return Err(anyhow!("Couldn't set DragDropManager once_cell"));
    }

    if let Err(e) = eframe::run_native(
        Application::NAME,
        options,
        Box::new(|cc| Box::new(Application::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
