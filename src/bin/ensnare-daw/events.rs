// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Select, Sender};
use ensnare_core::types::ChannelPair;
use ensnare_new_stuff::project::Project;
use ensnare_services::{AudioServiceEvent, MidiPanelEvent, ProjectServiceEvent};
use std::path::PathBuf;
use strum_macros::FromRepr;
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

/// An aggregation of all the service events that the Ensnare app might want to
/// process.
#[derive(Debug)]
pub(super) enum EnsnareEvent {
    MidiPanelEvent(MidiPanelEvent),
    AudioServiceEvent(AudioServiceEvent),
    ProjectServiceEvent(ProjectServiceEvent),
    ProjectLoaded(Result<Project, LoadError>),
    ProjectSaved(Result<PathBuf, SaveError>),
}

#[derive(Debug, Default)]
pub(super) struct EnsnareEventAggregationService {
    event_channel: ChannelPair<EnsnareEvent>,
}
impl EnsnareEventAggregationService {
    pub fn new_with(
        midi_receiver: &Receiver<MidiPanelEvent>,
        audio_receiver: &Receiver<AudioServiceEvent>,
        project_receiver: &Receiver<ProjectServiceEvent>,
    ) -> Self {
        let r = Self {
            event_channel: Default::default(),
        };
        r.spawn_thread(
            &r.event_channel.sender,
            midi_receiver,
            audio_receiver,
            project_receiver,
        );
        r
    }

    /// Watches all the channel receivers we know about, and either handles them
    /// immediately off the UI thread or forwards them to the app's event
    /// channel.
    fn spawn_thread(
        &self,
        app_sender: &Sender<EnsnareEvent>,
        midi_receiver: &Receiver<MidiPanelEvent>,
        audio_receiver: &Receiver<AudioServiceEvent>,
        project_receiver: &Receiver<ProjectServiceEvent>,
    ) {
        let app_sender = app_sender.clone();
        let midi_receiver = midi_receiver.clone();
        let audio_receiver = audio_receiver.clone();
        let project_receiver = project_receiver.clone();

        let _ = std::thread::spawn(move || -> ! {
            #[derive(FromRepr)]
            enum SelectIndex {
                Midi,
                Audio,
                Project,
            }

            let mut sel = Select::new();
            let r = sel.recv(&midi_receiver);
            debug_assert_eq!(r, SelectIndex::Midi as usize);
            let r = sel.recv(&audio_receiver);
            debug_assert_eq!(r, SelectIndex::Audio as usize);
            let r = sel.recv(&project_receiver);
            debug_assert_eq!(r, SelectIndex::Project as usize);

            loop {
                let operation = sel.select();
                if let Some(index) = SelectIndex::from_repr(operation.index()) {
                    match index {
                        SelectIndex::Midi => {
                            if let Ok(event) = operation.recv(&midi_receiver) {
                                match event {
                                    MidiPanelEvent::Midi(channel, message) => {
                                        // let _ = orchestrator_sender
                                        //     .send(OrchestratorInput::Midi(channel, message));
                                        // We still send this through to the app so
                                        // that it can update the UI activity
                                        // indicators, but not as urgently as this
                                        // block.
                                    }
                                    _ => {}
                                }
                                let _ = app_sender.send(EnsnareEvent::MidiPanelEvent(event));
                            }
                        }
                        SelectIndex::Audio => {
                            if let Ok(event) = operation.recv(&audio_receiver) {
                                let _ = app_sender.send(EnsnareEvent::AudioServiceEvent(event));
                            }
                        }
                        SelectIndex::Project => {
                            if let Ok(event) = operation.recv(&project_receiver) {
                                let _ = app_sender.send(EnsnareEvent::ProjectServiceEvent(event));
                            }
                        }
                        _ => {
                            panic!("missing case for a new receiver")
                        }
                    }
                }
            }
        });
    }

    pub fn receiver(&self) -> &Receiver<EnsnareEvent> {
        &self.event_channel.receiver
    }
}
