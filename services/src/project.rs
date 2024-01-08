// Copyright (c) 2024 Mike Tsao. All rights reserved.

use anyhow::Error;
use crossbeam_channel::{Receiver, Sender};
use ensnare_core::{piano_roll::PatternUid, prelude::*, types::AudioQueue};
use ensnare_entity::prelude::*;
use ensnare_new_stuff::project::Project;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub enum ProjectServiceInput {
    Init,
    Load(PathBuf),
    Save(Option<PathBuf>),
    Midi(MidiChannel, MidiMessage),
    SetSampleRate(SampleRate),
    Play,
    Stop,
    TempInsert16RandomPatterns,
    Quit,
    TrackAddEntity(TrackUid, EntityKey),
    PatternArrange(TrackUid, PatternUid, MusicalTime),
    LinkControl(Uid, Uid, ControlIndex),
    KeyEvent(eframe::egui::Key, bool),
    NextTimelineDisplayer,
    AudioQueue(AudioQueue),
    NeedsAudio(usize),
}

#[derive(Debug)]
pub enum ProjectServiceEvent {
    // The supplied Project is for the recipient to keep. No need to Arc::clone().
    Loaded(Arc<RwLock<Project>>),
    LoadFailed(Error),
    Saved(PathBuf),
    SaveFailed(Error),
    TitleChanged(String),
    IsPerformingChanged(bool),
    Quit,
}

/// A wrapper around a [Project] that provides a channel-based interface to it.
#[derive(Debug)]
pub struct ProjectService {
    input_channels: ChannelPair<ProjectServiceInput>,
    event_channels: ChannelPair<ProjectServiceEvent>,

    factory: Arc<EntityFactory<dyn EntityBounds>>,
}
impl ProjectService {
    pub fn new_with(factory: &Arc<EntityFactory<dyn EntityBounds>>) -> Self {
        let r = Self {
            input_channels: Default::default(),
            event_channels: Default::default(),
            factory: Arc::clone(factory),
        };
        r.spawn_thread();
        let _ = r.sender().send(ProjectServiceInput::Init);
        r
    }

    fn spawn_thread(&self) {
        let input_receiver = self.input_channels.receiver.clone();
        let event_sender = self.event_channels.sender.clone();
        let factory = Arc::clone(&self.factory);
        std::thread::spawn(move || {
            let mut project = Arc::new(RwLock::new(Project::new_project()));
            while let Ok(input) = input_receiver.recv() {
                match input {
                    ProjectServiceInput::Load(path) => match Project::load(path) {
                        Ok(new_project) => {
                            project = Arc::new(RwLock::new(new_project));
                            Self::notify_new_project(&event_sender, &project);
                        }
                        Err(e) => {
                            let _ = event_sender.send(ProjectServiceEvent::LoadFailed(e));
                        }
                    },
                    ProjectServiceInput::Save(path) => {
                        if let Ok(project) = project.read() {
                            match project.save(path) {
                                Ok(save_path) => {
                                    let _ =
                                        event_sender.send(ProjectServiceEvent::Saved(save_path));
                                }
                                Err(e) => {
                                    let _ = event_sender.send(ProjectServiceEvent::SaveFailed(e));
                                }
                            }
                        }
                    }
                    ProjectServiceInput::TempInsert16RandomPatterns => {
                        let _ = project.write().unwrap().temp_insert_16_random_patterns();
                    }
                    ProjectServiceInput::Quit => {
                        eprintln!("ProjectServiceInput::Quit");
                        let _ = event_sender.send(ProjectServiceEvent::Quit);
                        return;
                    }
                    ProjectServiceInput::SetSampleRate(sample_rate) => {
                        project.write().unwrap().update_sample_rate(sample_rate);
                    }
                    ProjectServiceInput::Play => {
                        project.write().unwrap().play();
                        let _ = event_sender.send(ProjectServiceEvent::IsPerformingChanged(true));
                    }
                    ProjectServiceInput::Stop => {
                        project.write().unwrap().stop();
                        let _ = event_sender.send(ProjectServiceEvent::IsPerformingChanged(false));
                    }
                    ProjectServiceInput::TrackAddEntity(track_uid, key) => {
                        if let Ok(mut project) = project.write() {
                            let uid = project.mint_entity_uid();
                            if let Some(entity) = factory.new_entity(key, uid) {
                                let _ = project.add_entity(track_uid, entity, Some(uid));
                                let _ = project
                                    .set_midi_receiver_channel(uid, Some(MidiChannel::default()));
                            }
                        }
                    }
                    ProjectServiceInput::PatternArrange(track_uid, pattern_uid, position) => {
                        let _ = project.write().unwrap().arrange_pattern(
                            &track_uid,
                            &pattern_uid,
                            position,
                        );
                    }
                    ProjectServiceInput::LinkControl(source_uid, target_uid, index) => {
                        let _ = project.write().unwrap().link(source_uid, target_uid, index);
                    }
                    ProjectServiceInput::KeyEvent(_key, _pressed) => {
                        eprintln!("{:?}", input);
                    }
                    ProjectServiceInput::NextTimelineDisplayer => {
                        todo!("self.project.switch_to_next_frontmost_timeline_displayer()");
                    }
                    ProjectServiceInput::Init => {
                        Self::notify_new_project(&event_sender, &project);
                    }
                    ProjectServiceInput::AudioQueue(queue) => {
                        project.write().unwrap().audio_queue = Some(queue);
                    }
                    ProjectServiceInput::NeedsAudio(count) => {
                        project.write().unwrap().fill_audio_queue(count);
                    }
                    ProjectServiceInput::Midi(channel, message) => project
                        .write()
                        .unwrap()
                        .handle_midi_message(channel, message, &mut |c, m| {
                            eprintln!("hey!");
                        }),
                }
            }
            eprintln!("ProjectService exit");
        });
    }

    fn notify_new_project(sender: &Sender<ProjectServiceEvent>, project: &Arc<RwLock<Project>>) {
        let _ = sender.send(ProjectServiceEvent::Loaded(Arc::clone(project)));
    }

    /// The receive side of the [ProjectServiceEvent] channel.
    pub fn receiver(&self) -> &Receiver<ProjectServiceEvent> {
        &self.event_channels.receiver
    }

    /// The sender side of the [ProjectServiceInput] channel.
    pub fn sender(&self) -> &Sender<ProjectServiceInput> {
        &self.input_channels.sender
    }
}
