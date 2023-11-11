// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::traits::Orchestrates;
use eframe::{egui::Widget, epaint::Galley};
use ensnare_core::{time::ViewRange, types::TrackTitle};
use ensnare_cores_egui::widgets::timeline::{self};
use std::sync::Arc;

use super::{
    new_track_widget,
    track::{make_title_bar_galley, title_bar},
};

/// Wraps an [OrchestratesTraitWidget] as a [Widget](eframe::egui::Widget).
pub fn orchestrates_trait_widget<'a>(
    orchestrates: &'a mut impl Orchestrates,
    view_range: &'a mut ViewRange,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| OrchestratesTraitWidget::new(orchestrates, view_range).ui(ui)
}

/// An egui component that draws anything implementing [Orchestrates].
#[derive(Debug)]
struct OrchestratesTraitWidget<'a> {
    orchestrates: &'a mut dyn Orchestrates,
    view_range: &'a mut ViewRange,
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

        ui.label("placeholder.........")
    }
}
impl<'a> OrchestratesTraitWidget<'a> {
    fn new(orchestrates: &'a mut impl Orchestrates, view_range: &'a mut ViewRange) -> Self {
        Self {
            orchestrates,
            view_range,
        }
    }
}
