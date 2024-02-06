// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Contains the traits that define many characteristics and relationships among
//! parts of the system.

// Are you making a change to this file? Consider enforcing new trait behavior
// in tests/entity_validator.rs.

use crate::{
    automation::{ControlIndex, ControlValue},
    core::time::{MusicalTime, SampleRate, Tempo, TimeRange, TimeSignature},
    midi::{u7, MidiChannel, MidiEvent, MidiMessage},
    types::TrackUid,
    types::{Normal, Sample, StereoSample},
};

/// Quick import of all important traits.
pub mod prelude {
    pub use super::{
        Configurable, ControlEventsFn, Controllable, Controls, Generates, HandlesMidi, HasExtent,
        HasSettings, IsStereoSampleVoice, IsVoice, MidiMessagesFn, PlaysNotes, Sequences,
        SequencesMidi, Serializable, StoresVoices, Ticks, TransformsAudio, WorkEvent,
    };
}

/// Something that [Generates] creates the given type `<V>` as its work product
/// over time. Examples are envelopes, which produce a [Normal] signal, and
/// oscillators, which produce a [crate::BipolarNormal] signal.
#[allow(unused_variables)]
pub trait Generates<V: Default>: Send + std::fmt::Debug + Ticks {
    /// The value for the current frame. Advance the frame by calling
    /// [Ticks::tick()].
    fn value(&self) -> V {
        V::default()
    }

    /// The batch version of value(). To deliver each value, this method will
    /// typically call tick() internally. If you don't want this, then call
    /// value() on your own.
    fn generate(&mut self, values: &mut [V]) {}

    fn get_next_value(&mut self) -> V {
        self.tick(1);
        self.value()
    }
}

/// Something that is [Controllable] exposes a set of attributes, each with a
/// text name, that an [IsController] can change. If you're familiar with DAWs,
/// this is typically called automation.
///
/// The [Controllable] trait is more powerful than ordinary getters/setters
/// because it allows runtime binding of an [IsController] to a [Controllable].
#[allow(unused_variables)]
pub trait Controllable {
    // See https://stackoverflow.com/a/71988904/344467 to show that we could
    // have made these functions rather than methods (no self). But then we'd
    // lose the ability to query an object without knowing its struct, which is
    // important for the loose binding that the automation system provides.

    /// The number of controllable parameters.
    fn control_index_count(&self) -> usize {
        unimplemented!()
    }
    /// Given a parameter name, return the corresponding index.
    fn control_index_for_name(&self, name: &str) -> Option<ControlIndex> {
        unimplemented!("Controllable trait methods are implemented by the Control #derive macro")
    }
    /// Given a parameter index, return the corresponding name.
    fn control_name_for_index(&self, index: ControlIndex) -> Option<String> {
        unimplemented!()
    }
    /// Given a parameter name and a new value for it, set that parameter's
    /// value.
    fn control_set_param_by_name(&mut self, name: &str, value: ControlValue) {
        unimplemented!()
    }
    /// Given a parameter index and a new value for it, set that parameter's
    /// value.
    fn control_set_param_by_index(&mut self, index: ControlIndex, value: ControlValue) {
        unimplemented!()
    }
}

/// Something that is [Configurable] is interested in staying in sync with
/// global configuration.
pub trait Configurable {
    /// Returns the Entity's sample rate.
    fn sample_rate(&self) -> SampleRate {
        // I was too lazy to add this everywhere when I added this to the trait,
        // but I didn't want unexpected usage to go undetected.
        unimplemented!("Someone asked for a SampleRate but we provided default");
    }

    /// The sample rate changed.
    #[allow(unused_variables)]
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {}

    /// Returns the [Entity]'s [Tempo].
    fn tempo(&self) -> Tempo {
        unimplemented!("Someone forgot to implement tempo()")
    }

    /// Tempo (beats per minute) changed.
    #[allow(unused_variables)]
    fn update_tempo(&mut self, tempo: Tempo) {}

    /// Returns the [Entity]'s [TimeSignature].
    fn time_signature(&self) -> TimeSignature {
        unimplemented!("Someone forgot to implement time_signature()")
    }

    /// The global time signature changed. Recipients are free to ignore this if
    /// they are dancing to their own rhythm (e.g., a polyrhythmic pattern), but
    /// they still want to know it, because they might perform local Time
    /// Signature L in terms of global Time Signature G.
    #[allow(unused_variables)]
    fn update_time_signature(&mut self, time_signature: TimeSignature) {}
}

