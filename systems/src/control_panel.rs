// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::{
    egui::{ImageButton, Layout},
    epaint::vec2,
};
use ensnare_core::prelude::*;
use ensnare_cores_egui::widgets::audio::{frequency_domain, time_domain, CircularSampleBuffer};
use ensnare_egui_widgets::activity_indicator;
use ensnare_entity::traits::{Acts, Displays, IsAction};
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
impl IsAction for ControlPanelAction {}

#[derive(Debug, Default)]
enum ControlPanelDisplayMode {
    #[default]
    Time,
    Frequency,
}

/// [ControlPanel] is the UI component at the top of the main window. Transport,
/// MIDI status, etc.
#[derive(Debug, Default)]
pub struct ControlPanel {
    action: Option<ControlPanelAction>,
    saw_midi_in_activity: bool,
    saw_midi_out_activity: bool,
    sample_buffer: CircularSampleBuffer,
    pub sample_channel: ChannelPair<[Sample; 64]>,
    which_display: ControlPanelDisplayMode,
    fft_buffer: Vec<f32>,
}
impl ControlPanel {
    /// Tell [ControlPanel] that the system just saw an incoming MIDI message.
    pub fn tickle_midi_in(&mut self) {
        self.saw_midi_in_activity = true;
    }

    /// Tell [ControlPanel] that the system just produced an outgoing MIDI message.
    pub fn tickle_midi_out(&mut self) {
        self.saw_midi_out_activity = true;
    }
}
impl Displays for ControlPanel {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.horizontal_centered(|ui| {
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../res/images/md-symbols/play_arrow.png"
                )))
                .on_hover_text("Start playback")
                .clicked()
            {
                self.action = Some(ControlPanelAction::Play);
            }
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../res/images/md-symbols/stop.png"
                )))
                .on_hover_text("Stop playback")
                .clicked()
            {
                self.action = Some(ControlPanelAction::Stop);
            }
            ui.separator();
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../res/images/md-symbols/new_window.png"
                )))
                .on_hover_text("New project")
                .clicked()
            {
                self.action = Some(ControlPanelAction::New);
            }
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../res/images/md-symbols/file_open.png"
                )))
                .on_hover_text("Open project")
                .clicked()
            {
                self.action = Some(ControlPanelAction::Open(PathBuf::from(
                    "my-ensnare-project.json",
                )));
            }
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../res/images/md-symbols/file_save.png"
                )))
                .on_hover_text("Save project")
                .clicked()
            {
                self.action = Some(ControlPanelAction::Save(PathBuf::from(
                    "my-ensnare-project.json",
                )));
            }
            ui.separator();
            ui.allocate_ui_with_layout(
                vec2(4.0, 8.0),
                Layout::top_down(eframe::emath::Align::Center),
                |ui| {
                    ui.add(activity_indicator(self.saw_midi_in_activity));
                    ui.add(activity_indicator(self.saw_midi_out_activity));
                    self.saw_midi_in_activity = false;
                    self.saw_midi_out_activity = false;
                },
            );

            // TODO: not on the UI thread!
            while let Ok(samples) = self.sample_channel.receiver.try_recv() {
                self.sample_buffer.push(&samples);
            }

            let (samples, start) = self.sample_buffer.get();
            ui.scope(|ui| {
                ui.set_max_size(vec2(64.0, 32.0));
                if match self.which_display {
                    ControlPanelDisplayMode::Time => ui.add(time_domain(samples, start)),
                    ControlPanelDisplayMode::Frequency => {
                        self.fft_buffer = self.sample_buffer.analyze_spectrum().unwrap();
                        ui.add(frequency_domain(&self.fft_buffer))
                    }
                }
                .clicked()
                {
                    self.which_display = match self.which_display {
                        ControlPanelDisplayMode::Time => ControlPanelDisplayMode::Frequency,
                        ControlPanelDisplayMode::Frequency => ControlPanelDisplayMode::Time,
                    }
                }
            });
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
