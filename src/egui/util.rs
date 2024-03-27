// Copyright (c) 2024 Mike Tsao. All rights reserved.

use core::fmt::Display;
use eframe::egui::{ComboBox, Response, Widget};
use strum::IntoEnumIterator;

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