/// A way for an [Entity] to do work corresponding to one or more frames.
#[deprecated]
pub trait Ticks: Configurable + Send + std::fmt::Debug {
    /// The entity should perform work for the current frame or frames. Under
    /// normal circumstances, successive tick()s represent successive frames.
    /// Exceptions include, for example, restarting a performance, which would
    /// reset the global clock, which the entity learns about via reset().
    ///
    /// Entities are responsible for tracking their own notion of time, which
    /// they should update during tick().
    ///
    /// tick() guarantees that any state for the current frame is valid *after*
    /// tick() has been called for the current frame. This means that Ticks
    /// implementers must treat the first frame as special. Normally, entity
    /// state is correct for the first frame after entity construction, so
    /// tick() must be careful not to update state on the first frame, because
    /// that would cause the state to represent the second frame, not the first.
    fn tick(&mut self, tick_count: usize) {}
}

#[allow(missing_docs)]
pub trait MessageBounds: std::fmt::Debug + Send {}

/// Implementers of [Controls] produce these events. Only the system receives
/// them; rather than forwarding them directly, the system converts them into
/// something else that might then get forwarded to recipients.
#[derive(Clone, Debug)]
pub enum WorkEvent {
    /// A MIDI message sent to a channel.
    Midi(MidiChannel, MidiMessage),

    /// A MIDI message that's limited to a specific track. Lower-level
    /// [WorkEvent::Midi] messages are decorated with the track information when
    /// passing to higher-level processors.
    MidiForTrack(TrackUid, MidiChannel, MidiMessage),

    /// A control event. Indicates that the sender's value has changed, and that
    /// subscribers should receive the update. This is how we perform
    /// automation: a controller produces a [WorkEvent::Control] message, and
    /// the system transforms it into [Controllable::control_set_param_by_index]
    /// method calls to inform subscribing entities that their linked parameters
    /// should change.
    Control(ControlValue),
}
impl MessageBounds for WorkEvent {}

/// Passes [WorkEvent]s to the caller. Used in [Controls::work()].
pub type ControlEventsFn<'a> = dyn FnMut(WorkEvent) + 'a;

/// A device that [Controls] produces [EntityEvent]s that control other things.
/// It also has a concept of a performance that has a beginning and an end. It
/// knows how to respond to requests to start, stop, restart, and seek within
/// the performance.
#[allow(unused_variables)]
pub trait Controls: Send {
    /// Returns the current [MusicalTime] range, or [None] if not performing or
    /// not applicable.
    fn time_range(&self) -> Option<TimeRange> {
        None
    }

    /// Sets the range of [MusicalTime] to which the next [Controls::work()]
    /// method applies.
    ///
    /// Because a project performance often groups many audio frames into a
    /// single batch of work for efficiency reasons, the [TimeRange] is not
    /// necessarily the same as the current audio frame being rendered. Instead,
    /// it is a window that covers the current batch of frames.
    fn update_time_range(&mut self, time_range: &TimeRange) {}

    /// Performs work for the time range specified in the previous
    /// [Controls::update_time_range()]. If the work produces any events,
    /// calling [control_events_fn] asks the system to queue them. They might be
    /// handled right away, or later.
    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {}

    /// Indicates whether this entity has completed all its scheduled work.
    ///
    /// The framework ends the performance only when all entities indicate that
    /// they're finished. Thus, an entity should return false only if it knows
    /// that it has more work to do (such as a sequencer that has not yet
    /// reached the end of its arranged sequences). An entity that performs work
    /// only on command, such as a synthesizer, should always return true;
    /// otherwise, the performance would never end.
    fn is_finished(&self) -> bool {
        true
    }

    /// Tells the entity to play its performance from the current location. A
    /// device *must* refresh [Controls::is_finished()] during this method.
    fn play(&mut self) {}

    /// Tells the device to stop playing its performance. It shouldn't change
    /// its cursor location, so that a [Controls::play()] after a
    /// [Controls::stop()] acts like a resume.
    fn stop(&mut self) {}

    /// Resets cursors to the beginning.
    fn skip_to_start(&mut self) {}

    /// Whether the entity is currently playing.
    //
    // TODO: This is part of the trait so that implementers don't have to leak
    // their internal state to unit test code. Consider removing.
    fn is_performing(&self) -> bool {
        false
    }
}

/// A [TransformsAudio] takes input audio, which is typically produced by
/// [SourcesAudio], does something to it, and then outputs it. It's what effects
/// do.
#[allow(unused_variables)]
pub trait TransformsAudio: std::fmt::Debug {
    /// Transforms a single sample of audio.
    fn transform_audio(&mut self, input_sample: StereoSample) -> StereoSample {
        // Beware: converting from mono to stereo isn't just doing the work
        // twice! You'll also have to double whatever state you maintain from
        // tick to tick that has to do with a single channel's audio data.
        StereoSample(
            self.transform_channel(0, input_sample.0),
            self.transform_channel(1, input_sample.1),
        )
    }

