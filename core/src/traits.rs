// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Contains the traits that define many characteristics and relationships among
//! parts of the system.

// Are you making a change to this file? Consider enforcing new trait behavior
// in tests/entity_validator.rs.

use crate::{
    control::{ControlIndex, ControlValue},
    midi::{u7, MidiChannel, MidiEvent, MidiMessage},
    prelude::*,
    time::{MusicalTime, SampleRate, TimeSignature, ViewRange},
    track::TrackUid,
};

/// Quick import of all important traits.
pub mod prelude {
    pub use super::{
        Acts, Configurable, ControlEventsFn, Controllable, Controls, Displays, Entity, EntityEvent,
        Generates, GeneratesToInternalBuffer, HandlesMidi, HasMetadata, HasSettings, IsAction,
        IsController, IsEffect, IsInstrument, IsStereoSampleVoice, IsVoice, MidiMessagesFn,
        Orchestrates, PlaysNotes, SequencesMidi, Serializable, StoresVoices, Ticks,
        TransformsAudio,
    };
}

#[allow(missing_docs)]
pub trait MessageBounds: std::fmt::Debug + Send {}

/// [Entities](Entity) produce these events to communicate with other Entities.
/// Only the system receives [EntityEvent]s; rather than forwarding them
/// directly, the system converts them into something else.
#[derive(Clone, Debug)]
pub enum EntityEvent {
    /// A MIDI message sent to a channel. Controllers produce this message, and
    /// the system transforms it into one or more
    /// [HandlesMidi::handle_midi_message()] calls to route it to instruments or
    /// other controllers.
    Midi(MidiChannel, MidiMessage),

    /// A control event. Indicates that the sender's value has changed, and that
    /// subscribers should receive the update. This is how we perform
    /// automation: a controller produces a [EntityEvent::Control] message, and
    /// the system transforms it into [Controllable::control_set_param_by_index]
    /// method calls to inform subscribing [Entities](Entity) that their linked
    /// parameters should change.
    Control(ControlValue),
}
impl MessageBounds for EntityEvent {}

/// An [IsController] controls things in the system that implement
/// [Controllable]. Examples are sequencers, arpeggiators, and discrete LFOs (as
/// contrasted with LFOs that are integrated into other instruments).
///
/// [IsController] emits messages, either control messages that the system
/// routes to [Controllable]s, or MIDI messages that go over the MIDI bus.
///
/// An [IsController] is the only kind of entity that can "finish." An
/// [IsEffect] or [IsInstrument] can't finish; they wait forever for audio to
/// process, or MIDI commands to handle. A performance ends once all
/// [IsController] entities indicate that they've finished.
pub trait IsController:
    Controls + HandlesMidi + HasMetadata + Displays + Send + std::fmt::Debug
{
}

/// An [IsEffect] transforms audio. It takes audio inputs and produces audio
/// output. It does not get called unless there is audio input to provide to it
/// (which can include silence, e.g., in the case of a muted instrument).
pub trait IsEffect:
    TransformsAudio + Controllable + Configurable + HasMetadata + Displays + Send + std::fmt::Debug
{
}

/// An [IsInstrument] produces audio, usually upon request from MIDI or
/// [IsController] input.
pub trait IsInstrument:
    Generates<StereoSample>
    + HandlesMidi
    + Controllable
    + HasMetadata
    + Displays
    + Send
    + std::fmt::Debug
{
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
    fn generate_batch_values(&mut self, values: &mut [V]) {}
}

/// [GeneratesToInternalBuffer] is like [Generates], except that the implementer
/// has its own internal buffer where it stores its values. This is useful when
/// we're parallelizing calls and don't want the caller to have to manage a
/// buffer for each parallel operation.
pub trait GeneratesToInternalBuffer<V>: Send + std::fmt::Debug + Ticks {
    /// Do whatever work is necessary to fill the internal buffer with the
    /// specified number of values. Returns the actual number of values
    /// generated.
    fn generate_batch_values(&mut self, len: usize) -> usize;

    /// Returns a reference to the internal buffer. The buffer size is typically
    /// static, so it's important to pay attention to the result of
    /// [GeneratesToInternalBuffer::generate_batch_values()] to know how many
    /// values in the buffer are valid.
    fn values(&self) -> &[V];
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
    fn control_index_for_name(&self, name: &'static str) -> Option<ControlIndex> {
        unimplemented!("Controllable trait methods are implemented by the Control #derive macro")
    }
    /// Given a parameter index, return the corresponding name.
    fn control_name_for_index(&self, index: ControlIndex) -> Option<String> {
        unimplemented!()
    }
    /// Given a parameter name and a new value for it, set that parameter's
    /// value.
    fn control_set_param_by_name(&mut self, name: &'static str, value: ControlValue) {
        unimplemented!()
    }
    /// Given a parameter index and a new value for it, set that parameter's
    /// value.
    fn control_set_param_by_index(&mut self, index: ControlIndex, value: ControlValue) {
        unimplemented!()
    }
}

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

