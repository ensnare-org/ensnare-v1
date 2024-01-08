// Copyright (c) 2023 Mike Tsao. All rights reserved.

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample as CpalSample, SizedSample, Stream, SupportedStreamConfig,
};
use crossbeam::queue::ArrayQueue;
use crossbeam_channel::{Receiver, Sender};
use ensnare_core::{prelude::*, types::AudioQueue};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    time::Instant,
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

#[derive(Debug)]
pub enum AudioServiceEvent {
    /// The audio interface changed. Sample rate etc. might have changed.
    Changed(AudioQueue),
    NeedsAudio(usize),
}

/// [AudioService] manages the audio interface.
#[derive(Debug)]
pub struct AudioService {
    input_channels: ChannelPair<AudioServiceInput>,
    event_channels: ChannelPair<AudioServiceEvent>,
    audio_stream_service: AudioStreamService,

    config: Arc<Mutex<Option<AudioSettings>>>,
}
impl AudioService {
    /// Construct a new [AudioService].
    pub fn new_with() -> Self {
        let r = Self {
            input_channels: Default::default(),
            event_channels: Default::default(),
            audio_stream_service: AudioStreamService::new(),
            config: Default::default(),
        };
        r.spawn_thread();
        r
    }

    fn spawn_thread(&self) {
        let config = Arc::clone(&self.config);
        let sender = self.event_channels.sender.clone();
        let receiver = self.audio_stream_service.receiver().clone();
        std::thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(event) => match event {
                        AudioStreamServiceEvent::Reset(sample_rate, channel_count, queue) => {
                            if let Ok(mut config) = config.lock() {
                                *config = Some(AudioSettings::new_with(sample_rate, channel_count));
                            }
                            let _ = sender.send(AudioServiceEvent::Changed(queue));
                        }
                        AudioStreamServiceEvent::NeedsAudio(_when, count) => {
                            let _ = sender.send(AudioServiceEvent::NeedsAudio(count));
                        }
                        AudioStreamServiceEvent::Quit => {
                            eprintln!("AudioStreamServiceEvent::Quit");
                            return;
                        }
                    },
                    Err(e) => {
                        eprintln!("AudioService {e:?}");
                        break;
                    }
                }
            }
            eprintln!("AudioService exit");
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

#[derive(Debug)]
pub enum AudioStreamServiceInput {
    SetBufferSize(usize),
    Play,
    Pause,

    // Requests the audio stream to quit.
    Quit,
}

#[derive(Clone, Debug)]
pub enum AudioStreamServiceEvent {
    /// Sample rate, channel count, queue for pushing audio samples
    Reset(SampleRate, u16, AudioQueue),

    /// A timestamp for measuring latency, and the number of samples requested
    /// by the audio interface
    NeedsAudio(Instant, usize),

    // Acknowledges the AudioInterfaceInput::Quit and confirms that we've ended
    // the stream.
    Quit,
}

/// Wraps the [cpal] audio interface and makes it easy to manage with crossbeam
/// channels.
#[derive(Debug)]
pub struct AudioStreamService {
    input_channels: ChannelPair<AudioStreamServiceInput>,
    event_channels: ChannelPair<AudioStreamServiceEvent>,
}
impl AudioStreamService {
    pub fn new() -> Self {
        let r = Self {
            input_channels: Default::default(),
            event_channels: Default::default(),
        };
        r.spawn_thread();
        r
    }

    pub fn sender(&self) -> &Sender<AudioStreamServiceInput> {
        &self.input_channels.sender
    }

    pub fn receiver(&self) -> &Receiver<AudioStreamServiceEvent> {
        &self.event_channels.receiver
    }

    fn spawn_thread(&self) {
        let receiver = self.input_channels.receiver.clone();
        let sender = self.event_channels.sender.clone();

        let _ = std::thread::spawn(move || {
            match AudioStream::create_default_stream(AudioStream::REASONABLE_BUFFER_SIZE, &sender) {
                Ok(audio_stream) => {
                    while let Ok(input) = receiver.recv() {
                        match input {
                            AudioStreamServiceInput::SetBufferSize(_new_size) => todo!(),
                            AudioStreamServiceInput::Play => audio_stream.play(),
                            AudioStreamServiceInput::Pause => audio_stream.pause(),
                            AudioStreamServiceInput::Quit => {
                                eprintln!("AudioStreamServiceInput::Quit");
                                audio_stream.quit();
                                return;
                            }
                        }
                    }
                    eprintln!("AudioStreamService exit");
                }
                Err(e) => {
                    eprintln!("AudioStreamService: default stream creation failed: {e:?}");
                }
            }
        });
    }
}

/// Encapsulates the connection to the audio interface.
pub struct AudioStream {
    // cpal config describing the current audio stream.
    config: SupportedStreamConfig,

    // The cpal audio stream.
    stream: Stream,

    // The queue of samples that the stream consumes.
    queue: AudioQueue,

