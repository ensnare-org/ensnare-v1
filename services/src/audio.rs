// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Sender};
use ensnare_core::{
    audio::{AudioStreamService, AudioStreamServiceEvent},
    prelude::*,
    types::AudioQueue,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

// TODO: when we get rid of legacy/, look through here and remove unneeded
// pub(crate).

#[derive(Serialize, Deserialize)]
#[serde(remote = "SampleRate")]
struct SampleRateDef(usize);

/// Contains persistent audio settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioSettings {
    #[serde(with = "SampleRateDef")]
    sample_rate: SampleRate,
    channel_count: u16,

    #[serde(skip)]
    has_been_saved: bool,
}
impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: SampleRate::default(),
            channel_count: 2,
            has_been_saved: false,
        }
    }
}
impl HasSettings for AudioSettings {
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
impl AudioSettings {
    pub(crate) fn new_with(sample_rate: SampleRate, channel_count: u16) -> Self {
        Self {
            sample_rate,
            channel_count,
            has_been_saved: Default::default(),
        }
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn channel_count(&self) -> u16 {
        self.channel_count
    }
}

// Thanks https://boydjohnson.dev/blog/impl-debug-for-fn-type/
pub trait NeedsAudioFnT: FnMut(&AudioQueue, usize) + Sync + Send {}
impl<F> NeedsAudioFnT for F where F: FnMut(&AudioQueue, usize) + Sync + Send {}
/// Takes an [AudioQueue] that accepts [StereoSample]s, and the number of
/// [StereoSample]s that the audio interface has requested.
pub type NeedsAudioFn = Box<dyn NeedsAudioFnT>;

#[derive(Debug)]
pub enum AudioServiceInput {
    Quit, // TODO
}

#[derive(Clone, Debug)]
pub enum AudioServiceEvent {
    /// The audio interface changed, and sample rate etc. might have changed.
    Changed,
}

/// [AudioService] manages the audio interface.
#[derive(Debug, Default)]
pub struct AudioService {
    input_channels: ChannelPair<AudioServiceInput>,
    event_channels: ChannelPair<AudioServiceEvent>,

    config: Arc<Mutex<Option<AudioSettings>>>,
}
impl AudioService {
    /// Construct a new [AudioService].
    pub fn new_with(needs_audio_fn: NeedsAudioFn) -> Self {
        let r = Self::default();
        r.spawn_thread(
            needs_audio_fn,
            AudioStreamService::default().receiver().clone(),
        );

        r
    }

    fn spawn_thread(
        &self,
        mut needs_audio_fn: NeedsAudioFn,
        receiver: Receiver<AudioStreamServiceEvent>,
    ) {
        let config = Arc::clone(&self.config);
        let sender = self.event_channels.sender.clone();
        std::thread::spawn(move || {
            let mut queue_opt = None;
            loop {
                if let Ok(event) = receiver.recv() {
                    match event {
                        AudioStreamServiceEvent::Reset(sample_rate, channel_count, queue) => {
                            if let Ok(mut config) = config.lock() {
                                *config = Some(AudioSettings::new_with(sample_rate, channel_count));
                            }
                            let _ = sender.send(AudioServiceEvent::Changed);
                            queue_opt = Some(queue);
                        }
                        AudioStreamServiceEvent::NeedsAudio(_when, count) => {
                            if let Some(queue) = queue_opt.as_ref() {
                                (*needs_audio_fn)(queue, count);
                            }
                        }
                        AudioStreamServiceEvent::Quit => todo!(),
                    }
                } else {
                    eprintln!("Unexpected failure of AudioInterfaceEvent channel");
                    break;
                }
            }
        });
    }

    /// The audio interface's current sample rate
    pub fn sample_rate(&self) -> SampleRate {
        if let Ok(config) = self.config.lock() {
            if let Some(config) = config.as_ref() {
                return config.sample_rate;
            }
        }
        eprintln!("Warning: returning default sample rate because actual was not available");
        SampleRate::DEFAULT
    }

    /// The audio interface's current number of channels. 1 = mono, 2 = stereo
    pub fn channel_count(&self) -> u16 {
        if let Ok(config) = self.config.lock() {
            if let Some(config) = config.as_ref() {
                return config.channel_count;
            }
        }
        0
    }

    /// The receiver side of the [AudioServiceEvent] channel
    pub fn receiver(&self) -> &Receiver<AudioServiceEvent> {
        &self.event_channels.receiver
    }

    /// The sender side of the [AudioServiceInput] channel
    pub fn sender(&self) -> &Sender<AudioServiceInput> {
        &self.input_channels.sender
    }

    /// Cleans up the audio service for quitting.
    pub fn exit(&self) {
        // TODO: Create the AudioPanelInput channel, add it to the receiver loop, etc.
        eprintln!("Audio Panel acks the quit... TODO");
    }
}