/// Something that is [Configurable] is interested in staying in sync with
/// global configuration.
pub trait Configurable {
    /// Returns the Entity's sample rate.
    fn sample_rate(&self) -> SampleRate {
        // I was too lazy to add this everywhere when I added this to the trait,
        // but I didn't want unexpected usage to go undetected.
        panic!("Someone asked for a SampleRate but we provided default");
    }

    /// The sample rate changed.
    #[allow(unused_variables)]
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {}

    /// Tempo (beats per minute) changed.
    #[allow(unused_variables)]
    fn update_tempo(&mut self, tempo: Tempo) {}

    /// The global time signature changed. Recipients are free to ignore this if
    /// they are dancing to their own rhythm (e.g., a polyrhythmic pattern), but
    /// they still want to know it, because they might perform local Time
    /// Signature L in terms of global Time Signature G.
    #[allow(unused_variables)]
    fn update_time_signature(&mut self, time_signature: TimeSignature) {}
}

/// A way for an [Entity] to do work corresponding to one or more frames.
#[allow(unused_variables)]
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

/// TODO: The [Uid] argument is a little weird. The ones actually producing the
/// messages should *not* be allowed to specify their uid, because we don't want
/// things to be able to impersonate other things. Rather, the ones who are
/// routing messages specify uid, because they know the identity of the entities
/// that they called. So a message-producing entity can specify uid if it wants,
/// but the facility that called it will ignore it and report the correct one.
/// This might end up like MIDI routing: there are some things that ask others
/// to do work, and there are some things that do work, and a transparent proxy
/// API like we have now isn't appropriate.
pub type ControlEventsFn<'a> = dyn FnMut(Option<Uid>, EntityEvent) + 'a;

/// A device that [Controls] produces [EntityEvent]s that control other things.
/// It also has a concept of a performance that has a beginning and an end. It
/// knows how to respond to requests to start, stop, restart, and seek within
/// the performance.
#[allow(unused_variables)]
pub trait Controls: Send {
    /// Sets the range of [MusicalTime] to which the next work() method applies.
    fn update_time(&mut self, range: &ViewRange) {}

    /// The entity should perform work for the time range specified in the
    /// previous [update_time()]. If the work produces any events, use
    /// [control_events_fn] to ask the system to queue them. They might be
    /// handled right away, or later.
    ///
    /// Returns the number of requested ticks handled before terminating (TODO:
    /// no it doesn't).
    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {}

    /// Returns true if the entity is done with all its scheduled work. An
    /// entity that performs work only on command should always return true, as
    /// the framework ends the piece being performed only when all things
    /// implementing [Controls] indicate that they're finished.
    fn is_finished(&self) -> bool {
        true
    }

    /// Tells the device to play its performance from the current location. A
    /// device *must* refresh is_finished() during this method.
    fn play(&mut self) {}

    /// Tells the device to stop playing its performance. It shouldn't change
    /// its cursor location, so that a play() after a stop() acts like a resume.
    fn stop(&mut self) {}

    /// Resets cursors to the beginning. This is set_cursor Lite (TODO).
    fn skip_to_start(&mut self) {}

