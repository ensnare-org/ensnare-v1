// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    orchestration::{OldOrchestrator, Orchestrator, OrchestratorAction},
    track::{track_widget, TrackWidgetAction},
    traits::Orchestrates,
};
use eframe::{egui::Widget, epaint::Galley};
use ensnare_core::{piano_roll::PianoRoll, time::ViewRange, traits::Controls, types::TrackTitle};
use ensnare_cores_egui::{
    piano_roll::piano_roll,
    widgets::timeline::{self, TimelineIconStripAction},
};
use std::sync::Arc;

use super::{
    new_track_widget,
    track::{make_title_bar_galley, title_bar},
};

/// Wraps an [OrchestratesTraitWidget] as a [Widget](eframe::egui::Widget).
pub fn orchestrates_trait_widget<'a>(
    orchestrates: &'a mut impl Orchestrates,
    view_range: &'a mut ViewRange,
    action: &'a mut Option<OrchestratorAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        OrchestratesTraitWidget::new(orchestrates, view_range, action).ui(ui)
    }
}

/// An egui component that draws anything implementing [Orchestrates].
#[derive(Debug)]
struct OrchestratesTraitWidget<'a> {
    orchestrates: &'a mut dyn Orchestrates,
    view_range: &'a mut ViewRange,
    action: &'a mut Option<OrchestratorAction>,
}
impl<'a> eframe::egui::Widget for OrchestratesTraitWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut action = None;
        ui.add(timeline::timeline_icon_strip(&mut action));

        // The timeline needs to be aligned with the track content, so
        // we create an empty track title bar to match with the real
        // ones.
        ui.horizontal(|ui| {
            ui.add_enabled(false, title_bar(None));
            ui.add(timeline::legend(self.view_range));
        });

        // Create a scrolling area for all the tracks.
        eframe::egui::ScrollArea::vertical()
            .id_source("orchestrator-scroller")
            .show(ui, |ui| {
                for track_uid in self.orchestrates.track_uids() {
                    let font_galley: Option<Arc<Galley>> =
                        Some(make_title_bar_galley(ui, &TrackTitle::default()));
                    let mut action = None;
                    ui.add(new_track_widget(
                        *track_uid,
                        self.view_range.clone(),
                        None,
                        font_galley,
                        &mut action,
                    ));
                }
            });

        // suppress warning
        *self.action = None;

        ui.label("placeholder.........")
    }
}
impl<'a> OrchestratesTraitWidget<'a> {
    fn new(
        orchestrates: &'a mut impl Orchestrates,
        view_range: &'a mut ViewRange,
        action: &'a mut Option<OrchestratorAction>,
    ) -> Self {
        Self {
            orchestrates,
            view_range,
            action,
        }
    }
}

/// Wraps an [OrchestratorWidget] as a [Widget](eframe::egui::Widget).
pub fn orchestrator<'a>(orchestrator: &'a mut Orchestrator) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| OrchestratorWidget::new(orchestrator).ui(ui)
}

/// An egui component that draws an [Orchestrator].
#[derive(Debug)]
struct OrchestratorWidget<'a> {
    orchestrator: &'a mut Orchestrator,
}
impl<'a> eframe::egui::Widget for OrchestratorWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
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
impl<'a> OrchestratorWidget<'a> {
    pub fn new(orchestrator: &'a mut Orchestrator) -> Self {
        Self { orchestrator }
    }
}

/// Wraps an [OldOrchestratorWidget] as a [Widget](eframe::egui::Widget).
pub fn old_orchestrator<'a>(
    orchestrator: &'a mut OldOrchestrator,
    view_range: &'a mut ViewRange,
    is_piano_roll_visible: &'a mut bool,
    action: &'a mut Option<OrchestratorAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        OldOrchestratorWidget::new(orchestrator, view_range, is_piano_roll_visible, action).ui(ui)
    }
}

/// An egui component that draws an [Orchestrator].
#[derive(Debug)]
struct OldOrchestratorWidget<'a> {
    orchestrator: &'a mut OldOrchestrator,
    view_range: &'a mut ViewRange,
    is_piano_roll_visible: &'a mut bool,
    action: &'a mut Option<OrchestratorAction>,
}
impl<'a> eframe::egui::Widget for OldOrchestratorWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        eframe::egui::Window::new("Piano Roll")
            .open(self.is_piano_roll_visible)
            .default_width(ui.available_width())
            .anchor(
                eframe::emath::Align2::LEFT_BOTTOM,
                eframe::epaint::vec2(5.0, 5.0),
            )
            .show(ui.ctx(), |ui| {
                let mut inner = self.orchestrator.piano_roll.write().unwrap();
                let inner_piano_roll: &mut PianoRoll = &mut inner;
                ui.add(piano_roll(inner_piano_roll))
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
                    if let Some(track_uid) = self.orchestrator.inner.track_for_entity.get(uid) {
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
                    ui.add_enabled(false, title_bar(None));
                    ui.add(timeline::legend(self.view_range));
                });

                // Create a scrolling area for all the tracks.
                eframe::egui::ScrollArea::vertical()
                    .id_source("orchestrator-scroller")
                    .show(ui, |ui| {
                        for track_uid in self.orchestrator.inner.track_uids.iter() {
                            if let Some(track) = self.orchestrator.tracks.get_mut(track_uid) {
                                let is_selected =
                                    self.orchestrator.e.track_selection_set.contains(track_uid);
                                let cursor = if self.orchestrator.inner.transport.is_performing() {
                                    Some(self.orchestrator.inner.transport.current_time())
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
                                    *self.action =
                                        Some(OrchestratorAction::DoubleClickTrack(*track_uid));
                                } else if response.clicked() {
                                    *self.action = Some(OrchestratorAction::ClickTrack(*track_uid));
                                }
                            }
                        }
                    });
            })
            .response
    }
}
impl<'a> OldOrchestratorWidget<'a> {
    pub fn new(
        orchestrator: &'a mut OldOrchestrator,
        view_range: &'a mut ViewRange,
        is_piano_roll_visible: &'a mut bool,
        action: &'a mut Option<OrchestratorAction>,
    ) -> Self {
        Self {
            orchestrator,
            view_range,
            is_piano_roll_visible,
            action,
        }
    }
}