    /// channel: 0 is left, 1 is right. Use the value as an index into arrays.
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        input_sample
    }

    /// Transforms a buffer of audio.
    fn transform_batch(&mut self, samples: &mut [StereoSample]) {
        for sample in samples {
            *sample = self.transform_audio(*sample);
        }
    }
}

/// Describes the public interface of an envelope generator, which provides a
/// normalized amplitude (0.0..=1.0) that changes over time according to its
/// internal parameters, external triggers, and the progression of time.
pub trait GeneratesEnvelope: Generates<Normal> + Send + std::fmt::Debug + Ticks {
    /// Triggers the envelope's active stage.
    fn trigger_attack(&mut self);

    /// Triggers the end of the envelope's active stage.
    fn trigger_release(&mut self);

    /// Requests a fast decrease to zero amplitude. Upon reaching zero, switches
    /// to idle. If the EG is already idle, then does nothing. For normal EGs,
    /// the EG's settings (ADSR, etc.) don't affect the rate of shutdown decay.
    ///
    /// See DSSPC, 4.5 Voice Stealing, for an understanding of how the shutdown
    /// state helps. TL;DR: if we have to steal one voice to play a different
    /// note, it sounds better if the voice very briefly stops and restarts.
    fn trigger_shutdown(&mut self);

    /// Whether the envelope generator is in the idle state, which usually means
    /// quiescent and zero amplitude.
    fn is_idle(&self) -> bool;
}

/// A [PlaysNotes] turns note events into sound. It seems to overlap with
/// [HandlesMidi]; the reason it exists is to allow the two interfaces to evolve
/// independently, because MIDI is unlikely to be perfect for all our needs.
pub trait PlaysNotes {
    /// Whether the entity is currently making sound.
    fn is_playing(&self) -> bool;

    /// Initiates a note-on event. Depending on implementation, might initiate a
    /// steal (tell envelope to go to shutdown state, then do note-on when
    /// that's done).
    fn note_on(&mut self, key: u7, velocity: u7);

    /// Initiates an aftertouch event.
    fn aftertouch(&mut self, velocity: u7);

    /// Initiates a note-off event, which can take a long time to complete,
    /// depending on how long the envelope's release is.
    fn note_off(&mut self, velocity: u7);
}

/// A [StoresVoices] provides access to a collection of voices for a polyphonic
/// synthesizer. Different implementers provide different policies for how to
/// handle voice-stealing.
pub trait StoresVoices: Generates<StereoSample> + Send + Sync + std::fmt::Debug {
    /// The associated type of sample generator for this voice store.
    type Voice;

    /// Generally, this value won't change after initialization, because we try
    /// not to dynamically allocate new voices.
    fn voice_count(&self) -> usize;

    /// The number of voices reporting is_playing() true.
    fn active_voice_count(&self) -> usize;

    /// Fails if we run out of idle voices and can't steal any active ones.
    fn get_voice(&mut self, key: &u7) -> anyhow::Result<&mut Box<Self::Voice>>;

    /// All the voices.
    // Thanks to https://stackoverflow.com/a/58612273/344467 for the lifetime
    // magic
    fn voices<'a>(&'a self) -> Box<dyn Iterator<Item = &Box<Self::Voice>> + 'a>;

    /// All the voices as a mutable iterator.
    fn voices_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Box<Self::Voice>> + 'a>;
}

/// Something that is [Serializable] might need to do work right before
/// serialization, or right after deserialization. These are the hooks.
pub trait Serializable {
    /// Called just before saving to disk.
    fn before_ser(&mut self) {}
    /// Called just after loading from disk.
    fn after_deser(&mut self) {}
}

/// A synthesizer is composed of Voices. Ideally, a synth will know how to
/// construct Voices, and then handle all the MIDI events properly for them.
pub trait IsVoice<V: Default>: Generates<V> + PlaysNotes + Send + Sync {}
/// Same as IsVoice, but stereo.
pub trait IsStereoSampleVoice: IsVoice<StereoSample> {}

/// Each app should have a Settings struct that is composed of subsystems having
/// their own settings. Implementing [HasSettings] helps the composed struct
/// manage its parts.
pub trait HasSettings {
    /// Whether the current state of this struct has been saved to disk.
    fn has_been_saved(&self) -> bool;
    /// Call this whenever the struct changes.
    fn needs_save(&mut self);
    /// Call this after a load() or a save().
    fn mark_clean(&mut self);
}

/// Passes MIDI messages to the caller.
pub type MidiMessagesFn<'a> = dyn FnMut(MidiChannel, MidiMessage) + 'a;

