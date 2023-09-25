// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Built-in musical instruments and supporting components.

// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    control::ControlTrip,
    controllers::Timer,
    entities::{
        controllers::{
            arpeggiator::{Arpeggiator, ArpeggiatorParams},
            lfo::{LfoController, LfoControllerParams},
            SignalPassthroughController,
        },
        effects::{
            bitcrusher::Bitcrusher,
            compressor::Compressor,
            filter::{BiQuadFilterLowPass24db, BiQuadFilterLowPass24dbParams},
            gain::{Gain, GainParams},
            limiter::Limiter,
            mixer::Mixer,
            reverb::{Reverb, ReverbParams},
        },
        instruments::{
            drumkit::{Drumkit, DrumkitParams},
            fm::{FmSynth, FmSynthParams},
            sampler::{Sampler, SamplerParams},
            welsh::{WelshSynth, WelshSynthParams},
        },
        test_entities::TestControllerAlwaysSendsMidiMessage,
        toys::{ToyController, ToyEffect, ToyInstrument, ToySynth, ToySynthParams},
    },
    generators::Waveform,
    midi::MidiChannel,
    mini_sequencer::SequencerBuilder,
    prelude::*,
    traits::prelude::*,
    uid::Uid,
    utils::Paths,
};
use anyhow::anyhow;
use atomic_counter::{AtomicCounter, RelaxedCounter};
use derive_more::Display;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, HashMap, HashSet},
    option::Option,
};

/// Recommended imports for easy onboarding.
// pub mod prelude {
//     pub use crate::entities::{
//         controllers::{
//             arpeggiator::{Arpeggiator, ArpeggiatorParams},
//             calculator::Calculator,
//             lfo::{LfoController, LfoControllerParams},
//             SignalPassthroughController, SignalPassthroughControllerParams, SignalPassthroughType,
//         },
//         effects::{
//             chorus::{Chorus, ChorusParams},
//             delay::{Delay, DelayParams, RecirculatingDelayLine},
//             filter::{
//                 BiQuadFilterAllPass, BiQuadFilterBandPass, BiQuadFilterLowPass24db,
//                 BiQuadFilterLowPass24dbParams,
//             },
//             gain::{Gain, GainParams},
//             reverb::{Reverb, ReverbParams},
//         },
//         instruments::welsh::{WelshSynth, WelshSynthParams},
//     };
// }

/// Controllers implement the [IsController](ensnare_core::traits::IsController)
/// trait, which means that they control other devices. An example of a
/// controller is a [Sequencer](ensnare_entities::controllers::Sequencer), which
/// produces MIDI messages.
///
/// Generally, controllers produce only control signals, and not audio. But
/// adapters exist that change one kind of signal into another, such as
/// [SignalPassthroughController], which is used in
/// [sidechaining](https://en.wikipedia.org/wiki/Dynamic_range_compression#Side-chaining).
/// In theory, a similar adapter could be used to change a control signal into
/// an audio signal.
pub mod controllers;

/// Effects implement the [IsEffect](ensnare_core::traits::IsEffect) trait, which
/// means that they transform audio. They don't produce their own audio, and
/// while they don't produce control signals, most of them do respond to
/// controls. Examples of effects are [Compressor](crate::effects::Compressor),
/// [BiQuadFilter](crate::effects::filter::BiQuadFilter), and
/// [Reverb](crate::effects::Reverb).
pub mod effects;

/// Instruments play sounds. They implement the
/// [IsInstrument](ensnare_core::traits::IsInstrument) trait, which means that
/// they respond to MIDI and produce [StereoSamples](ensnare_core::StereoSample).
/// Examples of instruments are [Sampler](crate::instruments::Sampler) and
/// [WelshSynth](crate::instruments::WelshSynth).
pub mod instruments;

/// Very simple entities that are good for development and testing.
pub mod toys;

