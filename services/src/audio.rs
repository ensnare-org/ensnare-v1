// Copyright (c) 2023 Mike Tsao. All rights reserved.

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample as CpalSample, SizedSample, Stream, SupportedStreamConfig,
};
use crossbeam::queue::ArrayQueue;
use crossbeam_channel::{Receiver, Sender};
use derivative::Derivative;
use ensnare_core::{prelude::*, types::AudioQueue};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Serialize, Deserialize)]
#[serde(remote = "SampleRate", rename_all = "kebab-case")]
struct SampleRateDef(usize);

/// Contains persistent audio settings.
#[derive(Debug, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct AudioSettings {
    #[serde(with = "SampleRateDef")]
    sample_rate: SampleRate,
    #[derivative(Default(value = "2"))]
    channel_count: u16,

    #[serde(skip)]
    has_been_saved: bool,
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

#[derive(Debug)]
pub enum AudioServiceInput {
    Quit, // TODO
}

#[derive(Debug)]
pub enum AudioServiceEvent {
    /// Sample rate, channel count, queue for pushing audio samples.
    Reset(SampleRate, u16, AudioQueue),
    /// The interface requests this many frames of audio ASAP. Provide them by
    /// pushing them into the AudioQueue.
    NeedsAudio(usize),
    /// Sent when the audio interface asked for more frames than we had in the
    /// audio queue.
    Underrun,
}

/// Manages the audio interface.
#[derive(Debug)]
pub struct AudioService {
    input_channels: ChannelPair<AudioServiceInput>,
    event_channels: ChannelPair<AudioServiceEvent>,
    audio_stream: AudioStream,
    config: Arc<Mutex<Option<AudioSettings>>>,
}
impl AudioService {
    pub fn new() -> Self {
        let event_channels: ChannelPair<AudioServiceEvent> = Default::default();
        match AudioStream::create_default_stream(
            AudioStream::REASONABLE_BUFFER_SIZE,
            &event_channels.sender,
        ) {
            Ok(audio_stream) => {
                let r = Self {
                    input_channels: Default::default(),
                    event_channels,
                    audio_stream,
                    config: Default::default(),
                };
                r.spawn_thread();
                r
            }
            Err(e) => panic!("AudioService: {e:?}"),
        }
    }

    fn spawn_thread(&self) {
        let receiver = self.input_channels.receiver.clone();
        let config = Arc::clone(&self.config);

        if let Ok(mut config) = config.lock() {
            *config = Some(AudioSettings::new_with(
                self.audio_stream.sample_rate(),
                self.audio_stream.channel_count(),
            ));
        }
        let _ = self.event_channels.sender.send(AudioServiceEvent::Reset(
            self.audio_stream.sample_rate(),
            self.audio_stream.channel_count(),
            Arc::clone(&self.audio_stream.queue),
        ));

        std::thread::spawn(move || {
            while let Ok(input) = receiver.recv() {
                match input {
                    AudioServiceInput::Quit => {
                        eprintln!("AudioServiceInput::Quit");
                        break;
                    }
                }
            }
            eprintln!("AudioService exit");
        });
    }

    /// The receiver side of the event channel
    pub fn receiver(&self) -> &Receiver<AudioServiceEvent> {
        &self.event_channels.receiver
    }

    /// The sender side of the input channel
    pub fn sender(&self) -> &Sender<AudioServiceInput> {
        &self.input_channels.sender
    }
}

/// Encapsulates the connection to the audio interface.
pub struct AudioStream {
    // cpal config describing the current audio stream.
    config: SupportedStreamConfig,

    // The cpal audio stream.
    #[allow(dead_code)]
    stream: Stream,

    // The queue of samples that the stream consumes.
    queue: AudioQueue,
}
impl Debug for AudioStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioStream")
            .field("config", &"(skipped)")
            .field("stream", &"(skipped)")
            .field("queue", &self.queue)
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
    pub const REASONABLE_BUFFER_SIZE: usize = 4 * 1024;

    pub fn create_default_stream(
        buffer_size: usize,
        sender: &Sender<AudioServiceEvent>,
    ) -> anyhow::Result<Self> {
        let (_host, device, config) = Self::host_device_setup()?;
        let queue = Arc::new(ArrayQueue::new(buffer_size));
        let stream = Self::stream_setup_for(&device, &config, &Arc::clone(&queue), &sender)?;
        let r = Self {
            config,
            stream,
            queue,
        };
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
    #[allow(dead_code)]
    pub fn play(&self) {
        let _ = self.stream.play();
    }

    /// Tells the audio stream to stop playing audio (which means it will also
    /// stop consuming samples from the queue).
    #[allow(dead_code)]
    pub fn pause(&self) {
        let _ = self.stream.pause();
    }

    /// Gives the audio stream a chance to clean up before the thread exits.
    #[allow(dead_code)]
    pub fn quit(&self) {
        todo!()
        //        let _ = self.sender.send(AudioStreamServiceEvent::Quit);
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
        let config = SupportedStreamConfig::new(
            config.channels(),
            config.sample_rate(),
            cpal::SupportedBufferSize::Range { min: 512, max: 512 },
            config.sample_format(),
        );
        Ok((host, device, config))
    }

    /// Creates and returns a Stream for the given device and config. The Stream
    /// will consume the supplied ArrayQueue<f32>. This function is actually a
    /// wrapper around the generic stream_make<T>().
    fn stream_setup_for(
        device: &cpal::Device,
        config: &SupportedStreamConfig,
        queue: &AudioQueue,
        sender: &Sender<AudioServiceEvent>,
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
                Self::stream_make::<f32>(&config.into(), device, queue, sender)
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
        sender: &Sender<AudioServiceEvent>,
    ) -> Result<Stream, anyhow::Error>
    where
        T: SizedSample + FromSample<f32>,
    {
        let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

        let queue = Arc::clone(queue);
        let channel_count = config.channels as usize;
        let sender = sender.clone();
        let stream = device.build_output_stream(
            config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                Self::on_window(output, channel_count, &queue, &sender)
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
        sender: &Sender<AudioServiceEvent>,
    ) where
        T: CpalSample + FromSample<f32>,
    {
        let have_len = queue.len();
        let need_len = output.len();

        // Calculate how many frames we should request.
        let request_len = if have_len < need_len {
            // We're at risk of underrun. Increase work amount beyond what
            // we're about to consume.
            need_len * 2
        } else if have_len > need_len * 2 {
            // We are far ahead of the current window's needs. Replace only half
            // of the current request.
            need_len / 2
        } else {
            // We're keeping up. Replace exactly what we're about to consume.
            need_len
        }
        .min(Self::REASONABLE_BUFFER_SIZE);

        for frame in output.chunks_exact_mut(channel_count) {
            if let Some(sample) = queue.pop() {
                let left = sample.0 .0 as f32;
                let right = sample.1 .0 as f32;
                frame[0] = T::from_sample(left);
                if channel_count > 1 {
                    frame[1] = T::from_sample(right);
                }
            } else {
                let _ = sender.send(AudioServiceEvent::Underrun);

                // No point in continuing to loop.
                break;
            }
        }

        // Don't ask for more than the queue can hold.
        let request_len = (queue.capacity() - queue.len()).min(request_len);

        // Request the frames.
        let _ = sender.send(AudioServiceEvent::NeedsAudio(request_len));
    }
}
