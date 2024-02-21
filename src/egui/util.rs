// Copyright (c) 2024 Mike Tsao. All rights reserved.

use core::fmt::Display;
use eframe::egui::{ComboBox, Frame, Response, Widget};
use std::sync::Arc;
use strum::IntoEnumIterator;

// See https://github.com/emilk/egui/issues/4059 for why this
// code is a bit cumbersome
pub fn dnd_drop_zone_with_inner_response<Payload>(
    ui: &mut eframe::egui::Ui,
    add_contents: impl FnOnce(&mut eframe::egui::Ui) -> Response,
) -> (Option<Response>, Response, Option<Arc<Payload>>)
where
    Payload: core::any::Any + Send + Sync,
{
    let mut inner_response = None;
    let (mut response, payload) = ui.dnd_drop_zone::<Payload>(Frame::default(), |ui| {
        inner_response = Some(add_contents(ui));
    });
    if let Some(inner_response) = inner_response.as_ref() {
        response |= inner_response.clone();
    }
    (inner_response, response, payload)
}

// Call this last in any ui() body if you want to fill the remaining space.
pub fn fill_remaining_ui_space(ui: &mut eframe::egui::Ui) {
    ui.allocate_space(ui.available_size());
}

#[derive(Debug)]
pub struct EnumComboBoxWidget<'a, E>
where
    E: IntoEnumIterator + PartialEq + Display,
{
    inner: &'a mut E,
    label: &'a str,
}
impl<'a, E> EnumComboBoxWidget<'a, E>
where
    E: IntoEnumIterator + PartialEq + Display,
{
    pub fn new(e: &'a mut E, label: &'a str) -> Self {
        Self { inner: e, label }
    }
}
impl<'a, E> Widget for EnumComboBoxWidget<'a, E>
where
    E: IntoEnumIterator + PartialEq + Display,
{
    fn ui(self, ui: &mut eframe::egui::Ui) -> Response {
        let current_str = self.inner.to_string();
        ComboBox::from_label(self.label)
            .selected_text(current_str)
            .show_ui(ui, |ui| {
                for item in E::iter() {
                    let item_str = item.to_string();
                    ui.selectable_value(self.inner, item, item_str);
                }
            })
            .response
    }
}