    /// Whether the device is currently playing. This is part of the trait so
    /// that implementers don't have to leak their internal state to unit test
    /// code.
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

/// [Entity] is a generic type of thing that can have various discoverable
/// capabilities. Almost everything in this system is an Entity of some kind.
/// The implementation of these trait methods is usually generated by the Is
/// proc macros ([IsController], [IsEffect], [IsInstrument], etc.)
#[allow(missing_docs)]
#[typetag::serde(tag = "type")]
pub trait Entity:
    HasMetadata + Displays + Configurable + Serializable + std::fmt::Debug + Send + Sync
{
    fn as_controller(&self) -> Option<&dyn IsController> {
        None
    }
    fn as_controller_mut(&mut self) -> Option<&mut dyn IsController> {
        None
    }
    fn as_effect(&self) -> Option<&dyn IsEffect> {
        None
    }
    fn as_effect_mut(&mut self) -> Option<&mut dyn IsEffect> {
        None
    }
    fn as_instrument(&self) -> Option<&dyn IsInstrument> {
        None
    }
    fn as_instrument_mut(&mut self) -> Option<&mut dyn IsInstrument> {
        None
    }
    fn as_handles_midi(&self) -> Option<&dyn HandlesMidi> {
        None
    }
    fn as_handles_midi_mut(&mut self) -> Option<&mut dyn HandlesMidi> {
        None
    }
    fn as_controllable(&self) -> Option<&dyn Controllable> {
        None
    }
    fn as_controllable_mut(&mut self) -> Option<&mut dyn Controllable> {
        None
    }
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

/// Something that can be called during egui rendering to display a view of
/// itself.
//
// Adapted from egui_demo_lib/src/demo/mod.rs
pub trait Displays {
    /// Renders this Entity. Returns a [Response](egui::Response).
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}

/// Reports an action requested during [Displays::ui()].
pub trait Acts: Displays {
    type Action: IsAction;

    /// Sets the current action. Typically called by the UI code that just
    /// detected a click, drop, change, etc.
    fn set_action(&mut self, action: Self::Action);

    /// Returns the pending action, if any, and resets it to None. Typically
    /// called by the UI code that has the context needed to perform the action.
    fn take_action(&mut self) -> Option<Self::Action>;
}
pub trait IsAction: std::fmt::Debug + std::fmt::Display {}

/// Manages relationships among [Entities](Entity) to produce a song.
pub trait Orchestrates: Configurable {
    /// Creates a new track, returning its [TrackUid] if successful. A track is
    /// a group of musical instruments that together produce a single sample for
    /// every frame of audio. Each track's frame sample is then merged into a
    /// single sample for the audio frame.
    ///
    /// The [TrackUid] should be appended to the internal list of [TrackUid]s.
    fn create_track(&mut self) -> anyhow::Result<TrackUid>;

    /// Returns an ordered list of [TrackUid]s. The ordering of tracks
    /// determines how tracks are presented in a visual rendering of the
    /// project, but it shouldn't affect how the project sounds.
    ///
    /// [TrackUid]s are generally appended to this list as they are created.
    fn track_uids(&self) -> &[TrackUid];

    /// Moves the specified [TrackUid] to the given position. Later [TrackUid]s
    /// are shifted to make room, if needed.
    fn set_track_position(
        &mut self,
        track_uid: TrackUid,
        new_position: usize,
    ) -> anyhow::Result<()>;

    /// Deletes the specified track, disposing of any [Entities](Entity) that it
    /// owns.
    fn delete_track(&mut self, track_uid: &TrackUid);

    /// Deletes the specified tracks. As with [Orchestrates::delete_track()],
    /// disposes of any owned [Entities](Entity).
    fn delete_tracks(&mut self, uids: &[TrackUid]);

    /// Adds the given [Entity] to the end of the specified track. The [Entity]
    /// must have a valid [Uid].
    fn add_entity(&mut self, track_uid: &TrackUid, entity: Box<dyn Entity>) -> anyhow::Result<()>;

    /// Assigns a new [Uid] to the given [Entity] and adds it to the end of the
    /// specified track.
    fn assign_uid_and_add_entity(
        &mut self,
        track_uid: &TrackUid,
        entity: Box<dyn Entity>,
    ) -> anyhow::Result<Uid>;

    /// Removes the specified [Entity], returning ownership (if successful) to
    /// the caller.
    fn remove_entity(&mut self, uid: &Uid) -> anyhow::Result<Box<dyn Entity>>;

    /// Moves the specified [Entity] to the end of the specified track.
    fn set_entity_track(&mut self, new_track_uid: &TrackUid, uid: &Uid) -> anyhow::Result<()>;

    /// Establishes a control link between the source [Entity]'s output and the
    /// given parameter of the target [Entity]'s.
    ///
    /// The global transport has a special [Uid] of 1, and its tempo parameter's
    /// index is zero. Therefore, it's possible to automate the global tempo by
    /// linking something with target Uid 1, control_index zero. Tempo ranges
    /// linearly from 0..=Tempo::MAX_VALUE (currently 1024), so a ControlValue
    /// of 0.125 corresponds to a Tempo of 128 BPM.
    fn link_control(
        &mut self,
        source_uid: Uid,
        target_uid: Uid,
        control_index: ControlIndex,
    ) -> anyhow::Result<()>;

    /// Removes the specified control link, if it exists.
    fn unlink_control(&mut self, source_uid: Uid, target_uid: Uid, control_index: ControlIndex);

    /// Sets the specified effect's wet/dry mix. A humidity of 1.0 is 100%
    /// effect, and 0.0 is 100% unprocessed input. Returns an error if the
    /// entity is not an effect.
    fn set_effect_humidity(&mut self, uid: Uid, humidity: Normal) -> anyhow::Result<()>;

    /// Repositions the specified effect in the track's effects chain.
    ///
    /// Note that ordering matters only for effects, not controllers or
    /// instruments. During a time slice, all controllers perform their work
    /// simultaneously, and all instruments generate signals simultaneously. But
    /// effects operate sequentially. Thus, the first effect operates on the
    /// output of the mixed instruments, and the second effect operates on the
    /// output of the first effect, and so on.
    fn set_effect_position(&mut self, uid: Uid, index: usize) -> anyhow::Result<()>;

    /// Configures a send from the given track to the given aux track. The
    /// `send_amount` parameter indicates how much signal attenuation should
    /// happen before reaching the aux: 1.0 means the full signal should reach
    /// it, and 0.0 means that none of it should.
    ///
    /// Note that send_to_aux(1, 2, Normal(0.0)) can be implemented as
    /// remove_send_to_aux(1, 2), because the behavior is identical.
    fn send(
        &mut self,
        send_track_uid: TrackUid,
        aux_track_uid: TrackUid,
        send_amount: Normal,
    ) -> anyhow::Result<()>;

    /// Removes a send configuration.
    fn remove_send(&mut self, send_track_uid: TrackUid, aux_track_uid: TrackUid);

    /// Sets the level of audio from the given track that reaches the main
    /// mixer.
    fn set_track_output(&mut self, track_uid: TrackUid, output: Normal);

    /// Sets whether the given track is muted.
    fn mute_track(&mut self, track_uid: TrackUid, muted: bool);

    /// Returns which track, if any, is soloing.
    fn solo_track(&self) -> Option<TrackUid>;

    /// Sets the current track that is soloing.
    fn set_solo_track(&mut self, track_uid: TrackUid);

    /// Ends any soloing.
    fn end_solo(&mut self);
}

/// The callback signature for handle_midi_message().
pub type MidiMessagesFn<'a> = dyn FnMut(MidiChannel, MidiMessage) + 'a;

/// Takes standard MIDI messages. Implementers can ignore MidiChannel if it's
/// not important, as the virtual cabling model tries to route only relevant
/// traffic to individual devices. midi_messages_fn allows the implementor to
/// produce more MIDI messages in response to this message. For example, an
/// arpeggiator might produce notes in response to a note-on.
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
    // /// Returns the default [MidiChannel], which is used by recording methods
    // /// that take an optional channel. fn
    // default_midi_recording_channel(&self) -> MidiChannel;

    // /// Sets the default [MidiChannel] for recording. fn
    // set_default_midi_recording_channel(&mut self, channel: MidiChannel);

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

/// Records and replays the given musical unit.
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

#[cfg(test)]
pub(crate) mod tests {
    use super::{Orchestrates, SequencesMidi, Ticks};
    use crate::{
        entities::factory::test_entities::TestInstrument,
        midi::{u7, MidiChannel, MidiMessage},
        prelude::*,
    };
    use more_asserts::assert_gt;
    use std::collections::HashSet;

    pub trait DebugTicks: Ticks {
        fn debug_tick_until(&mut self, tick_number: usize);
    }

    pub(crate) fn validate_orchestrates_trait(orchestrates: &mut dyn Orchestrates) {
        assert!(
            orchestrates.track_uids().is_empty(),
            "Initial impl should have no tracks"
        );
        let track_1_uid = orchestrates.create_track().unwrap();
        assert_gt!(track_1_uid.0, 0, "new track's uid should be nonzero");
        assert_eq!(
            orchestrates.track_uids().len(),
            1,
            "should be one track after creating one"
        );

        let track_2_uid = orchestrates.create_track().unwrap();
        assert_eq!(
            orchestrates.track_uids().len(),
            2,
            "should be two tracks after creating second"
        );
        assert!(orchestrates.set_track_position(track_2_uid, 0).is_ok());
        assert_eq!(
            orchestrates.track_uids(),
            vec![track_2_uid, track_1_uid],
            "order of track uids should be as expected after move"
        );
        orchestrates.delete_track(&track_2_uid);

        let target_uid = orchestrates
            .assign_uid_and_add_entity(&track_1_uid, Box::new(TestInstrument::default()))
            .unwrap();
        assert!(
            orchestrates
                .link_control(Uid(123), target_uid, ControlIndex(7))
                .is_ok(),
            "Linking control to a known target Uid should work"
        );
        orchestrates.unlink_control(Uid(234), Uid(345), ControlIndex(8));

        orchestrates.delete_track(&TrackUid(99999));
        assert_eq!(
            orchestrates.track_uids().len(),
            1,
            "Deleting nonexistent track shouldn't change anything"
        );

        let mut ids: HashSet<Uid> = HashSet::default();
        for _ in 0..64 {
            let e = Box::new(TestInstrument::default());
            let uid = orchestrates
                .assign_uid_and_add_entity(&track_1_uid, e)
                .unwrap();
            assert!(
                !ids.contains(&uid),
                "added entities should be assigned unique IDs"
            );
            ids.insert(uid);
        }

        orchestrates.delete_track(&track_1_uid);
        assert!(
            orchestrates.track_uids().is_empty(),
            "Deleting track should change track count"
        );

        assert!(
            orchestrates.solo_track().is_none(),
            "No track should be soloing at first"
        );
        orchestrates.set_solo_track(track_1_uid);
        assert_eq!(
            orchestrates.solo_track(),
            Some(track_1_uid),
            "set_solo_track() should work"
        );
        orchestrates.end_solo();
        assert!(
            orchestrates.solo_track().is_none(),
            "No track should be soloing after end_solo()"
        );
    }

    fn replay_messages(
        sequences_midi: &mut dyn SequencesMidi,
        start_time: MusicalTime,
        duration: MusicalTime,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        let mut v = Vec::default();
        sequences_midi.update_time(&(start_time..start_time + duration));
        sequences_midi.work(&mut |_, event| match event {
            crate::traits::EntityEvent::Midi(channel, message) => v.push((channel, message)),
            crate::traits::EntityEvent::Control(_) => panic!(),
        });
        v
    }

    fn replay_all_messages(
        sequences_midi: &mut dyn SequencesMidi,
    ) -> Vec<(MidiChannel, MidiMessage)> {
        replay_messages(
            sequences_midi,
            MusicalTime::TIME_ZERO,
            MusicalTime::TIME_MAX,
        )
    }

    /// Validates the provided implementation of [SequencesMidi].
    pub(crate) fn validate_sequences_midi_trait(sequences: &mut dyn SequencesMidi) {
        const SAMPLE_NOTE_ON_MESSAGE: MidiMessage = MidiMessage::NoteOn {
            key: u7::from_int_lossy(60),
            vel: u7::from_int_lossy(100),
        };
        const SAMPLE_NOTE_OFF_MESSAGE: MidiMessage = MidiMessage::NoteOff {
            key: u7::from_int_lossy(60),
            vel: u7::from_int_lossy(100),
        };
        const SAMPLE_MIDI_CHANNEL: MidiChannel = MidiChannel(7);

        assert!(replay_all_messages(sequences).is_empty());
        assert!(sequences
            .record_midi_message(
                SAMPLE_MIDI_CHANNEL,
                SAMPLE_NOTE_OFF_MESSAGE,
                MusicalTime::START
            )
            .is_ok());
        assert_eq!(
            replay_all_messages(sequences).len(),
            1,
            "sequencer should contain one recorded message"
        );
        sequences.clear();
        assert!(replay_all_messages(sequences).is_empty());

        assert!(
            sequences.is_finished(),
            "An empty sequencer should always be finished."
        );
        assert!(
            !sequences.is_performing(),
            "A sequencer should not be performing before play()"
        );

        let mut do_nothing = |_, _| {};

        assert!(!sequences.is_recording());
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_ON_MESSAGE,
            &mut do_nothing,
        );
        assert!(
            replay_all_messages(sequences).is_empty(),
            "sequencer should ignore incoming messages when not recording"
        );

        sequences.start_recording();
        assert!(sequences.is_recording());
        sequences.update_time(&(MusicalTime::new_with_beats(1)..MusicalTime::DURATION_QUARTER));
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_ON_MESSAGE,
            &mut do_nothing,
        );
        sequences.update_time(&(MusicalTime::new_with_beats(2)..MusicalTime::DURATION_QUARTER));
        sequences.handle_midi_message(
            MidiChannel::default(),
            SAMPLE_NOTE_OFF_MESSAGE,
            &mut do_nothing,
        );
        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should reflect recorded messages even while recording"
        );
        sequences.stop();
        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should reflect recorded messages after recording"
        );

        assert!(
            replay_messages(
                sequences,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_QUARTER,
            )
            .is_empty(),
            "sequencer should replay no events for time slice before recorded events"
        );

        assert_eq!(
            replay_messages(
                sequences,
                MusicalTime::new_with_beats(1),
                MusicalTime::DURATION_QUARTER,
            )
            .len(),
            1,
            "sequencer should produce appropriate messages for time slice"
        );

        assert_eq!(
            replay_all_messages(sequences).len(),
            2,
            "sequencer should produce appropriate messages for time slice"
        );
    }
}
