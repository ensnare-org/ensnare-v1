// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{ImageButton, Widget};
use ensnare_core::uid::{TrackUid, Uid};
use ensnare_drag_drop::{DragDropManager, DragSource, DropTarget};
use strum_macros::Display;

pub type SignalChainItem = (Uid, String, bool);

/// Wraps a [NewSignalChainWidget] as a [Widget](eframe::egui::Widget).
pub fn new_signal_chain_widget<'a>(
    track_uid: TrackUid,
    items: &'a [SignalChainItem],

    action: &'a mut Option<NewSignalChainWidgetAction>,
) -> impl Widget + 'a {
    move |ui: &mut eframe::egui::Ui| NewSignalChainWidget::new(track_uid, items, action).ui(ui)
}

#[derive(Debug, Display)]
pub enum NewSignalChainWidgetAction {
    EntitySelected(Uid, String),
}

struct NewSignalChainWidget<'a> {
    track_uid: TrackUid,
    items: &'a [SignalChainItem],
    action: &'a mut Option<NewSignalChainWidgetAction>,
}
impl<'a> NewSignalChainWidget<'a> {
    pub fn new(
        track_uid: TrackUid,
        items: &'a [SignalChainItem],
        action: &'a mut Option<NewSignalChainWidgetAction>,
    ) -> Self {
        Self {
            track_uid,
            items,
            action,
        }
    }

    fn can_accept(&self) -> bool {
        if let Some(source) = DragDropManager::source() {
            matches!(source, DragSource::NewDevice(_))
        } else {
            false
        }
    }
}
impl<'a> Widget for NewSignalChainWidget<'a> {
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
                            if ui
                                .add(signal_item(*uid, name.clone(), *is_control_source))
                                .clicked()
                            {
                                *self.action = Some(NewSignalChainWidgetAction::EntitySelected(
                                    *uid,
                                    name.clone(),
                                ));
                            }
                        });
                    let _ = DragDropManager::drop_target(ui, self.can_accept(), |ui| {
                        (
                            ui.add_enabled(false, eframe::egui::Button::new("Drag Items Here")),
                            DropTarget::Track(self.track_uid),
                        )
                    })
                    .response;
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
                let icon = eframe::egui::include_image!(
                    "../../../res/images/md-symbols/drag_indicator.png"
                );
                let response = ui.button(&self.name);
                DragDropManager::drag_source(
                    ui,
                    eframe::egui::Id::new(self.name),
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
