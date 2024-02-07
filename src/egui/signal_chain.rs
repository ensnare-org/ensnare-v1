// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::{DragSource, DropTarget};
use crate::prelude::*;
use eframe::egui::{Frame, Image, ImageButton, Widget};
use strum_macros::Display;

pub type SignalChainItem = (Uid, String, bool);

pub fn signal_chain_widget<'a>(
    track_uid: TrackUid,
    items: &'a [SignalChainItem],

    action: &'a mut Option<SignalChainWidgetAction>,
) -> impl Widget + 'a {
    move |ui: &mut eframe::egui::Ui| SignalChainWidget::new(track_uid, items, action).ui(ui)
}

#[derive(Debug, Display)]
pub enum SignalChainWidgetAction {
    Select(Uid, String),
    Remove(Uid),
}

struct SignalChainWidget<'a> {
    track_uid: TrackUid,
    items: &'a [SignalChainItem],
    action: &'a mut Option<SignalChainWidgetAction>,
}
impl<'a> SignalChainWidget<'a> {
    pub fn new(
        track_uid: TrackUid,
        items: &'a [SignalChainItem],
        action: &'a mut Option<SignalChainWidgetAction>,
    ) -> Self {
        Self {
            track_uid,
            items,
            action,
        }
    }

    // fn can_accept(&self) -> bool {
    //     if let Some(source) = DragDropManager::source() {
    //         matches!(source, DragSource::NewDevice(_))
    //     } else {
    //         false
    //     }
    // }
}
impl<'a> Widget for SignalChainWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let stroke = ui.ctx().style().visuals.noninteractive().bg_stroke;
        let response = eframe::egui::Frame::default()
            .stroke(stroke)
            .inner_margin(eframe::egui::Margin::same(stroke.width / 2.0))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    self.items
                        .iter()
                        .for_each(|(uid, name, is_control_source)| {
                            let item_response =
                                ui.add(signal_item(*uid, name.clone(), *is_control_source));
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
                    let (r, payload) = ui.dnd_drop_zone::<DragSource>(Frame::default(), |ui| {
                        ui.add_enabled(
                            false,
                            ImageButton::new(
                                Image::new(eframe::egui::include_image!(
                                    "../../res/images/md-symbols/playlist_add_circle.png"
                                ))
                                .fit_to_original_size(1.0),
                            ),
                        );
                    });
                    if let Some(payload) = payload {
                        eprintln!("{payload:?}");
                        //DropTarget::Track(self.track_uid),
                    }
                    ui.allocate_space(ui.available_size());
                })
                .inner
            })
            .response;
        response
    }
}

/// Wraps a [SignalItem] as a [Widget](eframe::egui::Widget).
fn signal_item<'a>(uid: Uid, name: String, is_control_source: bool) -> impl Widget + 'a {
    move |ui: &mut eframe::egui::Ui| SignalItem::new(uid, name, is_control_source).ui(ui)
}

struct SignalItem {
    uid: Uid,
    name: String,
    is_control_source: bool,
}
impl SignalItem {
    fn new(uid: Uid, name: String, is_control_source: bool) -> Self {
        Self {
            uid,
            name,
            is_control_source,
        }
    }
}
impl Widget for SignalItem {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if self.is_control_source {
            ui.horizontal(|ui| {
                let icon = Image::new(eframe::egui::include_image!(
                    "../../res/images/md-symbols/drag_indicator.png"
                ))
                .fit_to_original_size(1.0);
                let response = ui.button(&self.name);
                ui.dnd_drag_source(
                    eframe::egui::Id::new(self.uid),
                    DragSource::ControlSource(self.uid),
                    |ui| ui.add(ImageButton::new(icon).tint(ui.ctx().style().visuals.text_color())),
                );
                response
            })
            .inner
        } else {
            ui.button(self.name)
        }
    }
}
