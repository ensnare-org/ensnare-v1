// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::{
    track::{
        make_title_bar_galley, TitleBarWidget, TrackWidget, TrackWidgetAction, TrackWidgetInfo,
    },
    LegendWidget,
};
use crate::{orchestration::TrackViewMode, prelude::*};
use eframe::{egui::Widget, epaint::Galley};
use std::sync::Arc;
use strum_macros::Display;

/// Actions that widgets might need the parent to perform.
#[derive(Clone, Debug, Display)]
pub enum ProjectAction {
    /// A track wants a new device of type [EntityKey].
    NewDeviceForTrack(TrackUid, EntityKey),
    /// The user selected an entity with the given uid and name. The UI should
    /// show that entity's detail view.
    SelectEntity(Uid, String),
    /// The user wants to remove an entity from a track's signal chain.
    RemoveEntity(Uid),
}

/// An egui component that draws the main view of a project.
#[derive(Debug)]
pub struct ProjectWidget<'a> {
    project: &'a mut Project,
    action: &'a mut Option<ProjectAction>,
}
impl<'a> eframe::egui::Widget for ProjectWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // The timeline needs to be aligned with the track content, so
        // we create an empty track title bar to match with the real
        // ones.
        let response = ui
            .horizontal(|ui| {
                let mut action = None;
                ui.add_enabled(false, TitleBarWidget::widget(None, &mut action));
                ui.add(LegendWidget::widget(
                    &mut self.project.view_state.view_range,
                ));
            })
            .response;

        // Create a scrolling area for all the tracks.
        eframe::egui::ScrollArea::vertical()
            .id_source("orchestrator-scroller")
            .show(ui, |ui| {
                let track_uids = self.project.orchestrator.track_uids().to_vec();
                for track_uid in track_uids {
                    let track_title = self.project.track_titles.get(&track_uid);
                    let font_galley: Option<Arc<Galley>> = if let Some(track_title) = track_title {
                        Some(make_title_bar_galley(ui, track_title))
                    } else {
                        None
                    };
                    let color_scheme = self
                        .project
                        .track_color_schemes
                        .get(&track_uid)
                        .cloned()
                        .unwrap_or_default();

                    let mut action = None;
                    let track_info = TrackWidgetInfo {
                        track_uid,
                        title_font_galley: font_galley,
                        color_scheme,
                    };
                    ui.add(TrackWidget::widget(&track_info, self.project, &mut action));
                    if let Some(action) = action {
                        let mut switch_to_composition = false;
                        match action {
                            TrackWidgetAction::SelectEntity(uid, name) => {
                                *self.action = Some(ProjectAction::SelectEntity(uid, name));
                            }
                            TrackWidgetAction::RemoveEntity(uid) => {
                                *self.action = Some(ProjectAction::RemoveEntity(uid));
                            }
                            TrackWidgetAction::Clicked => {
                                let implement_this_bool_please = false;
                                self.project
                                    .view_state
                                    .track_selection_set
                                    .click(&track_uid, implement_this_bool_please);
                            }
                            TrackWidgetAction::NewDevice(key) => {
                                *self.action = Some(ProjectAction::NewDeviceForTrack(
                                    track_uid,
                                    EntityKey::from(key),
                                ));
                            }
                            TrackWidgetAction::AdvanceTimelineView(track_uid) => {
                                self.project.advance_track_view_mode(track_uid);
                            }
                            TrackWidgetAction::CreateAutomationLane(track_uid) => {
                                if let Ok(path_uid) = self.project.add_path(
                                    track_uid,
                                    SignalPathBuilder::default().build().unwrap(),
                                ) {
                                    self.project.set_track_view_mode(
                                        track_uid,
                                        TrackViewMode::Control(path_uid),
                                    );
                                }
                            }
                            TrackWidgetAction::ArrangePattern(pattern_uid, position) => {
                                let _ =
                                    self.project
                                        .arrange_pattern(track_uid, pattern_uid, position);
                                switch_to_composition = true;
                            }
                            TrackWidgetAction::MoveArrangement(
                                arrangement_uid,
                                position,
                                is_shift_pressed,
                            ) => {
                                let _ = self.project.move_arrangement(
                                    track_uid,
                                    arrangement_uid,
                                    position,
                                    is_shift_pressed,
                                );
                                switch_to_composition = true;
                            }
                            TrackWidgetAction::LinkPath(path_uid, uid, param) => {
                                let _ = self.project.link_path(path_uid, uid, param);
                                self.project.regenerate_signal_chain(track_uid);
                            }
                            TrackWidgetAction::UnlinkPath(path_uid, uid, param) => {
                                self.project.unlink_path(path_uid, uid, param);
                                self.project.regenerate_signal_chain(track_uid);
                            }
                        }
                        if switch_to_composition {
                            // Nice touch: if you drag to track and it's
                            // displaying a different mode from
                            // composition, then switch to the one that
                            // shows the result of what you just did.
                            self.project
                                .set_track_view_mode(track_uid, TrackViewMode::Composition);
                        }
                    }
                }
            });

        // Note! This response is from the timeline header and doesn't mean
        // anything.
        response
    }
}
impl<'a> ProjectWidget<'a> {
    fn new(project: &'a mut Project, action: &'a mut Option<ProjectAction>) -> Self {
        Self { project, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        project: &'a mut Project,
        action: &'a mut Option<ProjectAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| ProjectWidget::new(project, action).ui(ui)
    }
}
