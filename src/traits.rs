// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The traits that define many characteristics and relationships among parts of
//! the system.

/// Quick import of all important traits.
pub mod prelude {
    #[cfg(feature = "egui")]
    pub use super::Displays;
    pub use super::{
        CanPrototype, Configurable, Configurables, ControlEventsFn, ControlProxyEventsFn,
        Controllable, Controls, ControlsAsProxy, DisplaysAction, Generates, GeneratesEnvelope,
        GenerationBuffer, HandlesMidi, HasMetadata, HasSettings, IsStereoSampleVoice, IsVoice,
        MidiMessagesFn, PlaysNotes, Projects, Sequences, SequencesMidi, Serializable, StoresVoices,
        TransformsAudio, WorkEvent,
    };
}

// We re-export here so that consumers of traits don't have to worry as much
// about exactly where they are in the code, but those working on the code can
// still organize them.
pub use ensnare::orchestration::Projects;
pub use ensnare::traits::{
    Configurable, Configurables, ControlEventsFn, ControlProxyEventsFn, Controllable, Controls,
    ControlsAsProxy, Entity, Generates, GenerationBuffer, HandlesMidi, HasMetadata, HasSettings,
    MidiMessagesFn, MidiNoteLabelMetadata, Serializable, TransformsAudio, WorkEvent,
};
#[cfg(feature = "egui")]
pub use ensnare::traits::{Displays, DisplaysAction};

use crate::prelude::*;
use ensnare::prelude::*;

/// Describes the public interface of an envelope generator, which provides a
/// normalized amplitude (0.0..=1.0) that changes over time according to its
/// internal parameters, external triggers, and the progression of time.
pub trait GeneratesEnvelope: Generates<Normal> + Send + core::fmt::Debug {
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
pub trait StoresVoices: Generates<StereoSample> + Send + Sync + core::fmt::Debug {
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

/// A synthesizer is composed of Voices. Ideally, a synth will know how to
/// construct Voices, and then handle all the MIDI events properly for them.
pub trait IsVoice<V: Default + Clone>: Generates<V> + PlaysNotes + Send + Sync {}
/// Same as IsVoice, but stereo.
pub trait IsStereoSampleVoice: IsVoice<StereoSample> {}

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

/// Records and replays the given musical unit. This is another convenience
/// trait that helps rationalize sequencer interfaces while the concept of a
/// sequencer itself is under development. TODO: delete this trait when
/// sequencing is better developed.
pub trait Sequences: Controls + core::fmt::Debug {
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

/// Something that [CanPrototype] can make another of its kind, but it's a
/// little smarter than [Clone]. Not every one of its fields should be cloned --
/// for example, a cache -- and this trait's methods know which is which.
///
/// TODO: this trait overlaps with Serde's functionality. Most fields that are
/// #[serde(skip)] would also be excluded here. Is there a way to hook into
/// Serde and derive the make_another() functionality from it?
pub trait CanPrototype: core::fmt::Debug + Default {
    /// Treats self as a prototype and makes another.
    fn make_another(&self) -> Self {
        let mut r = Self::default();
        r.update_from_prototype(self);
        r
    }

    /// Given another of this kind, updates its fields using self as a
    /// prototype.
    fn update_from_prototype(&mut self, prototype: &Self) -> &Self;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn test_trait_configurable(mut c: impl Configurable) {
        assert_ne!(
            c.sample_rate().0,
            0,
            "Default sample rate should be reasonable"
        );
        let new_sample_rate = SampleRate(3);
        c.update_sample_rate(new_sample_rate);
        assert_eq!(
            c.sample_rate(),
            new_sample_rate,
            "Sample rate should be settable"
        );

        assert!(c.tempo().0 > 0.0, "Default tempo should be reasonable");
        let new_tempo = Tempo(64.0);
        c.update_tempo(new_tempo);
        assert_eq!(c.tempo(), new_tempo, "Tempo should be settable");

        assert_eq!(
            c.time_signature(),
            TimeSignature::default(),
            "time signature should match default"
        );
        let new_time_signature = TimeSignature::new_with(13, 512).unwrap();
        assert_ne!(new_time_signature, TimeSignature::default());
        c.update_time_signature(new_time_signature);
        assert_eq!(
            c.time_signature(),
            new_time_signature,
            "Time signature should be settable"
        );
    }
}
