// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Select, Sender};
use ensnare_core::{midi_interface::MidiInterfaceInput, types::ChannelPair};
use ensnare_services::{
    AudioService, AudioServiceEvent, AudioServiceInput, MidiService, MidiServiceEvent,
    ProjectService, ProjectServiceEvent, ProjectServiceInput,
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
pub(super) enum EnsnareInput {
    Quit,
}

/// An aggregation of all the service events that the Ensnare app might want to
/// process.
#[derive(Debug)]
pub(super) enum EnsnareEvent {
    MidiPanelEvent(MidiServiceEvent),
    AudioServiceEvent(AudioServiceEvent),
    ProjectServiceEvent(ProjectServiceEvent),
    Quit,
}

#[derive(Debug)]
pub(super) struct EnsnareEventAggregationService {
    input_channels: ChannelPair<EnsnareInput>,
    event_channels: ChannelPair<EnsnareEvent>,

    // The aggregated services. Avoid speaking directly to them; use the
    // channels instead.
    audio_service: AudioService,
    midi_service: MidiService,
    project_service: ProjectService,
}
impl EnsnareEventAggregationService {
    pub fn new_with(
        audio_service: AudioService,
        midi_service: MidiService,
        project_service: ProjectService,
    ) -> Self {
        let r = Self {
            input_channels: Default::default(),
            event_channels: Default::default(),
            audio_service,
            midi_service,
            project_service,
        };
        r.spawn_thread();
        r
    }

    /// Watches all the channel receivers we know about, and either handles them
    /// immediately off the UI thread or forwards them to the app's event
    /// channel.
    fn spawn_thread(&self) {
        let app_sender = self.event_channels.sender.clone();
        let ensnare_receiver = self.input_channels.receiver.clone();
        let audio_sender = self.audio_service.sender().clone();
        let audio_receiver = self.audio_service.receiver().clone();
        let midi_sender = self.midi_service.sender().clone();
        let midi_receiver = self.midi_service.receiver().clone();
        let project_sender = self.project_service.sender().clone();
        let project_receiver = self.project_service.receiver().clone();

        let _ = std::thread::spawn(move || {
            let mut sel = Select::new();
            let ensnare_index = sel.recv(&ensnare_receiver);
            let midi_index = sel.recv(&midi_receiver);
            let audio_index = sel.recv(&audio_receiver);
            let project_index = sel.recv(&project_receiver);

            loop {
                let operation = sel.select();
                match operation.index() {
                    index if index == ensnare_index => {
                        if let Ok(input) = operation.recv(&ensnare_receiver) {
                            match input {
                                EnsnareInput::Quit => {
                                    let _ = audio_sender.send(AudioServiceInput::Quit);
                                    let _ = midi_sender.send(MidiInterfaceInput::Quit);
                                    let _ = project_sender.send(ProjectServiceInput::Quit);
                                    let _ = app_sender.send(EnsnareEvent::Quit);
                                    return;
                                }
                            }
                        }
                    }
                    index if index == audio_index => match operation.recv(&audio_receiver) {
                        Ok(event) => {
                            let _ = app_sender.send(EnsnareEvent::AudioServiceEvent(event));
                        }
                        Err(e) => {
                            eprintln!("audio: {e:?}");
                        }
                    },
                    index if index == midi_index => {
                        match operation.recv(&midi_receiver) {
                            Ok(event) => {
                                match event {
                                    MidiServiceEvent::Midi(channel, message) => {
                                        let _ = project_sender
                                            .send(ProjectServiceInput::Midi(channel, message));
                                        // We still send this through to the app
                                        // so that it can update the UI activity
                                        // indicators, but not as urgently as
                                        // this block.
                                    }
                                    _ => {}
                                }
                                let _ = app_sender.send(EnsnareEvent::MidiPanelEvent(event));
                            }
                            Err(e) => eprintln!("midi: {e:?}"),
                        }
                    }
                    index if index == project_index => match operation.recv(&project_receiver) {
                        Ok(event) => {
                            let _ = app_sender.send(EnsnareEvent::ProjectServiceEvent(event));
                        }
                        Err(e) => {
                            eprintln!("project: {e:?}");
                        }
                    },
                    _ => {
                        panic!("missing case for a new receiver")
                    }
                }
            }
        });
    }

    pub fn sender(&self) -> &Sender<EnsnareInput> {
        &self.input_channels.sender
    }

    pub fn receiver(&self) -> &Receiver<EnsnareEvent> {
        &self.event_channels.receiver
    }
}
