// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Sender};
use ensnare_core::{piano_roll::PatternUid, prelude::*};
use ensnare_entity::prelude::*;
use ensnare_new_stuff::project::Project;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub enum ProjectServiceInput {
    SetSampleRate(SampleRate),
    Play,
    Stop,
    TempInsert16RandomPatterns,
    Quit,
    TrackAddEntity(TrackUid, EntityKey),
    PatternArrange(TrackUid, PatternUid, MusicalTime),
    LinkControl(Uid, Uid, ControlIndex),
    Save(Option<PathBuf>),
    KeyEvent(eframe::egui::Key, bool),
    NextTimelineDisplayer,
}

#[derive(Debug)]
pub enum ProjectServiceEvent {
    TitleChanged(String),
    IsPerformingChanged(bool),
    Quit,
}

/// A wrapper around a [Project] that provides a channel-based interface to it.
#[derive(Debug)]
pub struct ProjectService {
    project: Arc<RwLock<Project>>,
    input_channels: ChannelPair<ProjectServiceInput>,
    event_channels: ChannelPair<ProjectServiceEvent>,

    factory: Arc<EntityFactory<dyn EntityBounds>>,
}
impl ProjectService {
    pub fn new_with(factory: &Arc<EntityFactory<dyn EntityBounds>>) -> Self {
        let r = Self {
            project: Default::default(),
            input_channels: Default::default(),
            event_channels: Default::default(),
            factory: Arc::clone(factory),
        };
        r.spawn_thread();
        r
    }

    fn spawn_thread(&self) {
        let input_receiver = self.input_channels.receiver.clone();
        let event_sender = self.event_channels.sender.clone();
        let project = Arc::clone(&self.project);
        let factory = Arc::clone(&self.factory);
        std::thread::spawn(move || loop {
            if let Ok(input) = input_receiver.recv() {
                match input {
                    ProjectServiceInput::TempInsert16RandomPatterns => {
                        let _ = project.write().unwrap().temp_insert_16_random_patterns();
                    }
                    ProjectServiceInput::Quit => {
                        eprintln!("ProjectServiceInput::Quit");
                        let _ = event_sender.send(ProjectServiceEvent::Quit);
                        break;
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
                                let _ = project.add_entity(track_uid, entity, None);
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
                    ProjectServiceInput::Save(path) => {
                        todo!("ProjectServiceInput::Save({path:?})")
                    }
                    ProjectServiceInput::KeyEvent(_key, _pressed) => {
                        eprintln!("{:?}", input);
                    }
                    ProjectServiceInput::NextTimelineDisplayer => {
                        todo!("self.project.switch_to_next_frontmost_timeline_displayer()");
                    }
                }
            } else {
                eprintln!("Unexpected failure of MyServiceInput channel");
                break;
            }
        });
    }

    /// The receive side of the [ProjectServiceEvent] channel.
    pub fn receiver(&self) -> &Receiver<ProjectServiceEvent> {
        &self.event_channels.receiver
    }

    /// The sender side of the [ProjectServiceInput] channel.
    pub fn sender(&self) -> &Sender<ProjectServiceInput> {
        &self.input_channels.sender
    }

    pub fn project(&self) -> &Arc<std::sync::RwLock<Project>> {
        &self.project
    }
}
