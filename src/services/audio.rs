// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::traits::ProvidesService;
use crate::prelude::*;
use core::fmt::Debug;
use cpal::{
    traits::{DeviceTrait, HostTrait},
    BufferSize, FromSample, Sample as CpalSample, SizedSample, Stream, StreamConfig,
    SupportedStreamConfig,
};
use crossbeam::queue::ArrayQueue;
use crossbeam_channel::Sender;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A ring buffer of stereo samples that the audio stream consumes.
pub(super) type AudioQueue = Arc<ArrayQueue<StereoSample>>;

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
    /// Returns the currently selected audio sample rate, in Hertz (samples per
    /// second).
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Returns the currently selected number of audio channels. In most cases,
    /// this will be two (left channel and right channel).
    pub fn channel_count(&self) -> u16 {
        self.channel_count
    }
}

/// An [AudioServiceInput] tells [AudioService] what to do.
#[derive(Debug)]
pub enum AudioServiceInput {
    /// Exit the service
    Quit,
    /// These audio frames should be made available to the audio interface. They
    /// will be added to [AudioService]'s internal ring buffer and consumed as
    /// needed.
    Frames(Arc<Vec<StereoSample>>),
}

/// [AudioServiceEvent]s inform clients what's going on.
#[derive(Debug)]
pub enum AudioServiceEvent {
    /// The service has initialized or reinitialized. Provides the new sample
    /// rate and channel count.
    Reset(SampleRate, u8),
    /// The audio interface needs audio frames ASAP. Provide the specified
    /// number with [AudioServiceInput::Frames].
    FramesNeeded(usize),
    /// Sent when the audio interface asked for more frames than we had
    /// available in the ring buffer.
    Underrun,
}

/// Wrapper for cpal structs that implements core::fmt::Debug.
struct WrappedStream {
    #[allow(dead_code)]
    // reason = "We need to keep a reference to the service or else it'll be dropped"
    cpal_stream: Stream,