/// Registers all [EntityFactory]'s entities. Note that the function returns a
/// EntityFactory, rather than operating on an &mut. This encourages
/// one-and-done creation, after which the factory is immutable:
///
/// ```ignore
/// let factory = register_factory_entities(EntityFactory::default());
/// ```
///
/// TODO: maybe a Builder pattern is better, so that people can compose
/// factories out of any entities they want, and still get the benefits of
/// immutability.
#[must_use]
pub fn register_factory_entities(mut factory: EntityFactory) -> EntityFactory {
    // TODO: might be nice to move HasUid::name() to be a function.

    // Controllers
    factory.register_entity(EntityKey::from("arpeggiator"), || {
        Box::new(Arpeggiator::new_with(
            &ArpeggiatorParams::default(),
            MidiChannel(0),
        ))
    });
    factory.register_entity(EntityKey::from("control-trip"), || {
        Box::new(ControlTrip::default())
    });
    factory.register_entity(EntityKey::from("lfo"), || {
        Box::new(LfoController::new_with(&LfoControllerParams {
            frequency: FrequencyHz::from(0.2),
            waveform: Waveform::Sawtooth,
        }))
    });
    factory.register_entity(EntityKey::from("sequencer"), || {
        Box::new(
            SequencerBuilder::default()
                .midi_channel_out(MidiChannel(0))
                .build()
                .unwrap(),
        )
    });
    factory.register_entity(EntityKey::from("signal-passthrough"), || {
        Box::<SignalPassthroughController>::default()
    });
    factory.register_entity(EntityKey::from("signal-amplitude-passthrough"), || {
        Box::new(SignalPassthroughController::new_amplitude_passthrough_type())
    });
    factory.register_entity(
        EntityKey::from("signal-amplitude-inverted-passthrough"),
        || Box::new(SignalPassthroughController::new_amplitude_inverted_passthrough_type()),
    );
    factory.register_entity(EntityKey::from("timer"), || {
        Box::new(Timer::new_with(MusicalTime::DURATION_QUARTER))
    });
    factory.register_entity(EntityKey::from("toy-controller"), || {
        Box::<ToyController>::default()
    });
    factory.register_entity(EntityKey::from("toy-controller-noisy"), || {
        Box::new(TestControllerAlwaysSendsMidiMessage::default())
    });

    // Effects
    factory.register_entity(EntityKey::from("bitcrusher"), || {
        Box::new(Bitcrusher::default())
    });
    factory.register_entity(EntityKey::from("compressor"), || {
        Box::new(Compressor::default())
    });
    factory.register_entity(EntityKey::from("filter-low-pass-24db"), || {
        Box::new(BiQuadFilterLowPass24db::new_with(
            &BiQuadFilterLowPass24dbParams::default(),
        ))
    });
    factory.register_entity(EntityKey::from("gain"), || {
        Box::new(Gain::new_with(&GainParams {
            ceiling: Normal::from(0.5),
        }))
    });
    factory.register_entity(EntityKey::from("limiter"), || Box::new(Limiter::default()));
    factory.register_entity(EntityKey::from("mixer"), || Box::new(Mixer::default()));
    // TODO: this is lazy. It's too hard right now to adjust parameters within
    // code, so I'm creating a special instrument with the parameters I want.
    factory.register_entity(EntityKey::from("mute"), || {
        Box::new(Gain::new_with(&GainParams {
            ceiling: Normal::minimum(),
        }))
    });
    factory.register_entity(EntityKey::from("reverb"), || {
        Box::new(Reverb::new_with(&ReverbParams {
            attenuation: Normal::from(0.8),
            seconds: 1.0,
        }))
    });
    factory.register_entity(EntityKey::from("toy-effect"), || {
        Box::<ToyEffect>::default()
    });

    // Instruments
    factory.register_entity(EntityKey::from("toy-synth"), || {
        Box::new(ToySynth::new_with(&ToySynthParams::default()))
    });
    factory.register_entity(EntityKey::from("toy-instrument"), || {
        Box::<ToyInstrument>::default()
    });
    factory.register_entity(EntityKey::from("drumkit"), || {
        Box::new(Drumkit::new_with(
            &DrumkitParams::default(),
            &Paths::default(),
        ))
    });
    factory.register_entity(EntityKey::from("fm-synth"), || {
        Box::new(FmSynth::new_with(&FmSynthParams::default()))
    });
    factory.register_entity(EntityKey::from("sampler"), || {
        Box::new(Sampler::new_with(
            &SamplerParams {
                filename: "stereo-pluck.wav".to_string(),
                root: 0.0.into(),
            },
            &Paths::default(),
        ))
    });
    factory.register_entity(EntityKey::from("welsh-synth"), || {
        Box::new(WelshSynth::new_with(&WelshSynthParams::default()))
    });

    factory.complete_registration();

    factory
}

