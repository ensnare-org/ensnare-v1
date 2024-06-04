// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::fill_remaining_ui_space;
use crate::{orchestration::SignalChainItem, prelude::*};
use eframe::egui::{Button, Frame, Sense, Widget};
use ensnare::prelude::*;
use strum_macros::Display;

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
        let (response, payload) = ui.dnd_drop_zone::<EntityKey, _>(Frame::default(), |ui| {
            ui.horizontal_centered(|ui| {
                self.items.iter().for_each(|item| {
                    let item_response = ui.add(SignalItemWidget::widget(
                        item.name.clone(),
                        item.is_control_source,
                    ));

                    // We do this rather than wrapping with Ui::dnd_drag_source()
                    // because of https://github.com/emilk/egui/issues/2730.
                    item_response.dnd_set_drag_payload(ControlLinkSource::Entity(item.uid));

                    ui.separator();

                    let _ = item_response.context_menu(|ui| {
                        if ui.button("Remove").clicked() {
                            ui.close_menu();
                            *self.action = Some(SignalChainWidgetAction::Remove(item.uid));
                        }
                    });
                    if item_response.clicked() {
                        *self.action =
                            Some(SignalChainWidgetAction::Select(item.uid, item.name.clone()));
                    }
                });
                fill_remaining_ui_space(ui);
            })
            .response
        });
        if let Some(payload) = payload {
            *self.action = Some(SignalChainWidgetAction::NewDevice(payload.as_ref().clone()));
        }
        response.response
    }
}

struct SignalItemWidget {
    name: String,
    is_control_source: bool,
}
impl SignalItemWidget {
    fn new(name: String, is_control_source: bool) -> Self {
        Self {
            name,
            is_control_source,
        }
    }

    fn widget(name: String, is_control_source: bool) -> impl Widget {
        move |ui: &mut eframe::egui::Ui| SignalItemWidget::new(name, is_control_source).ui(ui)
    }
}
impl Widget for SignalItemWidget {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if self.is_control_source {
            ui.add(Button::new(&self.name).sense(Sense::click_and_drag()))
        } else {
            ui.button(self.name)
        }
    }
}
