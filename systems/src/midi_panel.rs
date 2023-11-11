// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{CollapsingHeader, ComboBox, Widget};
use ensnare_core::{
    midi_interface::{
        MidiInterfaceEvent, MidiInterfaceInput, MidiInterfaceService, MidiPortDescriptor,
    },
    prelude::*,
};
use ensnare_entity::traits::Displays;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Serialize, Deserialize)]
#[serde(remote = "MidiPortDescriptor")]
struct MidiPortDescriptorDef {
    pub index: usize,
    pub name: String,
}

// https://github.com/serde-rs/serde/issues/1301#issuecomment-394108486
mod opt_external_struct {
    use super::{MidiPortDescriptor, MidiPortDescriptorDef};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(
        value: &Option<MidiPortDescriptor>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "MidiPortDescriptorDef")] &'a MidiPortDescriptor);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<MidiPortDescriptor>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "MidiPortDescriptorDef")] MidiPortDescriptor);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

/// Contains persistent MIDI settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct MidiSettings {
    #[serde(default, with = "opt_external_struct")]
    selected_input: Option<MidiPortDescriptor>,
    #[serde(default, with = "opt_external_struct")]
    selected_output: Option<MidiPortDescriptor>,

    #[serde(skip)]
    has_been_saved: bool,

    #[serde(skip, default = "MidiSettings::create_last_input_instant")]
    last_input_instant: Arc<Mutex<Instant>>,
    #[serde(skip, default = "Instant::now")]
    last_output_instant: Instant,
}
impl Default for MidiSettings {
    fn default() -> Self {
        Self {
            selected_input: Default::default(),
            selected_output: Default::default(),
            has_been_saved: Default::default(),
            last_input_instant: Self::create_last_input_instant(),
            last_output_instant: Instant::now(),
        }
    }
}
impl HasSettings for MidiSettings {
    fn has_been_saved(&self) -> bool {
        self.has_been_saved
    }

    fn needs_save(&mut self) {
        self.has_been_saved = false;
    }

    fn mark_clean(&mut self) {
        self.has_been_saved = true;
    }
}
impl MidiSettings {
    /// Updates the field and marks the struct eligible to save.
    pub fn set_input(&mut self, input: Option<MidiPortDescriptor>) {
        if input != self.selected_input {
            self.selected_input = input;
            self.needs_save();
        }
    }
    /// Updates the field and marks the struct eligible to save.
    pub fn set_output(&mut self, output: Option<MidiPortDescriptor>) {
        if output != self.selected_output {
            self.selected_output = output;
            self.needs_save();
        }
    }

    fn create_last_input_instant() -> Arc<Mutex<Instant>> {
        Arc::new(Mutex::new(Instant::now()))
    }
}

/// The panel provides updates to the app through [MidiPanelEvent] messages.
#[derive(Clone, Debug)]
pub enum MidiPanelEvent {
    /// A MIDI message arrived from the interface.
    Midi(MidiChannel, MidiMessage),

    /// A MIDI message was just sent to the interface.
    MidiOut,

    /// The user has picked a MIDI input. Switch to it.
    ///
    /// Inputs are sent by the PC to the interface.
    SelectInput(MidiPortDescriptor),

    /// The user has picked a MIDI output. Switch to it.
    ///
    /// Outputs are sent by the interfaace to the PC.
    SelectOutput(MidiPortDescriptor),

    /// The requested port refresh is complete.
    PortsRefreshed,
}

/// [MidiPanel] manages external MIDI hardware interfaces.
#[derive(Debug)]
pub struct MidiPanel {
    sender: Sender<MidiInterfaceInput>, // for us to send to the interface
    app_channel: ChannelPair<MidiPanelEvent>, // to tell the app what's happened.

    inputs: Arc<Mutex<Vec<MidiPortDescriptor>>>,
    outputs: Arc<Mutex<Vec<MidiPortDescriptor>>>,