/// A globally unique identifier for a kind of entity, such as an arpeggiator
/// controller, an FM synthesizer, or a reverb effect.
#[derive(Clone, Debug, Display, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct EntityKey(String);
impl From<&String> for EntityKey {
    fn from(value: &String) -> Self {
        EntityKey(value.to_string())
    }
}
impl From<&str> for EntityKey {
    fn from(value: &str) -> Self {
        EntityKey(value.to_string())
    }
}

type EntityFactoryFn = fn() -> Box<dyn Entity>;

/// The one and only EntityFactory. Access it with `EntityFactory::global()`.
static FACTORY: OnceCell<EntityFactory> = OnceCell::new();

/// [EntityFactory] accepts [Key]s and creates instruments, controllers, and
/// effects. It makes sure every entity has a proper [Uid].
#[derive(Debug)]
pub struct EntityFactory {
    next_uid: RelaxedCounter,
    entities: HashMap<EntityKey, EntityFactoryFn>,
    keys: HashSet<EntityKey>,

    is_registration_complete: bool,
    sorted_keys: Vec<EntityKey>,
}
impl Default for EntityFactory {
    fn default() -> Self {
        Self {
            next_uid: RelaxedCounter::new(Self::MAX_RESERVED_UID + 1),
            entities: Default::default(),
            keys: Default::default(),
            is_registration_complete: Default::default(),
            sorted_keys: Default::default(),
        }
    }
}
impl EntityFactory {
    /// Specifies the range of [Uid]s that [EntityFactory] will never issue.
    pub const MAX_RESERVED_UID: usize = 1023;

    /// Provides the one and only [EntityFactory].
    pub fn global() -> &'static Self {
        FACTORY
            .get()
            .expect("EntityFactory has not been initialized")
    }

    /// Set the next [Uid]. This is needed if we're deserializing a project and
    /// need to reset the [EntityFactory] to mint unique [Uid]s.
    ///
    /// Note that the specified [Uid] is not necessarily the next one that will
    /// be issued; we guarantee only that subsequent [Uid]s won't be lower than
    /// it. This is because we're using [RelaxedCounter] under the hood to allow
    /// entirely immutable usage of this factory after creation and
    /// configuration.
    pub fn set_next_uid(&self, next_uid_value: usize) {
        self.next_uid.reset();
        self.next_uid
            .add(next_uid_value.max(Self::MAX_RESERVED_UID + 1));
    }

    /// Registers a new type for the given [Key] using the given closure.
    pub fn register_entity(&mut self, key: EntityKey, f: EntityFactoryFn) {
        if self.is_registration_complete {
            panic!("attempt to register an entity after registration completed");
        }
        if self.keys.insert(key.clone()) {
            self.entities.insert(key, f);
        } else {
            panic!("register_entity({}): duplicate key. Exiting.", key);
        }
    }

    /// Tells the factory that we won't be registering any more entities,
    /// allowing it to do some final housekeeping.
    pub fn complete_registration(&mut self) {
        self.is_registration_complete = true;
        self.sorted_keys = self.keys().iter().cloned().collect();
        self.sorted_keys.sort();
    }

    /// Creates a new entity of the type corresponding to the given [Key].
    pub fn new_entity(&self, key: &EntityKey) -> Option<Box<dyn Entity>> {
        if let Some(f) = self.entities.get(key) {
            let mut r = f();
            r.set_uid(self.mint_uid());
            Some(r)
        } else {
            None
        }
    }

    /// Returns the [HashSet] of all [Key]s.
    pub fn keys(&self) -> &HashSet<EntityKey> {
        &self.keys
    }

    /// Returns the [HashMap] for all [Key] and entity pairs.
    pub fn entities(&self) -> &HashMap<EntityKey, EntityFactoryFn> {
        &self.entities
    }

    /// Returns a [Uid] that is guaranteed to be unique among all [Uid]s minted
    /// by this factory. This method is exposed if someone wants to create an
    /// entity outside this factory, but still refer to it by [Uid]. An example
    /// is [super::Transport], which is an entity that [super::Orchestrator]
    /// treats specially.
    pub fn mint_uid(&self) -> Uid {
        Uid(self.next_uid.inc())
    }

    /// Returns all the [Key]s in sorted order for consistent display in the UI.
    pub fn sorted_keys(&self) -> &[EntityKey] {
        if !self.is_registration_complete {
            panic!("sorted_keys() can be called only after registration is complete.")
        }
        &self.sorted_keys
    }

    /// Sets the singleton [EntityFactory].
    pub fn initialize(entity_factory: Self) -> Result<(), Self> {
        FACTORY.set(entity_factory)
    }
}

