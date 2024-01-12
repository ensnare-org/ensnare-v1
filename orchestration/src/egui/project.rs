// Copyright (c) 2023 Mike Tsao. All rights reserved.

#[cfg(obsolete)]
mod obsolete {
    pub trait DescribesProject: core::fmt::Debug {
        fn track_title(&self, track_uid: &TrackUid) -> Option<&TrackTitle>;
        fn track_frontmost_timeline_displayer(&self, track_uid: &TrackUid) -> Option<Uid>;
    }

    use super::{
        new_track_widget,
        signal_chain::SignalChainItem,
        track::{make_title_bar_galley, title_bar, TrackInfo},
    };
    use crate::{
        orchestration::{Orchestrator, ProjectAction},
        track::{track_widget, TrackWidgetAction},
        traits::Orchestrates,
    };
    use eframe::{egui::Widget, epaint::Galley};
    use ensnare_core::{
        piano_roll::PianoRoll,
        traits::Controls,
        types::TrackTitle,
        uid::{TrackUid, Uid},
    };
    use ensnare_cores_egui::{
        piano_roll::piano_roll,
        widgets::timeline::{self, TimelineIconStripAction},
    };
    use ensnare_egui_widgets::ViewRange;
    use std::sync::Arc;

    /// Wraps an [OldOrchestratorWidget] as a [Widget](eframe::egui::Widget).
    pub fn old_orchestrator<'a>(
        orchestrator: &'a mut OldOrchestrator,
        view_range: &'a mut ViewRange,
        is_piano_roll_visible: &'a mut bool,
        action: &'a mut Option<ProjectAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| {
            OldOrchestratorWidget::new(orchestrator, view_range, is_piano_roll_visible, action)
                .ui(ui)
        }
    }

    /// An egui component that draws an [Orchestrator].
    #[derive(Debug)]
    struct OldOrchestratorWidget<'a> {
        orchestrator: &'a mut OldOrchestrator,
        view_range: &'a mut ViewRange,
        is_piano_roll_visible: &'a mut bool,
        action: &'a mut Option<ProjectAction>,
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
                                    let cursor =
                                        if self.orchestrator.inner.transport.is_performing() {
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
                                            Some(ProjectAction::DoubleClickTrack(*track_uid));
                                    } else if response.clicked() {
                                        *self.action = Some(ProjectAction::ClickTrack(*track_uid));
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
            action: &'a mut Option<ProjectAction>,
        ) -> Self {
            Self {
                orchestrator,
                view_range,
                is_piano_roll_visible,
                action,
            }
        }
    }
}
