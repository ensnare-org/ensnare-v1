// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `settings` module contains [Settings], which are all the user's
//! persistent global preferences. It also contains [SettingsPanel].

use crossbeam_channel::Sender;
use ensnare::{
    all_entities::EntityWrapper,
    arrangement::{Orchestrator, OrchestratorHelper},
    midi::interface::{MidiInterfaceInput, MidiPortDescriptor},
    services::{AudioService, AudioSettings, MidiService, MidiSettings, NeedsAudioFn},
    traits::{Displays, HasSettings, WorkEvent},
    types::Sample,
    ui::widgets::{audio_settings, midi_settings},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Global preferences.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Settings {
    audio_settings: AudioSettings,
    midi_settings: Arc<std::sync::Mutex<MidiSettings>>,
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
    pub(crate) settings: Settings,
    pub(crate) audio_service: AudioService,
    pub(crate) midi_service: MidiService,

    midi_inputs: Vec<MidiPortDescriptor>,
    midi_outputs: Vec<MidiPortDescriptor>,
}
impl SettingsPanel {
    /// Creates a new [SettingsPanel].
    pub fn new_with(
        settings: Settings,
        orchestrator: &Arc<Mutex<Orchestrator<dyn EntityWrapper>>>,
        sample_buffer_sender: Option<Sender<[Sample; 64]>>,
    ) -> Self {
        let orchestrator = Arc::clone(&orchestrator);
        let midi_service = MidiService::new_with(Arc::clone(&settings.midi_settings));
        let midi_service_sender = midi_service.sender().clone();
        let sample_buffer_sender = sample_buffer_sender.clone();
        let needs_audio_fn: NeedsAudioFn = {
            Box::new(move |audio_queue, samples_requested| {
                if let Ok(mut o) = orchestrator.lock() {
                    let o: &mut Orchestrator<dyn EntityWrapper> = &mut o;
                    let mut helper = OrchestratorHelper::new_with_sample_buffer_sender(
                        o,
                        sample_buffer_sender.clone(),
                    );
                    helper.render_and_enqueue(samples_requested, audio_queue, &mut |event| {
                        if let WorkEvent::Midi(channel, message) = event {
                            let _ = midi_service_sender
                                .send(MidiInterfaceInput::Midi(channel, message));
                        }
                    });
                }
            })
        };
        Self {
            settings,
            audio_service: AudioService::new_with(needs_audio_fn),
            midi_service,
            midi_inputs: Default::default(),
            midi_outputs: Default::default(),
        }
    }

    /// Asks the panel to shut down any services associated with contained panels.
    pub fn exit(&self) {
        self.audio_service.exit();
        self.midi_service.exit();
    }

    pub fn handle_midi_port_refresh(&mut self) {
        self.midi_inputs = self.midi_service.inputs().lock().unwrap().clone();
        self.midi_outputs = self.midi_service.outputs().lock().unwrap().clone();
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
            self.midi_service.select_input(new_input);
        }
        if let Some(new_output) = &new_output {
            self.midi_service.select_output(new_output);
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