/// An [EntityStore] owns [Entities](Entity). It implements some [Entity]
/// traits, such as [Configurable], and fans out usage of those traits to the
/// owned entities, making it easier for the owner of an [EntityStore] to treat
/// all its entities as a single [Entity].
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EntityStore {
    #[serde(skip)]
    sample_rate: SampleRate,
    entities: HashMap<Uid, Box<dyn Entity>>,
}
impl EntityStore {
    /// Adds an [Entity] to the store.
    pub fn add(&mut self, mut entity: Box<dyn Entity>) -> anyhow::Result<Uid> {
        let uid = entity.uid();
        if self.entities.contains_key(&uid) {
            return Err(anyhow!("Entity Uid {uid} already exists"));
        }
        entity.update_sample_rate(self.sample_rate);
        self.entities.insert(entity.uid(), entity);
        Ok(uid)
    }
    /// Returns the specified [Entity].
    pub fn get(&self, uid: &Uid) -> Option<&Box<dyn Entity>> {
        self.entities.get(uid)
    }
    /// Returns the specified mutable [Entity].
    pub fn get_mut(&mut self, uid: &Uid) -> Option<&mut Box<dyn Entity>> {
        self.entities.get_mut(uid)
    }
    /// Removes the specified [Entity] from the store, returning it (and thus
    /// ownership of it) to the caller.
    pub fn remove(&mut self, uid: &Uid) -> Option<Box<dyn Entity>> {
        self.entities.remove(uid)
    }
    /// Returns all the [Uid]s of owned [Entities](Entity). Order is undefined
    /// and may change frequently.
    pub fn uids(&self) -> hash_map::Keys<'_, Uid, Box<dyn Entity>> {
        self.entities.keys()
    }
    /// Returns all owned [Entities](Entity) as an iterator. Order is undefined.
    pub fn iter(&self) -> hash_map::Values<'_, Uid, Box<dyn Entity>> {
        self.entities.values()
    }
    /// Returns all owned [Entities](Entity) as an iterator (mutable). Order is
    /// undefined.
    pub fn iter_mut(&mut self) -> hash_map::ValuesMut<'_, Uid, Box<dyn Entity>> {
        self.entities.values_mut()
    }
    #[allow(missing_docs)]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Calculates the highest [Uid] of owned [Entities](Entity). This is used
    /// when after deserializing to make sure that newly generated [Uid]s don't
    /// collide with existing ones.
    pub fn calculate_max_entity_uid(&self) -> Option<Uid> {
        // TODO: keep an eye on this in case it gets expensive. It's currently
        // used only after loading from disk, and it's O(number of things in
        // system), so it's unlikely to matter.
        self.entities.keys().max().copied()
    }
}
impl Ticks for EntityStore {
    fn tick(&mut self, tick_count: usize) {
        self.iter_mut().for_each(|t| {
            if let Some(t) = t.as_instrument_mut() {
                t.tick(tick_count)
            }
        });
    }
}
impl Configurable for EntityStore {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.iter_mut().for_each(|t| {
            t.update_sample_rate(sample_rate);
        });
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.iter_mut().for_each(|t| {
            t.update_tempo(tempo);
        });
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.iter_mut().for_each(|t| {
            t.update_time_signature(time_signature);
        });
    }
}
impl Controls for EntityStore {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {
        self.iter_mut().for_each(|t| {
            if let Some(t) = t.as_controller_mut() {
                t.update_time(range);
            }
        });
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.iter_mut().for_each(|entity| {
            if let Some(e) = entity.as_controller_mut() {
                let tuid = e.uid();
                e.work(&mut |claimed_uid, message| {
                    control_events_fn(tuid, message);
                    if tuid != claimed_uid {
                        eprintln!("Warning: entity {tuid} is sending control messages with incorrect uid {claimed_uid}");
                    }
                });
            }
        });
    }

