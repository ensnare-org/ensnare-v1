// Copyright (c) 2024 Mike Tsao. All rights reserved.

use eframe::epaint::Color32;
use ensnare_core::types::ColorScheme;

pub struct ColorSchemeConverter {}
impl ColorSchemeConverter {
    pub fn to_color32(color_scheme: ColorScheme) -> (Color32, Color32) {
        match color_scheme {
            // https://www.rapidtables.com/web/color/RGB_Color.html
            // https://www.sttmedia.com/colornames
            ColorScheme::Red => (Color32::BLACK, Color32::from_rgb(255, 153, 153)),
            ColorScheme::Vermilion => (Color32::BLACK, Color32::from_rgb(255, 178, 153)),
            ColorScheme::Orange => (Color32::BLACK, Color32::from_rgb(255, 204, 153)),
            ColorScheme::Amber => (Color32::BLACK, Color32::from_rgb(255, 229, 153)),
            ColorScheme::Yellow => (Color32::BLACK, Color32::from_rgb(254, 255, 153)),
            ColorScheme::Lime => (Color32::BLACK, Color32::from_rgb(229, 255, 153)),
            ColorScheme::Chartreuse => (Color32::BLACK, Color32::from_rgb(204, 255, 153)),
            ColorScheme::Ddahal => (Color32::BLACK, Color32::from_rgb(178, 255, 153)),
            ColorScheme::Green => (Color32::BLACK, Color32::from_rgb(153, 255, 153)),
            ColorScheme::Erin => (Color32::BLACK, Color32::from_rgb(153, 255, 178)),
            ColorScheme::Spring => (Color32::BLACK, Color32::from_rgb(153, 255, 204)),
            ColorScheme::Gashyanta => (Color32::BLACK, Color32::from_rgb(153, 255, 229)),
            ColorScheme::Cyan => (Color32::BLACK, Color32::from_rgb(153, 254, 255)),
            ColorScheme::Capri => (Color32::BLACK, Color32::from_rgb(153, 229, 255)),
            ColorScheme::Azure => (Color32::BLACK, Color32::from_rgb(153, 203, 255)),
            ColorScheme::Cerulean => (Color32::BLACK, Color32::from_rgb(153, 178, 255)),
            ColorScheme::Blue => (Color32::BLACK, Color32::from_rgb(153, 153, 255)),
            ColorScheme::Volta => (Color32::BLACK, Color32::from_rgb(178, 153, 255)),
            ColorScheme::Violet => (Color32::BLACK, Color32::from_rgb(203, 153, 255)),
            ColorScheme::Llew => (Color32::BLACK, Color32::from_rgb(229, 153, 255)),
            ColorScheme::Magenta => (Color32::BLACK, Color32::from_rgb(255, 153, 254)),
            ColorScheme::Cerise => (Color32::BLACK, Color32::from_rgb(255, 153, 229)),
            ColorScheme::Rose => (Color32::BLACK, Color32::from_rgb(255, 153, 204)),
            ColorScheme::Crimson => (Color32::BLACK, Color32::from_rgb(255, 153, 178)),
            ColorScheme::Gray1 => (Color32::WHITE, Color32::from_rgb(0, 0, 0)),
            ColorScheme::Gray2 => (Color32::WHITE, Color32::from_rgb(32, 32, 32)),
            ColorScheme::Gray3 => (Color32::WHITE, Color32::from_rgb(64, 64, 64)),
            ColorScheme::Gray4 => (Color32::WHITE, Color32::from_rgb(96, 96, 96)),
            ColorScheme::Gray5 => (Color32::WHITE, Color32::from_rgb(128, 128, 128)),
            ColorScheme::Gray6 => (Color32::BLACK, Color32::from_rgb(160, 160, 160)),
            ColorScheme::Gray7 => (Color32::BLACK, Color32::from_rgb(192, 192, 192)),
            ColorScheme::Gray8 => (Color32::BLACK, Color32::from_rgb(224, 224, 224)),
        }
    }
}
