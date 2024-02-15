// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The traits that define many characteristics and relationships among parts of
//! the system.

/// Quick import of all important traits.
pub mod prelude {
    #[cfg(feature = "egui")]
    pub use super::Displays;
    pub use super::{
        CanPrototype, Configurable, Configurables, ControlEventsFn, ControlProxyEventsFn,
        Controllable, Controls, ControlsAsProxy, EntityBounds, Generates, GeneratesEnvelope,
        HandlesMidi, HasExtent, HasMetadata, HasSettings, IsStereoSampleVoice, IsVoice,
        MidiMessagesFn, PlaysNotes, Sequences, SequencesMidi, Serializable, StoresVoices, Ticks,
        TransformsAudio, WorkEvent,
    };
}

// We re-export here so that consumers of traits don't have to worry as much
// about exactly where they are in the code, but those working on the code can
// still organize them.
pub use crate::automation::{
    ControlEventsFn, ControlProxyEventsFn, Controllable, Controls, ControlsAsProxy,
};

use crate::prelude::*;
use strum_macros::Display;

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

/// A convenience struct for the fields implied by [Configurable]. Note that
/// this struct is not serde-compliant, because these fields typically aren't
/// meant to be serialized.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Configurables {
    sample_rate: SampleRate,
    tempo: Tempo,
    time_signature: TimeSignature,
}
impl Configurable for Configurables {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate
    }

    fn tempo(&self) -> Tempo {
        self.tempo
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo
    }

    fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature
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

/// An [Entity] is a generic musical instrument, which includes MIDI
/// instruments like synths, effects like reverb, and controllers like MIDI
/// sequencers. Almost everything in this system is an Entity of some kind. A
/// struct's implementation of these trait methods is usually generated by the
/// [IsEntity](ensnare_proc_macros::IsEntity) proc macro.
#[typetag::serde(tag = "type")]
pub trait Entity:
    HasMetadata
    + Controls
    + Controllable
    + Displays
    + Generates<StereoSample>
    + HandlesMidi
    + Serializable
    + TransformsAudio
    + std::fmt::Debug
    + Send
    + Sync
{
}

#[typetag::serde(tag = "type")]
pub trait EntityBounds: Entity {}

/// A [HasMetadata] has basic information about an [Entity]. Some methods apply
/// to the "class" of [Entity] (for example, all `ToyInstrument`s share the name
/// "ToyInstrument"), and others apply to each instance of a class (for example,
/// one ToyInstrument instance might be Uid 42, and another Uid 43).
pub trait HasMetadata {
    /// The [Uid] is a globally unique identifier for an instance of an
    /// [Entity].
    fn uid(&self) -> Uid;
    /// Assigns a [Uid].
    fn set_uid(&mut self, uid: Uid);
    /// A string that describes this class of [Entity]. Suitable for debugging
    /// or quick-and-dirty UIs.
    fn name(&self) -> &'static str;
    /// A kebab-case string that identifies this class of [Entity].
    fn key(&self) -> &'static str;
}

#[cfg(feature = "egui")]
#[derive(Debug, Display)]
pub enum DisplaysAction {
    // During the ui() call, the entity determined that something wants to link
    // with us at control param index ControlIndex.
    Link(crate::egui::ControlLinkSource, ControlIndex),
}

#[cfg(feature = "egui")]
/// Something that can be called during egui rendering to display a view of
/// itself.
//
// Adapted from egui_demo_lib/src/demo/mod.rs
pub trait Displays {
    /// Renders this Entity. Returns a [Response](egui::Response).
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }

    fn set_action(&mut self, action: DisplaysAction) {}
    /// Also resets the action to None
    fn take_action(&mut self) -> Option<DisplaysAction> {
        None
    }

    /// Indicates which section of the timeline is being displayed. Entities
    /// that don't render in the timeline can ignore this.
    #[allow(unused_variables)]
    fn set_view_range(&mut self, view_range: &ViewRange) {}
}
#[cfg(not(feature = "egui"))]
pub trait Displays {}

#[cfg(test)]
pub(crate) mod tests {
    use super::Ticks;

    pub trait DebugTicks: Ticks {
        fn debug_tick_until(&mut self, tick_number: usize);
    }
}
