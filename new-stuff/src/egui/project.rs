// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::{
    legend::legend,
    signal_chain::SignalChainItem,
    track::{make_title_bar_galley, title_bar, track_widget, TrackWidgetAction, TrackWidgetInfo},
};
use crate::project::Project;
use eframe::{egui::Widget, epaint::Galley};
use ensnare_core::uid::{TrackUid, Uid};
use ensnare_entity::factory::EntityKey;
use std::sync::Arc;
use strum_macros::Display;

/// Actions that widgets might need the parent to perform.
#[derive(Clone, Debug, Display)]
pub enum ProjectAction {
    /// A [Track] was clicked in the UI.
    ClickTrack(TrackUid),
    /// A [Track] was double-clicked in the UI.
    DoubleClickTrack(TrackUid),
    /// A [Track] wants a new device of type [Key].
    NewDeviceForTrack(TrackUid, EntityKey),
    // The user selected an entity with the given uid and name. The UI should
    // show that entity's detail view.
    EntitySelected(Uid, String),
}

/// Wraps a [ProjectWidget] as a [Widget](eframe::egui::Widget).
pub fn project_widget<'a>(
    project: &'a mut Project,
    action: &'a mut Option<ProjectAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| ProjectWidget::new(project, action).ui(ui)
}

/// An egui component that draws the main view of a project.
#[derive(Debug)]
struct ProjectWidget<'a> {
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
                ui.add_enabled(false, title_bar(None));
                ui.add(legend(&mut self.project.view_state.view_range));
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

                    // TODO: this feels cacheable.
                    let signal_items: Vec<SignalChainItem> = {
                        if let Some(entity_uids) = self
                            .project
                            .orchestrator
                            .entity_repo
                            .uids_for_track
                            .get(&track_uid)
                        {
                            entity_uids.iter().fold(Vec::default(), |mut v, uid| {
                                if let Some(entity) =
                                    self.project.orchestrator.entity_repo.entity(*uid)
                                {
                                    v.push((*uid, entity.name().to_string(), true));
                                }

                                v
                            })
                        } else {
                            Vec::default()
                        }
                    };

                    let mut action = None;
                    let track_info = TrackWidgetInfo {
                        track_uid,
                        signal_items: &signal_items,
                        title_font_galley: font_galley,
                        color_scheme,
                    };
                    ui.add(track_widget(
                        &track_info,
                        &mut self.project.composer,
                        &mut self.project.view_state,
                        &mut action,
                    ));
                    if let Some(action) = action {
                        match action {
                            TrackWidgetAction::EntitySelected(uid, name) => {
                                *self.action = Some(ProjectAction::EntitySelected(uid, name));
                            }
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
}
