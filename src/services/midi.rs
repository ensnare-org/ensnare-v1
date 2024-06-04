// Copyright (c) 2023 Mike Tsao. All rights reserved.

// use ensnare_services::{
//     MidiInterfaceService, MidiInterfaceServiceEvent, MidiInterfaceServiceInput,
// };
use crate::midi::MidiPortDescriptor;
use crate::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use derivative::Derivative;
use ensnare::{traits::ProvidesService, types::CrossbeamChannel};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex, RwLock},
    time::Instant,
};

#[derive(Serialize, Deserialize)]
#[serde(remote = "MidiPortDescriptor", rename_all = "kebab-case")]
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


/// The app sends [MidiServiceInput] messages to control the service.
#[derive(Debug)]
pub enum MidiServiceInput {
    Quit,
}

/// [MidiServiceEvent] messages tell the app what happens with the service.
#[derive(Debug)]
pub enum MidiServiceEvent {
    /// A MIDI message arrived from the interface from an external device.
    Midi(MidiChannel, MidiMessage),

    /// A MIDI message was just sent to the interface.
    MidiOut,

    /// Available input ports have been refreshed.
    InputPortsRefreshed(Vec<MidiPortDescriptor>),

    /// Available output ports have been refreshed.
    OutputPortsRefreshed(Vec<MidiPortDescriptor>),
}

/// [MidiService] manages external MIDI hardware interfaces.
#[derive(Debug)]
pub struct MidiService {
    // TEMP pub until MidiInterfaceService is refactored into this struct
    pub inputs: CrossbeamChannel<MidiServiceInput>,
    events: CrossbeamChannel<MidiServiceEvent>,

    midi_interface_service: MidiInterfaceService,

    input_port_descriptors: Arc<Mutex<Vec<MidiPortDescriptor>>>,
    output_port_descriptors: Arc<Mutex<Vec<MidiPortDescriptor>>>,

    settings: Arc<RwLock<MidiSettings>>,
}
impl ProvidesService<MidiServiceInput, MidiServiceEvent> for MidiService {
    fn sender(&self) -> &Sender<MidiServiceInput> {
        &self.inputs.sender
    }

    fn receiver(&self) -> &Receiver<MidiServiceEvent> {
        &self.events.receiver
    }
}
impl MidiService {
    /// Creates a new [MidiService].
    pub fn new_with(settings: &Arc<RwLock<MidiSettings>>) -> Self {
        let midi_interface_service = MidiInterfaceService::default();
        let mut r = Self {
            inputs: Default::default(),
            events: Default::default(),

            midi_interface_service,

            input_port_descriptors: Default::default(),
            output_port_descriptors: Default::default(),

            settings: Arc::clone(settings),
        };
        r.spawn_thread();
        r.conform_selections_to_settings();
        r
    }

    /// Sends a [MidiInterfaceServiceInput] message to the service.
    pub fn send(&mut self, input: MidiInterfaceServiceInput) {
        if let MidiInterfaceServiceInput::Midi(..) = input {
            if let Ok(mut settings) = self.settings.write() {
                settings.e.last_output_instant = Instant::now();
            }
        }

        let _ = self.midi_interface_service.sender().send(input);
    }

    pub fn interface_sender(&self) -> &Sender<MidiInterfaceServiceInput> {
        &self.midi_interface_service.sender()
    }

    // Sits in a loop, watching the receiving side of the event channel and
    // handling whatever comes through.
    fn spawn_thread(&self) {
        let inputs = Arc::clone(&self.input_port_descriptors);
        let outputs = Arc::clone(&self.output_port_descriptors);
        let settings = Arc::clone(&self.settings);
        let app_sender = self.events.sender.clone();
        let receiver = self.midi_interface_service.receiver().clone();
        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    MidiInterfaceServiceEvent::Ready => {}
                    MidiInterfaceServiceEvent::InputPorts(ports) => {
                        if let Ok(mut inputs) = inputs.lock() {
                            *inputs = ports.clone();
                            let _ = app_sender.send(MidiServiceEvent::InputPortsRefreshed(ports));
                        }
                    }
                    MidiInterfaceServiceEvent::InputPortSelected(port) => {
                        if let Ok(mut settings) = settings.write() {
                            settings.set_input(port);
                        }
                    }
                    MidiInterfaceServiceEvent::OutputPorts(ports) => {
                        if let Ok(mut outputs) = outputs.lock() {
                            *outputs = ports.clone();
                            let _ = app_sender.send(MidiServiceEvent::OutputPortsRefreshed(ports));
                        }
                    }
                    MidiInterfaceServiceEvent::OutputPortSelected(port) => {
                        if let Ok(mut settings) = settings.write() {
                            settings.set_output(port);
                        }
                    }
                    MidiInterfaceServiceEvent::Midi(channel, message) => {
                        if let Ok(mut settings) = settings.write() {
                            settings.e.last_input_instant =
                                MidiSettingsEphemerals::create_last_input_instant();
                        }
                        let _ = app_sender.send(MidiServiceEvent::Midi(channel, message));
                    }
                    MidiInterfaceServiceEvent::MidiOut => {
                        let _ = app_sender.send(MidiServiceEvent::MidiOut);
                    }
                    MidiInterfaceServiceEvent::Quit => return,
                }
            }
            eprintln!("MidiService exit");
        });
    }

    /// Returns a reference to the cached list of inputs.
    pub fn inputs(&self) -> &Mutex<Vec<MidiPortDescriptor>> {
        self.input_port_descriptors.as_ref()
    }

    /// Returns a reference to the cached list of outputs.
    pub fn outputs(&self) -> &Mutex<Vec<MidiPortDescriptor>> {
        self.output_port_descriptors.as_ref()
    }

    /// Handles a change in selected MIDI input.
    pub fn select_input(&mut self, port: &MidiPortDescriptor) {
        let _ = self
            .midi_interface_service
            .sender()
            .send(MidiInterfaceServiceInput::SelectMidiInput(port.clone()));
    }

    /// Handles a change in selected MIDI output.
    pub fn select_output(&mut self, port: &MidiPortDescriptor) {
        let _ = self
            .midi_interface_service
            .sender()
            .send(MidiInterfaceServiceInput::SelectMidiOutput(port.clone()));
    }

    /// Cleans up the MIDI service for quitting.
    pub fn exit(&self) {
        // TODO: Create the MidiPanelInput channel, add it to the receiver loop, etc.
        eprintln!("MIDI Panel acks the quit... TODO");
    }

    /// Allows sending directly to the [MidiInterfaceServiceInput] channel.
    pub fn sender(&self) -> &Sender<MidiInterfaceServiceInput> {
        self.midi_interface_service.sender()
    }

    /// Allows sending to the [MidiPanelEvent] channel.
    pub fn app_sender(&self) -> &Sender<MidiServiceEvent> {
        &self.events.sender
    }

    /// When settings are loaded, we have to look at them and update the actual
    /// state to match.
    fn conform_selections_to_settings(&mut self) {
        let (input, output) = if let Ok(settings) = self.settings.read() {
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
