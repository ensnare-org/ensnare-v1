// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `settings` module contains [Settings], which are all the user's
//! persistent global preferences. It also contains [SettingsPanel].

use ensnare::{
    arrangement::{OldOrchestrator, OrchestratorHelper},
    midi::interface::{MidiInterfaceInput, MidiPortDescriptor},
    systems::{AudioPanel, AudioSettings, MidiPanel, MidiSettings, NeedsAudioFn},
    traits::{EntityEvent, HasSettings},
    ui::widgets::{audio_settings, midi_settings},
};
use ensnare_entity::traits::Displays;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

/// Global preferences.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Settings {
    audio_settings: AudioSettings,
    midi_settings: std::sync::Arc<std::sync::Mutex<MidiSettings>>,
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
        serde_json::from_str(&contents)
            .map_err(|e| anyhow::format_err!("Couldn't parse {settings_path:?}: {}", e))
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
}
impl HasSettings for Settings {
    fn has_been_saved(&self) -> bool {
        let has_midi_been_saved = {
            if let Ok(midi) = self.midi_settings.lock() {
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
        if let Ok(mut midi) = self.midi_settings.lock() {
            midi.mark_clean();
        }
    }
}

#[derive(Debug)]
pub(crate) struct SettingsPanel {
    settings: Settings,
    audio_panel: AudioPanel,
    midi_panel: MidiPanel,

    midi_inputs: Vec<MidiPortDescriptor>,
    midi_outputs: Vec<MidiPortDescriptor>,
}
impl SettingsPanel {
    /// Creates a new [SettingsPanel].
    pub fn new_with(
        settings: Settings,
        orchestrator: std::sync::Arc<std::sync::Mutex<OldOrchestrator>>,
    ) -> Self {
        let midi_panel = MidiPanel::new_with(std::sync::Arc::clone(&settings.midi_settings));
        let midi_panel_sender = midi_panel.sender().clone();
        let needs_audio_fn: NeedsAudioFn = {
            Box::new(move |audio_queue, samples_requested| {
                if let Ok(mut o) = orchestrator.lock() {
                    let mut o: &mut OldOrchestrator = &mut o;
                    let mut helper = OrchestratorHelper::new_with(o);
                    helper.render_and_enqueue(samples_requested, audio_queue, &mut |_, event| {
                        if let EntityEvent::Midi(channel, message) = event {
                            let _ =
                                midi_panel_sender.send(MidiInterfaceInput::Midi(channel, message));
                        }
                    });
                }
            })
        };
        Self {
            settings,
            audio_panel: AudioPanel::new_with(needs_audio_fn),
            midi_panel,
            midi_inputs: Default::default(),
            midi_outputs: Default::default(),
        }
    }

    /// The owned [AudioPanel].
    pub fn audio_panel(&self) -> &AudioPanel {
        &self.audio_panel
    }

    /// The owned [MidiPanel].
    pub fn midi_panel(&self) -> &MidiPanel {
        &self.midi_panel
    }

    /// Asks the panel to shut down any services associated with contained panels.
    pub fn exit(&self) {
        self.audio_panel.exit();
        self.midi_panel.exit();
    }

    pub fn handle_midi_port_refresh(&mut self) {
        self.midi_inputs = self.midi_panel.inputs().lock().unwrap().clone();
        self.midi_outputs = self.midi_panel.outputs().lock().unwrap().clone();
    }

    pub(crate) fn settings(&self) -> &Settings {
        &self.settings
    }

    pub(crate) fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }
}
impl Displays for SettingsPanel {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut new_input = None;
        let mut new_output = None;
        let response = {
            ui.heading("Audio");
            ui.add(audio_settings(&mut self.settings.audio_settings))
        } | {
            ui.heading("MIDI");
            let mut settings = self.settings.midi_settings.lock().unwrap();
            ui.add(midi_settings(
                &mut settings,
                &self.midi_inputs,
                &self.midi_outputs,
                &mut new_input,
                &mut new_output,
            ))
        };

        if let Some(new_input) = &new_input {
            self.midi_panel.select_input(new_input);
        }
        if let Some(new_output) = &new_output {
            self.midi_panel.select_output(new_output);
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
