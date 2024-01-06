// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A digital audio workstation.

//use ::ensnare::all_entities::{EnsnareEntities, EntityWrapper};
use ::ensnare::all_entities::EnsnareEntities2;
use anyhow::anyhow;
use eframe::egui::ViewportBuilder;
use eframe::{
    egui::{
        CentralPanel, Context, Event, FontData, FontDefinitions, Layout, Modifiers, ScrollArea,
        SidePanel, TextStyle, TopBottomPanel,
    },
    emath::Align,
    epaint::{Color32, FontFamily, FontId},
    App, CreationContext,
};
use ensnare::Ensnare;
use ensnare_drag_drop::DragDropManager;
use ensnare_entity::factory::EntityFactory;
use env_logger;

mod ensnare;
mod events;
mod menu;
mod settings;

struct EnsnareVisuals {}
impl EnsnareVisuals {
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
        EnsnareVisuals::FONT_REGULAR.to_owned(),
        FontData::from_static(include_bytes!(
            "../../../res/fonts/jost/static/Jost-Regular.ttf"
        )),
    );
    fonts.font_data.insert(
        EnsnareVisuals::FONT_BOLD.to_owned(),
        FontData::from_static(include_bytes!(
            "../../../res/fonts/jost/static/Jost-Bold.ttf"
        )),
    );
    fonts.font_data.insert(
        EnsnareVisuals::FONT_MONO.to_owned(),
        FontData::from_static(include_bytes!(
            "../../../res/fonts/roboto-mono/RobotoMono-VariableFont_wght.ttf"
        )),
    );

    // Make these fonts the highest priority.
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, EnsnareVisuals::FONT_REGULAR.to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, EnsnareVisuals::FONT_MONO.to_owned());
    fonts
        .families
        .entry(FontFamily::Name(EnsnareVisuals::FONT_BOLD.into()))
        .or_default()
        .insert(0, EnsnareVisuals::FONT_BOLD.to_owned());

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
            FontId::new(16.0, FontFamily::Proportional),
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

    ctx.set_style(style);
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(Ensnare::NAME)
            .with_inner_size(eframe::epaint::vec2(1280.0, 720.0))
            .to_owned(),
        ..Default::default()
    };

    let factory = EnsnareEntities2::register(EntityFactory::default()).finalize();

    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        panic!("Couldn't set DragDropManager once_cell");
    }

    if let Err(e) = eframe::run_native(
        Ensnare::NAME,
        options,
        Box::new(|cc| {
            initialize_fonts(&cc.egui_ctx);
            initialize_visuals(&cc.egui_ctx);
            initialize_style(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(Ensnare::new(cc, factory))
        }),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