/// Takes standard MIDI messages. Implementers can ignore MidiChannel if it's
/// not important, as the virtual cabling model tries to route only relevant
/// traffic to individual devices. midi_messages_fn allows the implementor to
/// produce more MIDI messages in response to this message. For example, an
/// arpeggiator might produce notes in response to a note-on.
///
/// Note that this method implies that a MIDI message can produce more MIDI
/// messages, but not Control events. Devices can choose to accumulate Control
/// events and send them at the next work() if desired, though doing so will be
/// a work slice laterd.
pub trait HandlesMidi {
    #[allow(missing_docs)]
    #[allow(unused_variables)]
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
    }
}

/// Records and replays MIDI events.
///
/// This trait does not specify behavior in case of duplicate events, which
/// allows simple implementations to use plain vectors rather than sets.
pub trait SequencesMidi: Controls + Configurable + HandlesMidi {
    /// Records a [MidiMessage] at the given [MusicalTime] on the given
    /// [MidiChannel].
    fn record_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        time: MusicalTime,
    ) -> anyhow::Result<()> {
        self.record_midi_event(channel, MidiEvent { message, time })
    }

    /// Records a [MidiEvent] on the given [MidiChannel].
    fn record_midi_event(&mut self, channel: MidiChannel, event: MidiEvent) -> anyhow::Result<()>;

    /// Removes all recorded messages.
    fn clear(&mut self);

    /// Deletes all recorded [MidiMessage]s matching the provided paramaters.
    fn remove_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        time: MusicalTime,
    ) -> anyhow::Result<()> {
        self.remove_midi_event(channel, MidiEvent { message, time })
    }

    /// Deletes all recorded [MidiEvent]s matching the provided paramaters.
    fn remove_midi_event(&mut self, channel: MidiChannel, event: MidiEvent) -> anyhow::Result<()>;

    /// Starts recording. Messages received through
    /// [HandlesMidi::handle_midi_message()] will be recorded as of the start of
    /// the time slice provided by [Controls::update_time()].
    ///
    /// [Controls::stop()] stops recording.
    fn start_recording(&mut self);
    /// Returns whether the sequencer is recording.
    fn is_recording(&self) -> bool;
}

/// A convenience trait that helps describe the lifetime, in MusicalTime, of
/// something.
///
/// This is not necessarily the times of the first and last MIDI events. For
/// example, if the struct in question (MU, or Musical Unit) were one-measure
/// patterns, then the extent of such a pattern would be the full measure, even
/// if the pattern were empty, because it still takes up a measure of "musical
/// space."
///
/// Note that extent() returns a Range, not a RangeInclusive. This is most
/// natural for MUs like patterns that are aligned to musical boundaries. For a
/// MU that is instantaneous, like a MIDI event, however, the current
/// recommendation is to return a range whose end is the last event's time + one
/// MusicalTime unit, which adheres to the contract of Range, but can add an
/// extra measure of silence (since the range now extends to the next measure)
/// if the consumer of extent() doesn't understand what it's looking at.
pub trait HasExtent {
    /// Returns the range of MusicalTime that this thing spans.
    fn extent(&self) -> TimeRange;

    /// Sets the range.
    fn set_extent(&mut self, extent: TimeRange);

    /// Convenience method that returns the distance between extent's start and
    /// end. The duration is the amount of time from the start to the point when
    /// the next contiguous musical item should start. This does not necessarily
    /// mean the time between the first note-on and the first note-off! For
    /// example, an empty 4/4 pattern lasts for 4 beats.
    fn duration(&self) -> MusicalTime {
        let e = self.extent();
        e.0.end - e.0.start
    }
}
/// Records and replays the given musical unit. This is another convenience
/// trait that helps rationalize sequencer interfaces while the concept of a
/// sequencer itself is under development. TODO: delete this trait when
/// sequencing is better developed.
pub trait Sequences: Controls + std::fmt::Debug {
    /// "Musical Unit"
    type MU;

    /// Records an MU to the given [MidiChannel] as of the given [MusicalTime].
    /// An MU normally lasts longer than a single point in [MusicalTime]. In
    /// such a case, `position` indicates the start of the MU, and any durations
    /// or time offsets in the MU are interpreted relative to `position`.
    fn record(
        &mut self,
        channel: MidiChannel,
        unit: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()>;

    /// Deletes all recorded MUs matching the provided paramaters.
    fn remove(
        &mut self,
        channel: MidiChannel,
        unit: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()>;

    /// Removes all recorded MUs.
    fn clear(&mut self);
}

pub trait CanPrototype: std::fmt::Debug + Default {
    fn make_another(&self) -> Self {
        let mut r = Self::default();
        r.update_from_prototype(self);
        r
    }

    fn update_from_prototype(&mut self, prototype: &Self) -> &Self;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Ticks;

    pub trait DebugTicks: Ticks {
        fn debug_tick_until(&mut self, tick_number: usize);
    }
}