    queue: AudioQueue,
}
impl Debug for WrappedStream {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WrappedStream")
            .field("config", &"(skipped)")
            .field("cpal_stream", &"(skipped)")
            .field("queue", &self.queue)
            .finish()
    }
}
impl WrappedStream {
    pub fn new_with(
        buffer_size: usize,
        sender: &Sender<AudioServiceEvent>,
    ) -> anyhow::Result<Self> {
        let (_host, device, config) = Self::host_device_setup()?;
        let queue = Arc::new(ArrayQueue::new(buffer_size));

        let stream = Self::stream_setup_for(&device, &config, &queue, &sender)?;
        Ok(Self {
            cpal_stream: stream,
            queue,
        })
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
            *config.buffer_size(),
            config.sample_format(),
        );
        Ok((host, device, config))
    }

    /// Creates and returns a Stream for the given device and config. The Stream
    /// will consume the data in the supplied AudioQueue. This function is
    /// actually a wrapper around the generic stream_make<T>().
    fn stream_setup_for(
        device: &cpal::Device,
        config: &SupportedStreamConfig,
        queue: &AudioQueue,
        sender: &Sender<AudioServiceEvent>,
    ) -> anyhow::Result<Stream, anyhow::Error> {
        let config = config.clone();
        let sample_format = config.sample_format();
        let mut config: StreamConfig = config.into();

        // TODO: this is a short-term hack to confirm that good latency with
        // Alsa is possible. (It is!) We do it here, rather than in
        // host_device_setup(), because it's troublesome to create a
        // [SupportedBufferSize] on the fly.
        config.buffer_size = BufferSize::Fixed(512);

        match sample_format {
            cpal::SampleFormat::I8 => {
                Self::stream_make::<i8>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::I16 => {
                Self::stream_make::<i16>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::I32 => {
                Self::stream_make::<i32>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::I64 => {
                Self::stream_make::<i64>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::U8 => {
                Self::stream_make::<u8>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::U16 => {
                Self::stream_make::<u16>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::U32 => {
                Self::stream_make::<u32>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::U64 => {
                Self::stream_make::<u64>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::F32 => {
                Self::stream_make::<f32>(&config.into(), device, queue, sender)
            }
            cpal::SampleFormat::F64 => {
                Self::stream_make::<f64>(&config.into(), device, queue, sender)
            }
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
        T: SizedSample + FromSample<SampleType>,
    {
        let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

        let queue = Arc::clone(queue);
        let sender = sender.clone();
        let channel_count = config.channels as usize;
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

    /// cpal callback that supplies samples from the AudioQueue, converting
    /// them if needed to the stream's expected data type.
    fn on_window<T>(
        output: &mut [T],
        channel_count: usize,
        queue: &AudioQueue,
        sender: &Sender<AudioServiceEvent>,
    ) where
        T: CpalSample + FromSample<SampleType>,
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
        .min(AudioService::REASONABLE_BUFFER_SIZE);

        for frame in output.chunks_exact_mut(channel_count) {
            if let Some(sample) = queue.pop() {
                frame[0] = T::from_sample(sample.0 .0);
                if channel_count > 1 {
                    frame[1] = T::from_sample(sample.1 .0);
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
        let _ = sender.send(AudioServiceEvent::FramesNeeded(request_len));
    }
}

/// [AudioService] provides channel-based communication with the cpal audio
/// interface.
#[derive(Debug)]
pub struct AudioService {
    inputs: CrossbeamChannel<AudioServiceInput>,
    events: CrossbeamChannel<AudioServiceEvent>,

    /// The cpal audio stream.
    #[allow(dead_code)]
    stream: WrappedStream,
}
impl Default for AudioService {
    fn default() -> Self {
        Self::new()
    }
}
impl ProvidesService<AudioServiceInput, AudioServiceEvent> for AudioService {
    fn receiver(&self) -> &crossbeam_channel::Receiver<AudioServiceEvent> {
        &self.events.receiver
    }

    fn sender(&self) -> &Sender<AudioServiceInput> {
        &self.inputs.sender
    }
}
impl AudioService {
    /// This constant is provided to prevent decision paralysis when picking a
    /// `buffer_size` argument. At a typical sample rate of 44.1kHz, a value of
    /// 2048 would mean that samples at the end of a full buffer wouldn't reach
    /// the audio interface for 46.44 milliseconds, which is arguably not
    /// reasonable because audio latency is perceptible at as few as 10
    /// milliseconds. However, on my Ubuntu 20.04 machine, the audio interface
    /// asks for around 2,600 samples (1,300 stereo samples) at once, which
    /// means that 2,048 leaves a cushion of less than a single callback of
    /// samples.
    pub const REASONABLE_BUFFER_SIZE: usize = 4 * 1024;

    #[allow(missing_docs)]
    pub fn new() -> Self {
        let event_channels: CrossbeamChannel<AudioServiceEvent> = Default::default();
        match WrappedStream::new_with(Self::REASONABLE_BUFFER_SIZE, &event_channels.sender) {
            Ok(stream) => {
                let audio_service = Self {
                    inputs: Default::default(),
                    events: event_channels,
                    stream,
                };
                audio_service.start_thread();
                audio_service
            }
            Err(e) => panic!("While creating AudioService: {e:?}"),
        }
    }

    fn start_thread(&self) {
        let receiver = self.inputs.receiver.clone();
        let queue = Arc::clone(&self.stream.queue);
        std::thread::spawn(move || {
            while let Ok(input) = receiver.recv() {
                match input {
                    AudioServiceInput::Quit => {
                        println!("AudioServiceInput2::Quit");
                        break;
                    }
                    AudioServiceInput::Frames(frames) => {
                        for frame in frames.iter() {
                            if queue.force_push(*frame).is_some() {
                                eprintln!("Caution: audio buffer overrun");
                            };
                        }
                    }
                }
            }
        });
    }
}