    settings: Arc<Mutex<MidiSettings>>,
}
impl MidiPanel {
    /// Creates a new [MidiPanel].
    pub fn new_with(settings: Arc<Mutex<MidiSettings>>) -> Self {
        let midi_interface_service = MidiInterfaceService::default();
        let sender = midi_interface_service.sender().clone();

        let mut r = Self {
            sender,
            app_channel: Default::default(),

            inputs: Default::default(),
            outputs: Default::default(),

            settings,
        };
        r.start_midi_interface(midi_interface_service.receiver().clone());
        r.conform_selections_to_settings();
        r
    }

    /// Sends a [MidiInterfaceInput] message to the service.
    pub fn send(&mut self, input: MidiInterfaceInput) {
        if let MidiInterfaceInput::Midi(..) = input {
            if let Ok(mut settings) = self.settings.lock() {
                settings.last_output_instant = Instant::now();
            }
        }

        let _ = self.sender.send(input);
    }

    // Sits in a loop, watching the receiving side of the event channel and
    // handling whatever comes through.
    fn start_midi_interface(&self, receiver: Receiver<MidiInterfaceEvent>) {
        let inputs = Arc::clone(&self.inputs);
        let outputs = Arc::clone(&self.outputs);
        let settings = Arc::clone(&self.settings);
        let app_sender = self.app_channel.sender.clone();
        std::thread::spawn(move || {
            let mut inputs_refreshed = false;
            let mut outputs_refreshed = false;
            let mut refresh_sent = false;
            loop {
                if let Ok(event) = receiver.recv() {
                    match event {
                        MidiInterfaceEvent::Ready => {}
                        MidiInterfaceEvent::InputPorts(ports) => {
                            if let Ok(mut inputs) = inputs.lock() {
                                *inputs = ports.clone();
                                inputs_refreshed = true;
                            }
                        }
                        MidiInterfaceEvent::InputPortSelected(port) => {
                            if let Ok(mut settings) = settings.lock() {
                                settings.set_input(port);
                            }
                        }
                        MidiInterfaceEvent::OutputPorts(ports) => {
                            if let Ok(mut outputs) = outputs.lock() {
                                *outputs = ports.clone();
                                outputs_refreshed = true;
                            }
                        }
                        MidiInterfaceEvent::OutputPortSelected(port) => {
                            if let Ok(mut settings) = settings.lock() {
                                settings.set_output(port);
                            }
                        }
                        MidiInterfaceEvent::Midi(channel, message) => {
                            if let Ok(mut settings) = settings.lock() {
                                settings.last_input_instant =
                                    MidiSettings::create_last_input_instant();
                            }
                            let _ = app_sender.send(MidiPanelEvent::Midi(channel, message));
                        }
                        MidiInterfaceEvent::MidiOut => {
                            let _ = app_sender.send(MidiPanelEvent::MidiOut);
                        }
                        MidiInterfaceEvent::Quit => break,
                    }
                } else {
                    eprintln!("unexpected failure of MidiInterfaceEvent channel");
                    break;
                }
                if !refresh_sent && inputs_refreshed && outputs_refreshed {
                    refresh_sent = true;
                    let _ = app_sender.send(MidiPanelEvent::PortsRefreshed);
                }
            }
        });
    }

    /// Returns a reference to the cached list of inputs.
    pub fn inputs(&self) -> &Mutex<Vec<MidiPortDescriptor>> {
        self.inputs.as_ref()
    }

    /// Returns a reference to the cached list of outputs.
    pub fn outputs(&self) -> &Mutex<Vec<MidiPortDescriptor>> {
        self.outputs.as_ref()
    }

    /// Handles a change in selected MIDI input.
    pub fn select_input(&mut self, port: &MidiPortDescriptor) {
        let _ = self
            .sender
            .send(MidiInterfaceInput::SelectMidiInput(port.clone()));
        let _ = self
            .app_channel
            .sender
            .send(MidiPanelEvent::SelectInput(port.clone()));
    }

    /// Handles a change in selected MIDI output.
    pub fn select_output(&mut self, port: &MidiPortDescriptor) {
        let _ = self
            .sender
            .send(MidiInterfaceInput::SelectMidiOutput(port.clone()));
        let _ = self
            .app_channel
            .sender
            .send(MidiPanelEvent::SelectOutput(port.clone()));
    }

