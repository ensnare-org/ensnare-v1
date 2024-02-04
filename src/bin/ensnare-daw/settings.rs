// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `settings` module contains [Settings], which are all the user's
//! persistent global preferences. It also contains [SettingsPanel].

use crossbeam_channel::{Receiver, Sender};
use eframe::egui::Frame;
use ensnare::{
    midi::interface::{MidiInterfaceServiceInput, MidiPortDescriptor},
    services::{AudioSettings, MidiSettings},
    traits::{Displays, HasSettings},
    ui::widgets::{AudioSettingsWidget, MidiSettingsWidget},
};
use ensnare_core::types::ChannelPair;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

#[derive(Debug)]
pub enum SettingsEvent {
    ShouldRouteExternally(bool),
}

/// Global preferences.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Settings {
    pub(crate) audio_settings: AudioSettings,
    pub(crate) midi_settings: Arc<std::sync::RwLock<MidiSettings>>,

    #[serde(skip)]
    pub(crate) e: SettingsEphemerals,
}
#[derive(Debug, Default)]
pub(crate) struct SettingsEphemerals {
    event_channel: ChannelPair<SettingsEvent>,
    midi_sender: Option<Sender<MidiInterfaceServiceInput>>,

    // Cached options for fast menu drawing.
    midi_inputs: Vec<MidiPortDescriptor>,
    midi_outputs: Vec<MidiPortDescriptor>,
}
impl Settings {
    const FILENAME: &'static str = "settings.json";

    pub(crate) fn load() -> anyhow::Result<Self> {
        let settings_path = PathBuf::from(Self::FILENAME);
        let mut contents = String::new();

        // https://utcc.utoronto.ca/~cks/space/blog/sysadmin/ReportConfigFileLocations
        match std::env::current_dir() {
            Ok(cwd) => eprintln!(
                "Loading preferences from {settings_path:?}, current working directory {cwd:?}..."
            ),
            Err(e) => eprintln!("Couldn't get current working directory: {e:?}"),
        }

        let mut file = File::open(settings_path.clone())
            .map_err(|e| anyhow::format_err!("Couldn't open {settings_path:?}: {}", e))?;
        file.read_to_string(&mut contents)
            .map_err(|e| anyhow::format_err!("Couldn't read {settings_path:?}: {}", e))?;
        let settings: Self = serde_json::from_str(&contents)
            .map_err(|e| anyhow::format_err!("Couldn't parse {settings_path:?}: {}", e))?;

        let should = settings
            .midi_settings
            .read()
            .unwrap()
            .should_route_externally();
        settings.notify_should_route_externally(should);

        Ok(settings)
    }

    pub(crate) fn save(&mut self) -> anyhow::Result<()> {
        let settings_path = PathBuf::from(Self::FILENAME);
        let json = serde_json::to_string_pretty(&self)
            .map_err(|_| anyhow::format_err!("Unable to serialize settings JSON"))?;
        if let Some(dir) = settings_path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| {
                anyhow::format_err!(
                    "Unable to create {settings_path:?} parent directories: {}",
                    e
                )
            })?;
        }

        let mut file = File::create(settings_path.clone())
            .map_err(|e| anyhow::format_err!("Unable to create {settings_path:?}: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| anyhow::format_err!("Unable to write {settings_path:?}: {}", e))?;

        self.mark_clean();
        Ok(())
    }

    pub(crate) fn handle_midi_input_port_refresh(&mut self, ports: &[MidiPortDescriptor]) {
        self.e.midi_inputs = ports.to_vec();
    }

    pub(crate) fn handle_midi_output_port_refresh(&mut self, ports: &[MidiPortDescriptor]) {
        self.e.midi_outputs = ports.to_vec();
    }

    pub(crate) fn set_midi_sender(&mut self, sender: &Sender<MidiInterfaceServiceInput>) {
        self.e.midi_sender = Some(sender.clone());
    }

    pub(crate) fn receiver(&self) -> &Receiver<SettingsEvent> {
        &self.e.event_channel.receiver
    }

    // We require the parameter to be provided, even though we could look it up
    // ourselves, because our reference to MidiSettings is in a RwLock, and a
    // common case is to call this while the caller has a write() lock on
    // MidiSettings() -- deadlock!
    fn notify_should_route_externally(&self, should: bool) {
        let _ = self
            .e
            .event_channel
            .sender
            .send(SettingsEvent::ShouldRouteExternally(should));
    }
}
impl HasSettings for Settings {
    fn has_been_saved(&self) -> bool {
        let has_midi_been_saved = {
            if let Ok(midi) = self.midi_settings.read() {
                midi.has_been_saved()
            } else {
                true
            }
        };
        self.audio_settings.has_been_saved() || has_midi_been_saved
    }

    fn needs_save(&mut self) {
        panic!("TODO: this struct has no settings of its own, so there shouldn't be a reason to mark it dirty.")
    }

    fn mark_clean(&mut self) {
        self.audio_settings.mark_clean();
        if let Ok(mut midi) = self.midi_settings.write() {
            midi.mark_clean();
        }
    }
}
impl Displays for Settings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut new_input = None;
        let mut new_output = None;
        ui.set_max_width(480.0);
        let response = {
            Frame::default()
                .stroke(ui.ctx().style().visuals.noninteractive().fg_stroke)
                .inner_margin(5.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.heading("Audio");
                    ui.add(AudioSettingsWidget::widget(&mut self.audio_settings))
                })
                .inner
        } | {
            Frame::default()
                .stroke(ui.ctx().style().visuals.noninteractive().fg_stroke)
                .inner_margin(5.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.heading("MIDI");
                    let mut settings = self.midi_settings.write().unwrap();
                    let item_response = ui.add(MidiSettingsWidget::widget(
                        &mut settings,
                        &self.e.midi_inputs,
                        &self.e.midi_outputs,
                        &mut new_input,
                        &mut new_output,
                    ));
                    if item_response.changed() {
                        self.notify_should_route_externally(settings.should_route_externally());
                    }
                    item_response
                })
                .inner
        };

        if let Some(sender) = &self.e.midi_sender {
            if let Some(new_input) = &new_input {
                let _ = sender.send(MidiInterfaceServiceInput::SelectMidiInput(
                    new_input.clone(),
                ));
            }
            if let Some(new_output) = &new_output {
                let _ = sender.send(MidiInterfaceServiceInput::SelectMidiOutput(
                    new_output.clone(),
                ));
            }
        }

        #[cfg(debug_assertions)]
        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "🐛 Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }
        response
    }
}
