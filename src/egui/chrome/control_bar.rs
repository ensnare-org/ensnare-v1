// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::egui::{activity_indicator, analyze_spectrum, FrequencyDomainWidget, TimeDomainWidget};
use crate::types::VisualizationQueue;
use eframe::{
    egui::{Image, ImageButton, Layout, Widget},
    epaint::vec2,
};
use std::path::PathBuf;
use strum_macros::Display;

#[derive(Debug, Default)]
pub enum ControlBarDisplayMode {
    #[default]
    Time,
    Frequency,
}

/// [ControlBar] is the UI component at the top of the main window to the right
/// of Transport.
#[derive(Debug, Default)]
pub struct ControlBar {
    pub saw_midi_in_activity: bool,
    pub saw_midi_out_activity: bool,

    /// An owned VecDeque that acts as a ring buffer of the most recent
    /// generated audio frames.
    pub visualization_queue: VisualizationQueue,
    pub display_mode: ControlBarDisplayMode,
    pub fft_buffer: Vec<f32>,
}
impl ControlBar {
    /// Tell [ControlBar] that the system just saw an incoming MIDI message.
    pub fn tickle_midi_in(&mut self) {
        self.saw_midi_in_activity = true;
    }

    /// Tell [ControlPanel] that the system just produced an outgoing MIDI message.
    pub fn tickle_midi_out(&mut self) {
        self.saw_midi_out_activity = true;
    }
}

/// Actions the user might take via the control panel.
#[derive(Debug, Display)]
pub enum ControlBarAction {
    /// Play button pressed.
    Play,

    /// Stop button pressed.
    Stop,

    /// The user asked to create a new project.
    New,

    /// The user asked to load a project.
    Open,

    /// The user asked to save the current project.
    Save,

    /// The user asked to export the current project as a WAV.
    ExportToWav,

    /// The user pressed the settings icon.
    ToggleSettings,
}

#[derive(Debug)]
pub struct ControlBarWidget<'a> {
    control_bar: &'a mut ControlBar,
    action: &'a mut Option<ControlBarAction>,
}
impl<'a> ControlBarWidget<'a> {
    fn new_with(control_bar: &'a mut ControlBar, action: &'a mut Option<ControlBarAction>) -> Self {
        Self {
            control_bar,
            action,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        control_bar: &'a mut ControlBar,
        action: &'a mut Option<ControlBarAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| ControlBarWidget::new_with(control_bar, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for ControlBarWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.horizontal_centered(|ui| {
            if ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/play_arrow.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Start playback")
                .clicked()
            {
                *self.action = Some(ControlBarAction::Play);
            }
            if ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/stop.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Stop playback")
                .clicked()
            {
                *self.action = Some(ControlBarAction::Stop);
            }
            ui.separator();
            if ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/new_window.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("New project")
                .clicked()
            {
                *self.action = Some(ControlBarAction::New);
            }
            if ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/file_open.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Open project")
                .clicked()
            {
                *self.action = Some(ControlBarAction::Open);
            }
            if ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/file_save.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Save project")
                .clicked()
            {
                *self.action = Some(ControlBarAction::Save);
            }
            if ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/audio_file.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Export project to WAV")
                .clicked()
            {
                *self.action = Some(ControlBarAction::ExportToWav);
            }
            ui.separator();
            ui.allocate_ui_with_layout(
                vec2(4.0, 8.0),
                Layout::top_down(eframe::emath::Align::Center),
                |ui| {
                    ui.add(activity_indicator(self.control_bar.saw_midi_in_activity));
                    ui.add(activity_indicator(self.control_bar.saw_midi_out_activity));
                    self.control_bar.saw_midi_in_activity = false;
                    self.control_bar.saw_midi_out_activity = false;
                },
            );

            if let Ok(queue) = self.control_bar.visualization_queue.0.read() {
                let (sample_buffer_slice_1, sample_buffer_slice_2) = queue.as_slices();
                ui.scope(|ui| {
                    ui.set_max_size(vec2(64.0, 32.0));
                    if match self.control_bar.display_mode {
                        ControlBarDisplayMode::Time => ui.add(TimeDomainWidget::widget(
                            sample_buffer_slice_1,
                            sample_buffer_slice_2,
                        )),
                        ControlBarDisplayMode::Frequency => {
                            let values =
                                analyze_spectrum(sample_buffer_slice_1, sample_buffer_slice_2)
                                    .unwrap();
                            ui.add(FrequencyDomainWidget::widget(&values))
                        }
                    }
                    .clicked()
                    {
                        self.control_bar.display_mode = match self.control_bar.display_mode {
                            ControlBarDisplayMode::Time => ControlBarDisplayMode::Frequency,
                            ControlBarDisplayMode::Frequency => ControlBarDisplayMode::Time,
                        }
                    }
                });
            }
            ui.separator();
            if ui
                .add(ImageButton::new(
                    Image::new(eframe::egui::include_image!(
                        "../../../res/images/md-symbols/settings.png"
                    ))
                    .fit_to_original_size(1.0),
                ))
                .on_hover_text("Settings")
                .clicked()
            {
                *self.action = Some(ControlBarAction::ToggleSettings);
            }
        })
        .response
    }
}
