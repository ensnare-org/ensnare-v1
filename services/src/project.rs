// Copyright (c) 2024 Mike Tsao. All rights reserved.

use anyhow::Error;
use crossbeam_channel::{Receiver, Sender};
use derivative::Derivative;
use eframe::egui::Key;
use ensnare_core::{
    prelude::*,
    types::{AudioQueue, VisualizationQueue},
};
use ensnare_entity::prelude::*;
use ensnare_new_stuff::project::Project;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub enum ProjectServiceInput {
    Init,
    New,
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
    KeyEvent(Key, bool, Option<Key>),
    NextTimelineDisplayer,
    AudioQueue(AudioQueue),
    VisualizationQueue(VisualizationQueue),
    NeedsAudio(usize),
}

#[derive(Debug)]
pub enum ProjectServiceEvent {
    // The supplied Project is for the recipient to keep. No need to Arc::clone().
    Loaded(Arc<RwLock<Project>>),
    LoadFailed(PathBuf, Error),
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
            let mut key_handler = KeyHandler::default();
            while let Ok(input) = input_receiver.recv() {
                match input {
                    ProjectServiceInput::Init => {
                        Self::notify_new_project(&event_sender, &project);
                    }
                    ProjectServiceInput::New => {
                        todo!()
                    }
                    ProjectServiceInput::Load(path) => match Project::load(path.clone()) {
                        Ok(mut new_project) => {
                            project.read().unwrap().set_up_successor(&mut new_project);
                            project = Arc::new(RwLock::new(new_project));
                            Self::notify_new_project(&event_sender, &project);
                        }
                        Err(e) => {
                            let _ = event_sender.send(ProjectServiceEvent::LoadFailed(path, e));
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
                            } else {
                                eprintln!("ProjectServiceInput::TrackAddEntity failed");
                            }
                        }
                    }
                    ProjectServiceInput::PatternArrange(track_uid, pattern_uid, position) => {
                        let _ = project.write().unwrap().arrange_pattern(
                            track_uid,
                            pattern_uid,
                            position,
                        );
                    }
                    ProjectServiceInput::LinkControl(source_uid, target_uid, index) => {
                        let _ = project.write().unwrap().link(source_uid, target_uid, index);
                    }
                    ProjectServiceInput::KeyEvent(key, pressed, _physical_key) => {
                        if let Some(message) = key_handler.handle_key(&key, pressed) {
                            project.write().unwrap().handle_midi_message(
                                MidiChannel::default(),
                                message,
                                &mut |c, m| {
                                    eprintln!("TODO: {c:?} {m:?}");
                                },
                            )
                        }
                    }
                    ProjectServiceInput::NextTimelineDisplayer => {
                        project.write().unwrap().advance_arrangement_view_mode();
                    }
                    ProjectServiceInput::AudioQueue(queue) => {
                        project.write().unwrap().e.audio_queue = Some(queue);
                    }
                    ProjectServiceInput::VisualizationQueue(queue) => {
                        project.write().unwrap().e.visualization_queue = Some(queue)
                    }
                    ProjectServiceInput::NeedsAudio(count) => {
                        project.write().unwrap().fill_audio_queue(count);
                    }
                    ProjectServiceInput::Midi(channel, message) => project
                        .write()
                        .unwrap()
                        .handle_midi_message(channel, message, &mut |c, m| {
                            eprintln!("TODO: {c:?} {m:?}");
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

/// Represents an octave as MIDI conventions expect them: A before middle C is
/// in octave 5, and the range is from 0 to 10.
///
/// TODO: I looked around for a bounded integer type or crate, but all made a
/// mountain out of this molehill-sized use case.
#[derive(Debug, Derivative)]
#[derivative(Default)]
struct Octave(#[derivative(Default(value = "5"))] pub u8);
impl Octave {
    fn decrease(&mut self) {
        if self.0 > 0 {
            self.0 -= 1;
        }
    }
    fn increase(&mut self) {
        if self.0 < 10 {
            self.0 += 1;
        }
    }
}

/// Maps [eframe::egui::Key] presses to MIDI events using a piano-keyboard-like
/// layout of QWERTY keys homed at the A-K row. Contains a bit of state, using
/// left/right arrow to change octaves.
#[derive(Debug, Default)]
struct KeyHandler {
    octave: Octave,
}

impl KeyHandler {
    fn handle_key(&mut self, key: &Key, pressed: bool) -> Option<MidiMessage> {
        match key {
            Key::A => Some(self.midi_note_message(0, pressed)),
            Key::W => Some(self.midi_note_message(1, pressed)),
            Key::S => Some(self.midi_note_message(2, pressed)),
            Key::E => Some(self.midi_note_message(3, pressed)),
            Key::D => Some(self.midi_note_message(4, pressed)),
            Key::F => Some(self.midi_note_message(5, pressed)),
            Key::T => Some(self.midi_note_message(6, pressed)),
            Key::G => Some(self.midi_note_message(7, pressed)),
            Key::Y => Some(self.midi_note_message(8, pressed)),
            Key::H => Some(self.midi_note_message(9, pressed)),
            Key::U => Some(self.midi_note_message(10, pressed)),
            Key::J => Some(self.midi_note_message(11, pressed)),
            Key::K => Some(self.midi_note_message(12, pressed)),
            Key::O => Some(self.midi_note_message(13, pressed)),
            Key::ArrowLeft => {
                if pressed {
                    self.octave.decrease();
                }
                None
            }
            Key::ArrowRight => {
                if pressed {
                    self.octave.increase();
                }
                None
            }
            _ => None,
        }
    }

    fn midi_note_message(&self, midi_note_number: u8, pressed: bool) -> MidiMessage {
        let midi_note_number = (midi_note_number + self.octave.0 * 12).min(127);

        if pressed {
            MidiMessage::NoteOn {
                key: u7::from(midi_note_number),
                vel: u7::from(127),
            }
        } else {
            MidiMessage::NoteOff {
                key: u7::from(midi_note_number),
                vel: u7::from(0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_messages_for_keystrokes() {
        let mut k = KeyHandler::default();
        let message = k.handle_key(&Key::A, true).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C4 as u8),
                vel: u7::from(127)
            }
        );
    }

    #[test]
    fn octaves() {
        let mut k = KeyHandler::default();

        // Play a note at initial octave 4.
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C4 as u8),
                vel: u7::from(127)
            }
        );

        // Increase octave and try again.
        let _ = k.handle_key(&Key::ArrowRight, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C5 as u8),
                vel: u7::from(127)
            }
        );

        // Up to maximum octave 10 (AKA octave 9).
        let _ = k.handle_key(&Key::ArrowRight, true);
        let _ = k.handle_key(&Key::ArrowRight, true);
        let _ = k.handle_key(&Key::ArrowRight, true);
        let _ = k.handle_key(&Key::ArrowRight, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C9 as u8),
                vel: u7::from(127)
            }
        );

        let _ = k.handle_key(&Key::ArrowRight, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C9 as u8),
                vel: u7::from(127)
            },
            "Trying to go higher than max octave shouldn't change anything."
        );

        // Now start over and try again with lower octaves.
        let mut k = KeyHandler::default();
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C3 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C2 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C1 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::C0 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::CSub0 as u8),
                vel: u7::from(127)
            }
        );
        let _ = k.handle_key(&Key::ArrowLeft, true);
        let message = k.handle_key(&Key::A, true).unwrap();
        let _ = k.handle_key(&Key::A, false).unwrap();
        assert_eq!(
            message,
            MidiMessage::NoteOn {
                key: u7::from(MidiNote::CSub0 as u8),
                vel: u7::from(127)
            },
            "Trying to go below the lowest octave should stay at lowest octave."
        );
    }
}