    /// The receive side of the [MidiPanelEvent] channel
    pub fn receiver(&self) -> &Receiver<MidiPanelEvent> {
        &self.app_channel.receiver
    }

    /// Cleans up the MIDI service for quitting.
    pub fn exit(&self) {
        // TODO: Create the MidiPanelInput channel, add it to the receiver loop, etc.
        eprintln!("MIDI Panel acks the quit... TODO");
    }

    /// Allows sending to the [MidiInterfaceInput] channel.
    pub fn sender(&self) -> &Sender<MidiInterfaceInput> {
        &self.sender
    }

    /// Allows sending to the [MidiPanelEvent] channel.
    pub fn app_sender(&self) -> &Sender<MidiPanelEvent> {
        &self.app_channel.sender
    }

    /// When settings are loaded, we have to look at them and update the actual
    /// state to match.
    fn conform_selections_to_settings(&mut self) {
        let (input, output) = if let Ok(settings) = self.settings.lock() {
            (
                settings.selected_input.clone(),
                settings.selected_output.clone(),
            )
        } else {
            (None, None)
        };
        if let Some(port) = input {
            self.select_input(&port);
        }
        if let Some(port) = output {
            self.select_output(&port);
        }
    }
}
impl Displays for MidiPanel {}

/// Wraps a [MidiSettingsWidget] as a [Widget](eframe::egui::Widget). Mutates the given view_range.
pub fn midi_settings<'a>(
    settings: &'a mut MidiSettings,
    inputs: &'a [MidiPortDescriptor],
    outputs: &'a [MidiPortDescriptor],
    new_input: &'a mut Option<MidiPortDescriptor>,
    new_output: &'a mut Option<MidiPortDescriptor>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        MidiSettingsWidget::new_with(settings, inputs, outputs, new_input, new_output).ui(ui)
    }
}

#[derive(Debug)]
struct MidiSettingsWidget<'a> {
    settings: &'a mut MidiSettings,
    inputs: &'a [MidiPortDescriptor],
    outputs: &'a [MidiPortDescriptor],
    new_input: &'a mut Option<MidiPortDescriptor>,
    new_output: &'a mut Option<MidiPortDescriptor>,
}
impl<'a> MidiSettingsWidget<'a> {
    pub fn new_with(
        settings: &'a mut MidiSettings,
        inputs: &'a [MidiPortDescriptor],
        outputs: &'a [MidiPortDescriptor],
        new_input: &'a mut Option<MidiPortDescriptor>,
        new_output: &'a mut Option<MidiPortDescriptor>,
    ) -> Self {
        Self {
            settings,
            inputs,
            outputs,
            new_input,
            new_output,
        }
    }
}
impl<'a> eframe::egui::Widget for MidiSettingsWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        CollapsingHeader::new("MIDI")
            .default_open(true)
            .show(ui, |ui| {
                let mut cb = ComboBox::from_label("MIDI in");
                let (mut selected_index, _selected_text) =
                    if let Some(selected) = &self.settings.selected_input {
                        cb = cb.selected_text(selected.name.clone());
                        (selected.index, selected.name.as_str())
                    } else {
                        (usize::MAX, "None")
                    };
                cb.show_ui(ui, |ui| {
                    for port in self.inputs.iter() {
                        if ui
                            .selectable_value(&mut selected_index, port.index, port.name.clone())
                            .changed()
                        {
                            self.settings.set_input(Some(port.clone()));
                            *self.new_input = Some(port.clone());
                        }
                    }
                });
                ui.end_row();

                let mut cb = ComboBox::from_label("MIDI out");
                let (mut selected_index, _selected_text) =
                    if let Some(selected) = &self.settings.selected_output {
                        cb = cb.selected_text(selected.name.clone());
                        (selected.index, selected.name.as_str())
                    } else {
                        (usize::MAX, "None")
                    };
                cb.show_ui(ui, |ui| {
                    for port in self.outputs.iter() {
                        if ui
                            .selectable_value(&mut selected_index, port.index, port.name.clone())
                            .changed()
                        {
                            self.settings.set_output(Some(port.clone()));
                            *self.new_output = Some(port.clone());
                        }
                    }
                });
                ui.end_row();
            })
            .header_response
    }
}