    fn is_finished(&self) -> bool {
        self.iter().all(|t| {
            if let Some(t) = t.as_controller() {
                t.is_finished()
            } else {
                true
            }
        })
    }

    fn play(&mut self) {
        // TODO: measure whether it's faster to speed through everything and
        // check type than to look up each UID in self.controllers
        self.iter_mut().for_each(|t| {
            if let Some(t) = t.as_controller_mut() {
                t.play();
            }
        });
    }

    fn stop(&mut self) {
        self.iter_mut().for_each(|t| {
            if let Some(t) = t.as_controller_mut() {
                t.stop();
            }
        });
    }

    fn skip_to_start(&mut self) {
        self.iter_mut().for_each(|t| {
            if let Some(t) = t.as_controller_mut() {
                t.skip_to_start();
            }
        });
    }

    fn is_performing(&self) -> bool {
        self.iter().any(|t| {
            if let Some(t) = t.as_controller() {
                t.is_performing()
            } else {
                true
            }
        })
    }
}
impl Serializable for EntityStore {
    fn after_deser(&mut self) {
        self.entities.iter_mut().for_each(|(_, t)| t.after_deser());
    }
}

pub mod test_entities {
    use crate::{midi::prelude::*, prelude::*, traits::prelude::*};
    use ensnare_proc_macros::{Control, IsController, IsEffect, IsInstrument, Params, Uid};
    use serde::{Deserialize, Serialize};
    use std::sync::{Arc, Mutex};

    /// The smallest possible [IsController].
    #[derive(Debug, Default, IsController, Serialize, Deserialize, Uid)]
    pub struct TestController {
        uid: Uid,
    }
    impl Displays for TestController {}
    impl HandlesMidi for TestController {}
    impl Controls for TestController {}
    impl Configurable for TestController {}
    impl Serializable for TestController {}

    /// The smallest possible [IsInstrument].
    #[derive(Debug, Default, IsInstrument, Serialize, Deserialize, Uid)]
    pub struct TestInstrument {
        uid: Uid,
        sample_rate: SampleRate,
    }
    impl Displays for TestInstrument {}
    impl HandlesMidi for TestInstrument {}
    impl Controllable for TestInstrument {}
    impl Configurable for TestInstrument {
        fn sample_rate(&self) -> SampleRate {
            self.sample_rate
        }

        fn update_sample_rate(&mut self, sample_rate: SampleRate) {
            self.sample_rate = sample_rate;
        }
    }
    impl Serializable for TestInstrument {}
    impl Generates<StereoSample> for TestInstrument {
        fn value(&self) -> StereoSample {
            StereoSample::default()
        }

        fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
            values.fill(StereoSample::default())
        }
    }
    impl Ticks for TestInstrument {}

    /// Produces a constant audio signal. Used for ensuring that a known signal
    /// value gets all the way through the pipeline.
    #[derive(Debug, Default, Control, IsInstrument, Params, Uid, Serialize, Deserialize)]
    pub struct TestAudioSource {
        uid: Uid,

        // This should be a Normal, but we use this audio source for testing edge
        // conditions. Thus we need to let it go out of range.
        #[control]
        #[params]
        level: ParameterType,

        #[serde(skip)]
        sample_rate: SampleRate,
    }
    impl Serializable for TestAudioSource {}
    impl Generates<StereoSample> for TestAudioSource {
        fn value(&self) -> StereoSample {
            StereoSample::from(self.level)
        }

        fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
            values.fill(self.value());
        }
    }
    impl Configurable for TestAudioSource {
        fn sample_rate(&self) -> SampleRate {
            self.sample_rate
        }

        fn update_sample_rate(&mut self, sample_rate: SampleRate) {
            self.sample_rate = sample_rate;
        }
    }
    impl Ticks for TestAudioSource {}
    impl HandlesMidi for TestAudioSource {}
    #[allow(dead_code)]
    impl TestAudioSource {
        pub const TOO_LOUD: SampleType = 1.1;
        pub const LOUD: SampleType = 1.0;
        pub const MEDIUM: SampleType = 0.5;
        pub const SILENT: SampleType = 0.0;
        pub const QUIET: SampleType = -1.0;
        pub const TOO_QUIET: SampleType = -1.1;

        pub fn new_with(params: &TestAudioSourceParams) -> Self {
            Self {
                level: params.level(),
                ..Default::default()
            }
        }

        pub fn level(&self) -> f64 {
            self.level
        }

        pub fn set_level(&mut self, level: ParameterType) {
            self.level = level;
        }
    }
    impl Displays for TestAudioSource {}

    /// The smallest possible [IsEffect].
    #[derive(Debug, Default, IsEffect, Serialize, Deserialize, Uid)]
    pub struct TestEffect {
        uid: Uid,
    }
    impl Displays for TestEffect {}
    impl Configurable for TestEffect {}
    impl Controllable for TestEffect {}
    impl Serializable for TestEffect {}
    impl TransformsAudio for TestEffect {}

    /// An effect that negates the input.
    #[derive(Debug, Default, IsEffect, Serialize, Deserialize, Uid)]
    pub struct TestEffectNegatesInput {
        uid: Uid,
    }
    impl Displays for TestEffectNegatesInput {}
    impl Configurable for TestEffectNegatesInput {}
    impl Controllable for TestEffectNegatesInput {}
    impl Serializable for TestEffectNegatesInput {}
    impl TransformsAudio for TestEffectNegatesInput {
        fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
            -input_sample
        }
    }

    #[derive(Debug, Default, Uid, IsController, Serialize, Deserialize)]
    pub struct TestControllerAlwaysSendsMidiMessage {
        uid: Uid,

        #[serde(skip)]
        midi_note: u8,

        #[serde(skip)]
        is_performing: bool,
    }
    impl Displays for TestControllerAlwaysSendsMidiMessage {}
    impl HandlesMidi for TestControllerAlwaysSendsMidiMessage {}
    impl Controls for TestControllerAlwaysSendsMidiMessage {
        fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
            if self.is_performing {
                control_events_fn(
                    self.uid,
                    EntityEvent::Midi(
                        MidiChannel(0),
                        MidiMessage::NoteOn {
                            key: u7::from(self.midi_note),
                            vel: u7::from(127),
                        },
                    ),
                );
                self.midi_note += 1;
                if self.midi_note > 127 {
                    self.midi_note = 1;
                }
            }
        }

        fn is_finished(&self) -> bool {
            false
        }

        fn play(&mut self) {
            self.is_performing = true;
        }

        fn stop(&mut self) {
            self.is_performing = false;
        }

        fn is_performing(&self) -> bool {
            self.is_performing
        }
    }
    impl Configurable for TestControllerAlwaysSendsMidiMessage {}
    impl Serializable for TestControllerAlwaysSendsMidiMessage {}

    /// An [IsInstrument](ensnare::traits::IsInstrument) that counts how many MIDI messages it has received.
    #[derive(Debug, Default, IsInstrument, Uid, Serialize, Deserialize)]
    pub struct TestInstrumentCountsMidiMessages {
        uid: Uid,
        pub received_midi_message_count: Arc<Mutex<usize>>,
    }
    impl Generates<StereoSample> for TestInstrumentCountsMidiMessages {
        fn value(&self) -> StereoSample {
            StereoSample::default()
        }

        fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
            values.fill(StereoSample::default())
        }
    }
    impl Configurable for TestInstrumentCountsMidiMessages {}
    impl Controllable for TestInstrumentCountsMidiMessages {}
    impl Ticks for TestInstrumentCountsMidiMessages {}
    impl HandlesMidi for TestInstrumentCountsMidiMessages {
        fn handle_midi_message(
            &mut self,
            _action: MidiChannel,
            _: MidiMessage,
            _: &mut MidiMessagesFn,
        ) {
            if let Ok(mut received_count) = self.received_midi_message_count.lock() {
                *received_count += 1;
            }
        }
    }
    impl Serializable for TestInstrumentCountsMidiMessages {}
    impl TestInstrumentCountsMidiMessages {
        pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
            &self.received_midi_message_count
        }
    }
    impl Displays for TestInstrumentCountsMidiMessages {}
}

#[cfg(test)]
mod tests {
    use super::test_entities::{TestController, TestEffect, TestInstrument};
    use super::EntityStore;
    use crate::entities::{EntityFactory, EntityKey};
    use crate::{prelude::*, traits::prelude::*};
    use std::collections::HashSet;

