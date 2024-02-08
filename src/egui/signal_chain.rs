// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::fill_remaining_ui_space;
use crate::prelude::*;
use eframe::egui::{Button, Frame, Sense, Widget};
use strum_macros::Display;

/// Utility
pub type SignalChainItem = (Uid, String, bool);

#[derive(Debug, Display)]
pub enum SignalChainWidgetAction {
    Select(Uid, String),
    Remove(Uid),
    NewDevice(EntityKey),
}

pub struct SignalChainWidget<'a> {
    items: &'a [SignalChainItem],
    action: &'a mut Option<SignalChainWidgetAction>,
}
impl<'a> SignalChainWidget<'a> {
    fn new(items: &'a [SignalChainItem], action: &'a mut Option<SignalChainWidgetAction>) -> Self {
        Self { items, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        items: &'a [SignalChainItem],

        action: &'a mut Option<SignalChainWidgetAction>,
    ) -> impl Widget + 'a {
        move |ui: &mut eframe::egui::Ui| SignalChainWidget::new(items, action).ui(ui)
    }
}
impl<'a> Widget for SignalChainWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (response, payload) = ui.dnd_drop_zone::<EntityKey>(Frame::default(), |ui| {
            ui.horizontal_centered(|ui| {
                self.items
                    .iter()
                    .for_each(|(uid, name, is_control_source)| {
                        let item_response = ui.add(SignalItemWidget::widget(
                            *uid,
                            name.clone(),
                            *is_control_source,
                        ));
                        ui.separator();
                        let _ = item_response.context_menu(|ui| {
                            if ui.button("Remove").clicked() {
                                *self.action = Some(SignalChainWidgetAction::Remove(*uid));
                            }
                        });
                        if item_response.clicked() {
                            *self.action =
                                Some(SignalChainWidgetAction::Select(*uid, name.clone()));
                        }
                    });
                fill_remaining_ui_space(ui);
            })
            .inner
        });
        if let Some(payload) = payload {
            *self.action = Some(SignalChainWidgetAction::NewDevice(payload.as_ref().clone()));
        }
        response
    }
}

struct SignalItemWidget {
    uid: Uid,
    name: String,
    is_control_source: bool,
}
impl SignalItemWidget {
    fn new(uid: Uid, name: String, is_control_source: bool) -> Self {
        Self {
            uid,
            name,
            is_control_source,
        }
    }

    fn widget(uid: Uid, name: String, is_control_source: bool) -> impl Widget {
        move |ui: &mut eframe::egui::Ui| SignalItemWidget::new(uid, name, is_control_source).ui(ui)
    }
}
impl Widget for SignalItemWidget {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if self.is_control_source {
            let response = ui.add(Button::new(&self.name).sense(Sense::click_and_drag()));
            // We do this rather than wrapping with Ui::dnd_drag_source()
            // because of https://github.com/emilk/egui/issues/2730.
            response.dnd_set_drag_payload(self.uid);
            response
        } else {
            ui.button(self.name)
        }
    }
}
