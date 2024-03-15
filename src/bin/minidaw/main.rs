// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! A digital audio workstation.

use anyhow::anyhow;
use eframe::egui::{ViewportBuilder, Visuals};
use eframe::{
    egui::{Context, FontData, FontDefinitions, TextStyle},
    epaint::{Color32, FontFamily, FontId},
};
use ensnare::prelude::*;
use env_logger;
use minidaw::MiniDaw;

mod events;
mod menu;
mod minidaw;
mod settings;

struct MiniDawVisuals {}
impl MiniDawVisuals {
    /// internal-only key for regular font.
    const FONT_REGULAR: &'static str = "font-regular";
    /// internal-only key for bold font.
    const FONT_BOLD: &'static str = "font-bold";
    /// internal-only key for monospaced font.
    const FONT_MONO: &'static str = "font-mono";
}

fn initialize_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        MiniDawVisuals::FONT_REGULAR.to_owned(),
        FontData::from_static(include_bytes!(
            "../../../res/fonts/jost/static/Jost-Regular.ttf"
        )),
    );
    fonts.font_data.insert(
        MiniDawVisuals::FONT_BOLD.to_owned(),
        FontData::from_static(include_bytes!(
            "../../../res/fonts/jost/static/Jost-Bold.ttf"
        )),
    );
    fonts.font_data.insert(
        MiniDawVisuals::FONT_MONO.to_owned(),
        FontData::from_static(include_bytes!(
            "../../../res/fonts/roboto-mono/RobotoMono-VariableFont_wght.ttf"
        )),
    );

    // Make these fonts the highest priority.
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, MiniDawVisuals::FONT_REGULAR.to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, MiniDawVisuals::FONT_MONO.to_owned());
    fonts
        .families
        .entry(FontFamily::Name(MiniDawVisuals::FONT_BOLD.into()))
        .or_default()
        .insert(0, MiniDawVisuals::FONT_BOLD.to_owned());

    ctx.set_fonts(fonts);
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
            FontId::new(20.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        (
            TextStyle::Monospace,
            FontId::new(16.0, FontFamily::Monospace),
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

    style.visuals = Visuals::dark();

    ctx.set_style(style);
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(MiniDaw::NAME)
            .with_inner_size(eframe::epaint::vec2(1280.0, 720.0))
            .to_owned(),
        vsync: true,
        centered: true,
        ..Default::default()
    };

    Paths::set_instance(Paths::default());
    SampleLibrary::set_instance(SampleLibrary::default());
    let factory = MiniDawEntities::register(EntityFactory::default()).finalize();

    if let Err(e) = eframe::run_native(
        MiniDaw::NAME,
        options,
        Box::new(|cc| {
            initialize_fonts(&cc.egui_ctx);
            initialize_visuals(&cc.egui_ctx);
            initialize_style(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MiniDaw::new(cc, factory))
        }),
    ) {
        return Err(anyhow!("eframe::run_native(): {:?}", e));
    }

    Ok(())
}
