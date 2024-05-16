// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::settings::SettingsEvent;
use crossbeam_channel::{Receiver, Select, Sender};
use ensnare::{
    midi::MidiInterfaceServiceInput,
    services::{
        AudioService, AudioServiceEvent, AudioServiceInput, MidiService, MidiServiceEvent,
        MidiServiceInput, ProjectService, ProjectServiceEvent, ProjectServiceInput,
        ProvidesService,
    },
    util::ChannelPair,
};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub(super) enum LoadError {
    #[error("see https://crates.io/crates/thiserror to write better error messages")]
    Todo,
}

#[allow(dead_code)]
#[derive(Debug, derive_more::Display)]
pub(super) enum SaveError {
    Todo,
}

#[derive(Debug)]
pub(super) enum MiniDawInput {
    Quit,
}

/// An aggregation of all the service events that the app might want to process.
#[derive(Debug)]
pub(super) enum MiniDawEvent {
    MidiPanelEvent(MidiServiceEvent),
    AudioServiceEvent(AudioServiceEvent),
    ProjectServiceEvent(ProjectServiceEvent),
    Quit,
}

#[derive(Debug)]
pub(super) struct MiniDawEventAggregationService {
    input_channels: ChannelPair<MiniDawInput>,
    event_channels: ChannelPair<MiniDawEvent>,

    // The aggregated services. Avoid speaking directly to them; use the
    // channels instead.
    audio_service: AudioService,
    midi_service: MidiService,
    project_service: ProjectService,

    settings_receiver: Receiver<SettingsEvent>,
}
impl MiniDawEventAggregationService {
    pub fn new_with(
        audio_service: AudioService,
        midi_service: MidiService,
        project_service: ProjectService,
        settings_receiver: &Receiver<SettingsEvent>,
    ) -> Self {
        let r = Self {
            input_channels: Default::default(),
            event_channels: Default::default(),
            audio_service,
            midi_service,
            project_service,
            settings_receiver: settings_receiver.clone(),
        };
        r.spawn_thread();
        r
    }

    /// Watches all the channel receivers we know about, and either handles them
    /// immediately off the UI thread or forwards them to the app's event
    /// channel.
    fn spawn_thread(&self) {
        // Sends aggregated events for the app to handle.
        let app_sender = self.event_channels.sender.clone();

        // Takes commands from the app.
        let app_receiver = self.input_channels.receiver.clone();

        // Each of these pairs communicates with a service.
        let audio_sender = self.audio_service.sender().clone();
        let audio_receiver = self.audio_service.receiver().clone();

        // Note that this one is temporarily different! MidiInterfaceService and
        // MidiService are separate but shouldn't be. It's confusing!
        let midi_sender = self.midi_service.inputs.sender.clone();
        let midi_interface_sender = self.midi_service.interface_sender().clone();

        let midi_receiver = self.midi_service.receiver().clone();
        let project_sender = self.project_service.sender().clone();
        let project_receiver = self.project_service.receiver().clone();
        let settings_receiver = self.settings_receiver.clone();

        let _ = std::thread::spawn(move || {
            let mut sel = Select::new();
            let app_index = sel.recv(&app_receiver);
            let midi_index = sel.recv(&midi_receiver);
            let audio_index = sel.recv(&audio_receiver);
            let project_index = sel.recv(&project_receiver);
            let settings_index = sel.recv(&settings_receiver);
            let mut should_route_midi = true;

            loop {
                let operation = sel.select();
                match operation.index() {
                    index if index == app_index => {
                        if let Ok(input) = operation.recv(&app_receiver) {
                            match input {
                                MiniDawInput::Quit => {
                                    let _ = audio_sender.send(AudioServiceInput::Quit);
                                    let _ = midi_sender.send(MidiServiceInput::Quit);
                                    let _ = project_sender.send(ProjectServiceInput::ServiceQuit);
                                    let _ = app_sender.send(MiniDawEvent::Quit);
                                    return;
                                }
                            }
                        }
                    }
                    index if index == audio_index => {
                        if let Ok(event) = operation.recv(&audio_receiver) {
                            match event {
                                AudioServiceEvent::FramesNeeded(count) => {
                                    let _ = project_sender
                                        .send(ProjectServiceInput::FramesNeeded(count));
                                }
                                AudioServiceEvent::Reset(sample_rate, channel_count) => {
                                    let _ = project_sender.send(ProjectServiceInput::AudioReset(
                                        sample_rate,
                                        channel_count,
                                    ));
                                }
                                AudioServiceEvent::Underrun => {}
                            }
                            let _ = app_sender.send(MiniDawEvent::AudioServiceEvent(event));
                        }
                    }
                    index if index == midi_index => {
                        if let Ok(event) = operation.recv(&midi_receiver) {
                            match event {
                                // MIDI messages that came from external interfaces.
                                MidiServiceEvent::Midi(channel, message) => {
                                    // Forward right away to the project. We
                                    // still forward it to the app so that it
                                    // can update the UI activity indicators.
                                    let _ = project_sender
                                        .send(ProjectServiceInput::Midi(channel, message));
                                }
                                _ => {
                                    // fall through and forward to the app.
                                }
                            }
                            let _ = app_sender.send(MiniDawEvent::MidiPanelEvent(event));
                        }
                    }
                    index if index == project_index => {
                        if let Ok(event) = operation.recv(&project_receiver) {
                            match event {
                                // MIDI messages that came from the project.
                                ProjectServiceEvent::Midi(channel, message) => {
                                    if should_route_midi {
                                        // Fast-route generated MIDI messages so app
                                        // doesn't have to. This handles
                                        // ProjectServiceEvent::Midi, so the app
                                        // should never see it.
                                        let _ = midi_interface_sender.send(
                                            MidiInterfaceServiceInput::Midi(channel, message),
                                        );
                                    }
                                }
                                _ => {
                                    let _ =
                                        app_sender.send(MiniDawEvent::ProjectServiceEvent(event));
                                }
                            }
                        }
                    }
                    index if index == settings_index => {
                        if let Ok(event) = operation.recv(&settings_receiver) {
                            match event {
                                SettingsEvent::ShouldRouteExternally(should) => {
                                    should_route_midi = should;
                                }
                            }
                        }
                    }
                    _ => {
                        panic!("missing case for a new receiver")
                    }
                }
            }
        });
    }

    pub fn sender(&self) -> &Sender<MiniDawInput> {
        &self.input_channels.sender
    }

    pub fn receiver(&self) -> &Receiver<MiniDawEvent> {
        &self.event_channels.receiver
    }
}