    // The sending half of the channel that the audio stream uses to send
    // updates to the subscription.
    sender: Sender<AudioStreamServiceEvent>,
}
impl Debug for AudioStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioStream")
            .field("config", &"(skipped)")
            .field("stream", &"(skipped)")
            .field("queue", &self.queue)
            .field("sender", &self.sender)
            .finish()
    }
}
impl AudioStream {
    /// This constant is provided to prevent decision paralysis when picking a
    /// `buffer_size` argument. At a typical sample rate of 44.1KHz, a value of
    /// 2048 would mean that samples at the end of a full buffer wouldn't reach
    /// the audio interface for 46.44 milliseconds, which is arguably not
    /// reasonable because audio latency is perceptible at as few as 10
    /// milliseconds. However, on my Ubuntu 20.04 machine, the audio interface
    /// asks for around 2,600 samples (1,300 stereo samples) at once, which
    /// means that 2,048 leaves a cushion of less than a single callback of
    /// samples.
    pub const REASONABLE_BUFFER_SIZE: usize = 2048;

    pub fn create_default_stream(
        buffer_size: usize,
        audio_stream_event_sender: &Sender<AudioStreamServiceEvent>,
    ) -> anyhow::Result<Self> {
        let (_host, device, config) = Self::host_device_setup()?;
        let queue = Arc::new(ArrayQueue::new(buffer_size));
        let stream = Self::stream_setup_for(
            &device,
            &config,
            &Arc::clone(&queue),
            audio_stream_event_sender.clone(),
        )?;
        let r = Self {
            config,
            stream,
            queue,
            sender: audio_stream_event_sender.clone(),
        };
        r.send_reset();
        Ok(r)
    }

    /// Returns the sample rate of the current audio stream.
    pub fn sample_rate(&self) -> SampleRate {
        let config: &cpal::StreamConfig = &self.config.clone().into();
        SampleRate::new(config.sample_rate.0 as usize)
    }

    /// Returns the channel count of the current audio stream.
    pub fn channel_count(&self) -> u16 {
        let config: &cpal::StreamConfig = &self.config.clone().into();
        config.channels
    }

    /// Tells the audio stream to resume playing audio (and consuming samples
    /// from the queue).
    pub fn play(&self) {
        let _ = self.stream.play();
    }

    /// Tells the audio stream to stop playing audio (which means it will also
    /// stop consuming samples from the queue).
    pub fn pause(&self) {
        let _ = self.stream.pause();
    }

    /// Gives the audio stream a chance to clean up before the thread exits.
    pub fn quit(&self) {
        let _ = self.sender.send(AudioStreamServiceEvent::Quit);
    }

    /// Returns the default host, device, and stream config (all of which are
    /// cpal concepts).
    fn host_device_setup(
    ) -> anyhow::Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error>
    {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
        let config = device.default_output_config()?;
        Ok((host, device, config))
    }

    /// Creates and returns a Stream for the given device and config. The Stream
    /// will consume the supplied ArrayQueue<f32>. This function is actually a
    /// wrapper around the generic stream_make<T>().
    fn stream_setup_for(
        device: &cpal::Device,
        config: &SupportedStreamConfig,
        queue: &AudioQueue,
        audio_stream_event_sender: Sender<AudioStreamServiceEvent>,
    ) -> anyhow::Result<Stream, anyhow::Error> {
        let config = config.clone();

        match config.sample_format() {
            cpal::SampleFormat::I8 => todo!(),
            cpal::SampleFormat::I16 => todo!(),
            cpal::SampleFormat::I32 => todo!(),
            cpal::SampleFormat::I64 => todo!(),
            cpal::SampleFormat::U8 => todo!(),
            cpal::SampleFormat::U16 => todo!(),
            cpal::SampleFormat::U32 => todo!(),
            cpal::SampleFormat::U64 => todo!(),
            cpal::SampleFormat::F32 => {
                Self::stream_make::<f32>(&config.into(), device, queue, audio_stream_event_sender)
            }
            cpal::SampleFormat::F64 => todo!(),
            _ => todo!(),
        }
    }

    /// Generic portion of stream_setup_for().
    fn stream_make<T>(
        config: &cpal::StreamConfig,
        device: &cpal::Device,
        queue: &AudioQueue,
        audio_stream_event_sender: Sender<AudioStreamServiceEvent>,
    ) -> Result<Stream, anyhow::Error>
    where
        T: SizedSample + FromSample<f32>,
    {
        let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

        let queue = Arc::clone(queue);
        let channel_count = config.channels as usize;
        let stream = device.build_output_stream(
            config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                Self::on_window(
                    output,
                    channel_count,
                    &queue,
                    audio_stream_event_sender.clone(),
                )
            },
            err_fn,
            None,
        )?;
        Ok(stream)
    }

    /// cpal callback that supplies samples from the [AudioQueue], converting
    /// them if needed to the stream's expected data type.
    fn on_window<T>(
        output: &mut [T],
        channel_count: usize,
        queue: &AudioQueue,
        audio_stream_event_sender: Sender<AudioStreamServiceEvent>,
    ) where
        T: CpalSample + FromSample<f32>,
    {
        for frame in output.chunks_exact_mut(channel_count) {
            let sample = queue.pop().unwrap_or_default();
            let left = sample.0 .0 as f32;
            let right = sample.1 .0 as f32;
            frame[0] = T::from_sample(left);
            if channel_count > 1 {
                frame[1] = T::from_sample(right);
            }
        }
        let capacity = queue.capacity();
        let len = queue.len();
        if len < capacity {
            let _ = audio_stream_event_sender.send(AudioStreamServiceEvent::NeedsAudio(
                Instant::now(),
                capacity - len,
            ));
        }
    }

    fn send_reset(&self) {
        let _ = self.sender.send(AudioStreamServiceEvent::Reset(
            self.sample_rate(),
            self.channel_count(),
            Arc::clone(&self.queue),
        ));
    }
}
