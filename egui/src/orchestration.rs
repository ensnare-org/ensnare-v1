// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    track::{track_widget, TrackWidgetAction},
    widgets::{
        timeline::{self, TimelineIconStripAction},
        track,
    },
};
use ensnare_core::{
    orchestration::OrchestratorAction,
    prelude::*,
    traits::{Acts, Controls, Displays, Orchestrates},
};

/// Wraps an [OrchestratorEgui] as a [Widget](eframe::egui::Widget).
pub fn orchestrator<'a>(orchestrator: &'a mut Orchestrator) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| OrchestratorEgui::new(orchestrator).ui(ui)
}

/// An egui component that draws an [Orchestrator].
#[derive(Debug)]
struct OrchestratorEgui<'a> {
    orchestrator: &'a mut Orchestrator,
}
impl<'a> Displays for OrchestratorEgui<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(format!(
            "There are {} tracks",
            self.orchestrator.track_uids().len()
        ));
        let add_track_button_response = ui.button("Add Track");
        if add_track_button_response.clicked() {
            let _ = self.orchestrator.create_track();
        }
        add_track_button_response
    }
}
impl<'a> OrchestratorEgui<'a> {
    pub fn new(orchestrator: &'a mut Orchestrator) -> Self {
        Self { orchestrator }
    }
}

/// Wraps an [OldOrchestratorEgui] as a [Widget](eframe::egui::Widget).
pub fn old_orchestrator<'a>(
    orchestrator: &'a mut OldOrchestrator,
    view_range: &'a mut ViewRange,
    is_piano_roll_visible: &'a mut bool,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        OldOrchestratorEgui::new(orchestrator, view_range, is_piano_roll_visible).ui(ui)
    }
}

/// An egui component that draws an [Orchestrator].
#[derive(Debug)]
struct OldOrchestratorEgui<'a> {
    orchestrator: &'a mut OldOrchestrator,
    view_range: &'a mut ViewRange,
    is_piano_roll_visible: &'a mut bool,
}
impl<'a> Displays for OldOrchestratorEgui<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        eframe::egui::Window::new("Piano Roll")
            .open(self.is_piano_roll_visible)
            .default_width(ui.available_width())
            .anchor(
                eframe::emath::Align2::LEFT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                self.orchestrator.piano_roll.write().unwrap().ui(ui)
            });

        eframe::egui::Window::new(&self.orchestrator.e.entity_detail_title)
            .id(eframe::egui::Id::from("Entity Detail"))
            .open(&mut self.orchestrator.e.is_entity_detail_open)
            .anchor(
                eframe::emath::Align2::RIGHT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                if let Some(uid) = &self.orchestrator.e.selected_entity_uid {
                    if let Some(track_uid) = self.orchestrator.entity_uid_to_track_uid.get(uid) {
                        if let Some(track) = self.orchestrator.tracks.get_mut(track_uid) {
                            if let Some(entity) = track.entity_mut(uid) {
                                entity.ui(ui);
                            }
                        }
                    }
                }
            });

        eframe::egui::CentralPanel::default()
            .show(ui.ctx(), |ui| {
                let mut action = None;
                ui.add(timeline::timeline_icon_strip(&mut action));
                if let Some(action) = action {
                    match action {
                        TimelineIconStripAction::NextTimelineView => {
                            panic!("get rid of this")
                        }
                        TimelineIconStripAction::ShowPianoRoll => {
                            *self.is_piano_roll_visible = !*self.is_piano_roll_visible;
                        }
                    }
                }

                // The timeline needs to be aligned with the track content, so
                // we create an empty track title bar to match with the real
                // ones.
                ui.horizontal(|ui| {
                    ui.add_enabled(false, track::title_bar(None));
                    ui.add(timeline::legend(self.view_range));
                });

                // Create a scrolling area for all the tracks.
                eframe::egui::ScrollArea::vertical()
                    .id_source("orchestrator-scroller")
                    .show(ui, |ui| {
                        let mut track_action = None;
                        let mut track_action_track_uid = None;
                        for track_uid in self.orchestrator.track_uids.iter() {
                            if let Some(track) = self.orchestrator.tracks.get_mut(track_uid) {
                                let is_selected =
                                    self.orchestrator.e.track_selection_set.contains(track_uid);
                                let cursor = if self.orchestrator.transport.is_performing() {
                                    Some(self.orchestrator.transport.current_time())
                                } else {
                                    None
                                };
                                //                                track.update_font_galley(ui);
                                let mut track_widget_action = None;
                                let response = ui.add(track_widget(
                                    *track_uid,
                                    track,
                                    is_selected,
                                    cursor,
                                    &self.view_range,
                                    &mut track_widget_action,
                                ));
                                if let Some(track_widget_action) = track_widget_action {
                                    match track_widget_action {
                                        TrackWidgetAction::EntitySelected(uid, name) => {
                                            self.orchestrator.e.selected_entity_uid = Some(uid);
                                            self.orchestrator.e.is_entity_detail_open = true;
                                            self.orchestrator.e.entity_detail_title = name;
                                        }
                                    }
                                }
                                if response.double_clicked() {
                                    self.orchestrator.e.action =
                                        Some(OrchestratorAction::DoubleClickTrack(*track_uid));
                                } else if response.clicked() {
                                    self.orchestrator.e.action =
                                        Some(OrchestratorAction::ClickTrack(*track_uid));
                                }

                                if let Some(action) = track.take_action() {
                                    track_action = Some(action);
                                    track_action_track_uid = Some(*track_uid);
                                }
                            }
                        }
                        if let Some(action) = track_action {
                            if let Some(track_uid) = track_action_track_uid {
                                self.orchestrator.handle_track_action(track_uid, action);
                            }
                        }
                    });
            })
            .response
    }
}
impl<'a> OldOrchestratorEgui<'a> {
    pub fn new(
        orchestrator: &'a mut OldOrchestrator,
        view_range: &'a mut ViewRange,
        is_piano_roll_visible: &'a mut bool,
    ) -> Self {
        Self {
            orchestrator,
            view_range,
            is_piano_roll_visible,
        }
    }
}
