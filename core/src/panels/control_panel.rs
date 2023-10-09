// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::traits::{Acts, Displays};
use eframe::egui::{Response, Ui};
use std::path::PathBuf;
use strum_macros::Display;

/// Actions the user might take via the control panel.
#[derive(Debug, Display)]
pub enum ControlPanelAction {
    /// Play button pressed.
    Play,

    /// Stop button pressed.
    Stop,

    /// The user asked to create a new project.
    New,

    /// The user asked to load the project having the given filename.
    Open(PathBuf),

    /// The user asked to save the current project to the given filename.
    Save(PathBuf),

    /// The user pressed the settings icon.
    ToggleSettings,
}

/// [ControlPanel] is the UI component at the top of the main window. Transport,
/// MIDI status, etc.
#[derive(Debug, Default)]
pub struct ControlPanel {
    action: Option<ControlPanelAction>,
}
impl Displays for ControlPanel {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.horizontal_centered(|ui| {
            if ui.button("play").clicked() {
                self.action = Some(ControlPanelAction::Play);
            }
            if ui.button("stop").clicked() {
                self.action = Some(ControlPanelAction::Stop);
            }
            ui.separator();
            if ui.button("new").clicked() {
                self.action = Some(ControlPanelAction::New);
            }
            if ui.button("open").clicked() {
                self.action = Some(ControlPanelAction::Open(PathBuf::from("minidaw.json")));
            }
            if ui.button("save").clicked() {
                self.action = Some(ControlPanelAction::Save(PathBuf::from("minidaw.json")));
            }
            ui.separator();
            if ui.button("settings").clicked() {
                self.action = Some(ControlPanelAction::ToggleSettings);
            }
        })
        .response
    }
}
impl Acts for ControlPanel {
    type Action = ControlPanelAction;

    fn set_action(&mut self, action: Self::Action) {
        debug_assert!(
            self.action.is_none(),
            "Uh-oh, tried to set to {action} but it was already set to {:?}",
            self.action
        );
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<Self::Action> {
        self.action.take()
    }
}
