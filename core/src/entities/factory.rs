// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    controllers::{ControlTrip, Timer, Trigger},
    entities::prelude::*,
    generators::{EnvelopeParams, Waveform},
    midi::MidiChannel,
    modulators::DcaParams,
    prelude::*,
    traits::prelude::*,
    uid::Uid,
    utils::Paths,
};
use anyhow::anyhow;
use derive_more::Display;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, HashMap, HashSet},
    option::Option,
};

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
    factory.register_entity_with_str_key(Arpeggiator::ENTITY_KEY, |_uid| {
        Box::new(Arpeggiator::new_with(
            &ArpeggiatorParams::default(),
            MidiChannel::default(),
        ))
    });
    factory.register_entity_with_str_key(ControlTrip::ENTITY_KEY, |_uid| {
        Box::<ControlTrip>::default()
    });
    factory.register_entity_with_str_key(LfoController::ENTITY_KEY, |_uid| {
        Box::new(LfoController::new_with(&LfoControllerParams {
            frequency: FrequencyHz::from(0.2),
            waveform: Waveform::Sawtooth,
        }))
    });
    factory.register_entity_with_str_key(SignalPassthroughController::ENTITY_KEY, |_uid| {
        Box::<SignalPassthroughController>::default()
    });
    factory.register_entity_with_str_key("signal-amplitude-passthrough", |_uid| {
        Box::new(SignalPassthroughController::new_amplitude_passthrough_type())
    });
    factory.register_entity_with_str_key("signal-amplitude-inverted-passthrough", |_uid| {
        Box::new(SignalPassthroughController::new_amplitude_inverted_passthrough_type())
    });
    factory.register_entity_with_str_key(Timer::ENTITY_KEY, |_uid| {
        Box::new(Timer::new_with(MusicalTime::DURATION_QUARTER))
    });
    factory.register_entity_with_str_key(Trigger::ENTITY_KEY, |_uid| {
        Box::new(Trigger::new_with(
            Timer::new_with(MusicalTime::DURATION_QUARTER),
            ControlValue(1.0),
        ))
    });
    factory.register_entity_with_str_key(ToyController::ENTITY_KEY, |_uid| {
        Box::<ToyController>::default()
    });
    factory.register_entity_with_str_key("toy-controller-noisy", |_uid| {
        Box::<ToyControllerAlwaysSendsMidiMessage>::default()
    });

    // Effects
    factory
        .register_entity_with_str_key(Bitcrusher::ENTITY_KEY, |_uid| Box::<Bitcrusher>::default());
    factory
        .register_entity_with_str_key(Compressor::ENTITY_KEY, |_uid| Box::<Compressor>::default());
    factory.register_entity_with_str_key("filter-low-pass-24db", |_uid| {
        Box::new(BiQuadFilterLowPass24db::new_with(
            &BiQuadFilterLowPass24dbParams::default(),
        ))
    });
    factory.register_entity_with_str_key(Gain::ENTITY_KEY, |_uid| {
        Box::new(Gain::new_with(&GainParams {
            ceiling: Normal::from(0.5),
        }))
    });
    factory.register_entity_with_str_key(Limiter::ENTITY_KEY, |_uid| Box::<Limiter>::default());
    factory.register_entity_with_str_key(Mixer::ENTITY_KEY, |_uid| Box::<Mixer>::default());
    // TODO: this is lazy. It's too hard right now to adjust parameters within
    // code, so I'm creating a special instrument with the parameters I want.
    factory.register_entity_with_str_key("mute", |_uid| {
        Box::new(Gain::new_with(&GainParams {
            ceiling: Normal::minimum(),
        }))
    });
    factory.register_entity_with_str_key(Reverb::ENTITY_KEY, |_uid| {
        Box::new(Reverb::new_with(&ReverbParams {
            attenuation: Normal::from(0.8),
            seconds: 1.0,
        }))
    });
    factory.register_entity_with_str_key(ToyEffect::ENTITY_KEY, |_uid| Box::<ToyEffect>::default());

    // Instruments
    factory.register_entity_with_str_key(ToySynth::ENTITY_KEY, |uid| {
        Box::new(ToySynth::new_with(
            uid,
            &ToySynthParams {
                voice_count: Default::default(),
                waveform: Default::default(),
                envelope: EnvelopeParams::safe_default(),
                dca: Default::default(),
            },
        ))
    });
    factory.register_entity_with_str_key(ToyInstrument::ENTITY_KEY, |_uid| {
        Box::<ToyInstrument>::default()
    });
    factory.register_entity_with_str_key(Drumkit::ENTITY_KEY, |_uid| {
        Box::new(Drumkit::new_with(
            &DrumkitParams::default(),
            &Paths::default(),
        ))
    });
    factory.register_entity_with_str_key(FmSynth::ENTITY_KEY, |_uid| {
        // A crisp, classic FM sound that brings me back to 1985.
        Box::new(FmSynth::new_with(&FmSynthParams {
            depth: 1.0.into(),
            ratio: 16.0.into(),
            beta: 10.0.into(),
            carrier_envelope: EnvelopeParams::safe_default(),
            modulator_envelope: EnvelopeParams::default(),
            dca: DcaParams::default(),
        }))
    });
    factory.register_entity_with_str_key(Sampler::ENTITY_KEY, |_uid| {
        let mut sampler = Sampler::new_with(&SamplerParams {
            filename: "stereo-pluck.wav".to_string(),
            root: 0.0.into(),
        });
        let _ = sampler.load(&Paths::default()); // TODO: we're ignoring the error
        Box::new(sampler)
    });
    factory.register_entity_with_str_key(WelshSynth::ENTITY_KEY, |uid| {
        Box::new(WelshSynth::new_with(uid, &WelshSynthParams::default()))
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
impl From<String> for EntityKey {
    fn from(value: String) -> Self {
        EntityKey(value)
    }
}

type EntityFactoryFn = fn(Uid) -> Box<dyn Entity>;

/// The one and only EntityFactory. Access it with `EntityFactory::global()`.
static FACTORY: OnceCell<EntityFactory> = OnceCell::new();

/// [EntityFactory] accepts [Key]s and creates instruments, controllers, and
/// effects. It makes sure every entity has a proper [Uid].
#[derive(Debug)]
pub struct EntityFactory {
    entities: HashMap<EntityKey, EntityFactoryFn>,
    keys: HashSet<EntityKey>,

    is_registration_complete: bool,
    sorted_keys: Vec<EntityKey>,
}
impl Default for EntityFactory {
    fn default() -> Self {
        Self {
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

    /// Registers a new type for the given [Key] using the given closure.
    pub fn register_entity(&mut self, key: EntityKey, f: EntityFactoryFn) {
        if self.is_registration_complete {
            panic!("attempt to register an entity after registration completed");
        }
        if self.keys.insert(key.clone()) {
            self.entities.insert(key, f);
        } else {
            panic!("register_entity({key}): duplicate key. Exiting.");
        }
    }

    /// Registers a new type for the given [Key] using the given closure, but
    /// takes a &str and creates the [Key] from it.
    pub fn register_entity_with_str_key(&mut self, key: &str, f: EntityFactoryFn) {
        self.register_entity(EntityKey::from(key), f)
    }

    /// Tells the factory that we won't be registering any more entities,
    /// allowing it to do some final housekeeping.
    pub fn complete_registration(&mut self) {
        self.is_registration_complete = true;
        self.sorted_keys = self.keys().iter().cloned().collect();
        self.sorted_keys.sort();
    }

    /// Creates a new entity of the type corresponding to the given [Key].
    pub fn new_entity(&self, key: &EntityKey, uid: Uid) -> Option<Box<dyn Entity>> {
        if let Some(f) = self.entities.get(key) {
            let mut entity = f(uid);
            entity.set_uid(uid);
            Some(entity)
        } else {
            eprintln!("WARNING: {key} for uid {uid} produced no entity");
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
    pub fn add(&mut self, mut entity: Box<dyn Entity>, uid: Uid) -> anyhow::Result<()> {
        if uid.0 == 0 {
            return Err(anyhow!("Entity Uid zero is invalid"));
        }
        if self.entities.contains_key(&uid) {
            return Err(anyhow!("Entity Uid {uid} already exists"));
        }
        entity.update_sample_rate(self.sample_rate);
        self.entities.insert(uid, entity);
        Ok(())
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
    fn update_time(&mut self, range: &ViewRange) {
        self.iter_mut().for_each(|t| {
            if let Some(t) = t.as_controller_mut() {
                t.update_time(range);
            }
        });
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.entities.iter_mut().for_each(|(uid, entity)| {
            if let Some(e) = entity.as_controller_mut() {
                e.work(&mut |claimed_uid, message| {
                    //    debug_assert!(claimed_uid.is_none(), "Entities controlled by EntityStore should not know or claim to know their own Uid");

                    // Here is where we substitute the known Uid of the Entity
                    // that we just called for the None that should have been
                    // passed to us. Past this point in the control_events_fn
                    // chain, we can rely on the uid argument being Some().
                    //
                    // Why does this work?
                    //
                    // The normal case is that EntityStore knows an Entity's
                    // Uid, so this is where EntityStore does its job of filling
                    // in the Uid.
                    //
                    // The odd case is ControlAtlas, which owns a bunch of
                    // ControlTrips, each having its own Uid, that generate
                    // control events. When we call ControlAtlas::work(), we
                    // thus expect it to have filled in control_events_fn's uid
                    // parameter. But at the moment (and unfortunately this has
                    // been changing a lot lately), Track owns ControlAtlas, so
                    // we know that we (EntityStore) won't be the one calling
                    // ControlAtlas::work(). So it's an odd case, but it's also
                    // inapplicable to this block of code.
                    control_events_fn(claimed_uid.or(Some(*uid)), message);
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
    use crate::{
        generators::{Envelope, EnvelopeParams, Oscillator, OscillatorParams, Waveform},
        midi::prelude::*,
        prelude::*,
        traits::{prelude::*, GeneratesEnvelope, MidiMessagesFn},
    };
    use ensnare_proc_macros::{Control, IsController, IsEffect, IsInstrument, Metadata, Params};
    use serde::{Deserialize, Serialize};
    use std::sync::{Arc, Mutex};

    /// The smallest possible [IsController].
    #[derive(Debug, Default, IsController, Serialize, Deserialize, Metadata)]
    pub struct TestController {
        uid: Uid,
    }
    impl Displays for TestController {}
    impl HandlesMidi for TestController {}
    impl Controls for TestController {}
    impl Configurable for TestController {}
    impl Serializable for TestController {}

    /// The smallest possible [IsInstrument].
    #[derive(Debug, Default, IsInstrument, Serialize, Deserialize, Metadata)]
    pub struct TestInstrument {
        pub uid: Uid,
        pub sample_rate: SampleRate,
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
    #[derive(Debug, Default, Control, IsInstrument, Params, Metadata, Serialize, Deserialize)]
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
    #[derive(Debug, Default, IsEffect, Serialize, Deserialize, Metadata)]
    pub struct TestEffect {
        uid: Uid,
    }
    impl Displays for TestEffect {}
    impl Configurable for TestEffect {}
    impl Controllable for TestEffect {}
    impl Serializable for TestEffect {}
    impl TransformsAudio for TestEffect {}

    /// An effect that negates the input.
    #[derive(Debug, Default, IsEffect, Serialize, Deserialize, Metadata)]
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

    /// An [IsInstrument](ensnare::traits::IsInstrument) that counts how many MIDI messages it has received.
    #[derive(Debug, Default, IsInstrument, Metadata, Serialize, Deserialize)]
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

    #[derive(Debug, Default, IsInstrument, Metadata, Serialize, Deserialize)]
    pub struct TestControllable {
        uid: Uid,

        #[serde(skip)]
        tracker: Arc<std::sync::RwLock<Vec<(Uid, ControlIndex, ControlValue)>>>,
    }
    impl TestControllable {
        pub fn new_with(
            uid: Uid,
            tracker: Arc<std::sync::RwLock<Vec<(Uid, ControlIndex, ControlValue)>>>,
        ) -> Self {
            Self { uid, tracker }
        }
    }
    impl Controllable for TestControllable {
        fn control_set_param_by_index(&mut self, index: ControlIndex, value: ControlValue) {
            if let Ok(mut tracker) = self.tracker.write() {
                tracker.push((self.uid, index, value));
            }
        }
    }
    impl HandlesMidi for TestControllable {}
    impl Generates<StereoSample> for TestControllable {
        fn value(&self) -> StereoSample {
            StereoSample::SILENCE
        }

        fn generate_batch_values(&mut self, _values: &mut [StereoSample]) {
            todo!()
        }
    }
    impl Ticks for TestControllable {
        fn tick(&mut self, _tick_count: usize) {
            todo!()
        }
    }
    impl Serializable for TestControllable {}
    impl Configurable for TestControllable {}
    impl Displays for TestControllable {}

    #[derive(Debug)]
    pub struct TestVoice {
        sample_rate: SampleRate,
        oscillator: Oscillator,
        envelope: Envelope,

        sample: StereoSample,

        note_on_key: u7,
        note_on_velocity: u7,
        steal_is_underway: bool,
    }
    impl IsStereoSampleVoice for TestVoice {}
    impl IsVoice<StereoSample> for TestVoice {}
    impl PlaysNotes for TestVoice {
        fn is_playing(&self) -> bool {
            !self.envelope.is_idle()
        }

        fn note_on(&mut self, key: u7, velocity: u7) {
            if self.is_playing() {
                self.steal_is_underway = true;
                self.note_on_key = key;
                self.note_on_velocity = velocity;
                self.envelope.trigger_shutdown();
            } else {
                self.set_frequency_hz(key.into());
                self.envelope.trigger_attack();
            }
        }

        fn aftertouch(&mut self, _velocity: u7) {
            todo!()
        }

        fn note_off(&mut self, _velocity: u7) {
            self.envelope.trigger_release();
        }
    }
    impl Generates<StereoSample> for TestVoice {
        fn value(&self) -> StereoSample {
            self.sample
        }

        fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
            for sample in values {
                self.tick(1);
                *sample = self.value();
            }
        }
    }
    impl Configurable for TestVoice {
        fn sample_rate(&self) -> SampleRate {
            self.sample_rate
        }

        fn update_sample_rate(&mut self, sample_rate: SampleRate) {
            self.sample_rate = sample_rate;
            self.oscillator.update_sample_rate(sample_rate);
            self.envelope.update_sample_rate(sample_rate);
        }
    }
    impl Ticks for TestVoice {
        fn tick(&mut self, tick_count: usize) {
            for _ in 0..tick_count {
                if self.is_playing() {
                    self.oscillator.tick(1);
                    self.envelope.tick(1);
                    if !self.is_playing() && self.steal_is_underway {
                        self.steal_is_underway = false;
                        self.note_on(self.note_on_key, self.note_on_velocity);
                    }
                }
            }
            self.sample = if self.is_playing() {
                StereoSample::from(self.oscillator.value() * self.envelope.value())
            } else {
                StereoSample::SILENCE
            };
        }
    }

    impl TestVoice {
        pub(crate) fn new() -> Self {
            Self {
                sample_rate: Default::default(),
                oscillator: Oscillator::new_with(&OscillatorParams::default_with_waveform(
                    Waveform::Sine,
                )),
                envelope: Envelope::new_with(&EnvelopeParams {
                    attack: Normal::minimum(),
                    decay: Normal::minimum(),
                    sustain: Normal::maximum(),
                    release: Normal::minimum(),
                }),
                sample: Default::default(),
                note_on_key: Default::default(),
                note_on_velocity: Default::default(),
                steal_is_underway: Default::default(),
            }
        }
        fn set_frequency_hz(&mut self, frequency_hz: FrequencyHz) {
            self.oscillator.set_frequency(frequency_hz);
        }

        pub fn debug_is_shutting_down(&self) -> bool {
            self.envelope.debug_is_shutting_down()
        }

        pub fn debug_oscillator_frequency(&self) -> FrequencyHz {
            self.oscillator.frequency()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        test_entities::{TestController, TestEffect, TestInstrument},
        EntityStore,
    };
    use crate::{
        entities::{
            factory::register_factory_entities, prelude::*, toys::register_toy_factory_entities,
        },
        prelude::*,
        traits::prelude::*,
    };

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
        factory.register_entity_with_str_key(TestInstrument::ENTITY_KEY, |_uid| {
            Box::new(TestInstrument::default())
        });
        factory.register_entity_with_str_key(TestController::ENTITY_KEY, |_uid| {
            Box::new(TestController::default())
        });
        factory.register_entity_with_str_key(TestEffect::ENTITY_KEY, |_uid| {
            Box::new(TestEffect::default())
        });

        factory.complete_registration();

        factory
    }

    fn check_entity_factory(factory: EntityFactory) {
        assert!(factory
            .new_entity(&EntityKey::from(".9-#$%)@#)"), Uid::default())
            .is_none());

        for (uid, key) in factory.keys().iter().enumerate() {
            let uid = Uid(uid + 1000);
            let e = factory.new_entity(key, uid);
            assert!(e.is_some());
            if let Some(e) = e {
                assert!(!e.name().is_empty());
                assert_eq!(
                    e.uid(),
                    uid,
                    "Entity should remember the Uid given at creation"
                );
                assert!(
                    e.as_controller().is_some()
                        || e.as_instrument().is_some()
                        || e.as_effect().is_some(),
                    "Entity '{}' is missing its entity type",
                    key
                );
            } else {
                panic!("new_entity({key}) failed");
            }
        }
    }

    #[test]
    fn creation_of_test_entities() {
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

        check_entity_factory(factory);
    }

    #[test]
    fn creation_of_toy_entities() {
        assert!(
            EntityFactory::default().entities().is_empty(),
            "A new EntityFactory should be empty"
        );

        let factory = register_toy_factory_entities(EntityFactory::default());
        assert!(
            !factory.entities().is_empty(),
            "after registering toy entities, factory should contain at least one"
        );

        // After registration, rebind as immutable
        let factory = factory;

        check_entity_factory(factory);
    }

    // TODO: if we want to re-enable this, then we need to change
    // Sampler/Drumkit and anyone else to not load files when instantiated. This
    // might not be practical for those instruments.
    #[ignore = "This test requires Path hives to be set up properly, but they aren't on the CI machine."]
    #[test]
    fn creation_of_production_entities() {
        assert!(
            EntityFactory::default().entities().is_empty(),
            "A new EntityFactory should be empty"
        );

        let factory = register_factory_entities(EntityFactory::default());
        assert!(
            !factory.entities().is_empty(),
            "after registering entities, factory should contain at least one"
        );

        // After registration, rebind as immutable
        let factory = factory;

        check_entity_factory(factory);
    }

    #[test]
    fn store_is_responsible_for_sample_rate() {
        let mut t = EntityStore::default();
        assert_eq!(t.sample_rate, SampleRate::DEFAULT);
        t.update_sample_rate(SampleRate(44444));
        let factory = register_test_factory_entities(EntityFactory::default());

        let entity = factory
            .new_entity(&EntityKey::from(TestInstrument::ENTITY_KEY), Uid::default())
            .unwrap();
        assert_eq!(
            entity.sample_rate(),
            SampleRate::DEFAULT,
            "before adding to store, sample rate should be untouched"
        );

        let uid = Uid(234);
        assert!(t.add(entity, uid).is_ok());
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

        let uid_1 = Uid(9999);
        let one = Box::new(TestInstrument::default());
        assert!(
            t.add(one, uid_1).is_ok(),
            "adding a unique UID should succeed"
        );

        let two = Box::new(TestInstrument::default());
        assert!(
            t.add(two, uid_1).is_err(),
            "adding a duplicate UID should fail"
        );

        let uid_2 = Uid(10000);
        let two = Box::new(TestInstrument::default());
        assert!(
            t.add(two, uid_2).is_ok(),
            "Adding a second unique UID should succeed."
        );
    }
}