    /// Registers all [EntityFactory]'s entities. Note that the function returns an
    /// &EntityFactory. This encourages usage like this:
    ///
    /// ```
    /// let mut factory = EntityFactory::default();
    /// let factory = register_test_factory_entities(&mut factory);
    /// ```
    ///
    /// This makes the factory immutable once it's set up.
    #[must_use]
    pub fn register_test_factory_entities(mut factory: EntityFactory) -> EntityFactory {
        factory.register_entity(EntityKey::from("instrument"), || {
            Box::new(TestInstrument::default())
        });
        factory.register_entity(EntityKey::from("controller"), || {
            Box::new(TestController::default())
        });
        factory.register_entity(
            EntityKey::from("effect"),
            || Box::new(TestEffect::default()),
        );

        factory.complete_registration();

        factory
    }

    #[test]
    fn entity_creation() {
        assert!(
            EntityFactory::default().entities().is_empty(),
            "A new EntityFactory should be empty"
        );

        let factory = register_test_factory_entities(EntityFactory::default());
        assert!(
            !factory.entities().is_empty(),
            "after registering test entities, factory should contain at least one"
        );

        // After registration, rebind as immutable
        let factory = factory;

        assert!(factory.new_entity(&EntityKey::from(".9-#$%)@#)")).is_none());

        let mut ids: HashSet<Uid> = HashSet::default();
        for key in factory.keys().iter() {
            let e = factory.new_entity(key);
            assert!(e.is_some());
            if let Some(e) = e {
                assert!(!e.name().is_empty());
                assert!(!ids.contains(&e.uid()));
                ids.insert(e.uid());
                assert!(
                    e.as_controller().is_some()
                        || e.as_instrument().is_some()
                        || e.as_effect().is_some(),
                    "Entity '{}' is missing its entity type",
                    key
                );
            }
        }
    }

    #[test]
    fn entity_factory_uid_uniqueness() {
        let ef = EntityFactory::default();
        let uid = ef.mint_uid();

        let ef = EntityFactory::default();
        let uid2 = ef.mint_uid();

        assert_eq!(uid, uid2);

        let ef = EntityFactory::default();
        ef.set_next_uid(uid.0 + 1);
        let uid2 = ef.mint_uid();
        assert_ne!(uid, uid2);
    }

    #[test]
    fn store_is_responsible_for_sample_rate() {
        let mut t = EntityStore::default();
        assert_eq!(t.sample_rate, SampleRate::DEFAULT);
        t.update_sample_rate(SampleRate(44444));
        let factory = register_test_factory_entities(EntityFactory::default());

        let entity = factory.new_entity(&EntityKey::from("instrument")).unwrap();
        assert_eq!(
            entity.sample_rate(),
            SampleRate::DEFAULT,
            "before adding to store, sample rate should be untouched"
        );

        let uid = t.add(entity).unwrap();
        let entity = t.remove(&uid).unwrap();
        assert_eq!(
            entity.sample_rate(),
            SampleRate(44444),
            "after adding/removing to/from store, sample rate should match"
        );
    }

    #[test]
    fn disallow_duplicate_uids() {
        let mut t = EntityStore::default();
        assert_eq!(t.calculate_max_entity_uid(), None);

        let mut one = Box::new(TestInstrument::default());
        one.set_uid(Uid(9999));
        assert!(t.add(one).is_ok(), "adding a unique UID should succeed");
        assert_eq!(t.calculate_max_entity_uid(), Some(Uid(9999)));

        let mut two = Box::new(TestInstrument::default());
        two.set_uid(Uid(9999));
        assert!(t.add(two).is_err(), "adding a duplicate UID should fail");

        let max_uid = t.calculate_max_entity_uid().unwrap();
        // Though the add() was sure to fail, it's still considered mutably
        // borrowed at compile time.
        let mut two = Box::new(TestInstrument::default());
        two.set_uid(Uid(max_uid.0 + 1));
        assert!(
            t.add(two).is_ok(),
            "using Orchestrator's max_entity_uid as a guide should work."
        );
        assert_eq!(t.calculate_max_entity_uid(), Some(Uid(10000)));
    }
}
