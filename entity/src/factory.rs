// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    prelude::IsInstrument,
    traits::{Entity, EntityBounds, IsEffect},
};
use anyhow::anyhow;
use derive_more::Display;
use ensnare_core::{
    prelude::*,
    traits::{ControlProxyEventsFn, ControlsAsProxy},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, HashMap, HashSet},
    option::Option,
};

/// A globally unique identifier for a kind of entity, such as an arpeggiator
/// controller, an FM synthesizer, or a reverb effect.
#[derive(Clone, Debug, Display, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
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

pub type EntityFactoryFn<E> = fn(Uid) -> Box<E>;

// /// The one and only EntityFactory. Access it with `EntityFactory::global()`.
// static FACTORY: OnceCell<EntityFactory<dyn EntityBounds>> = OnceCell::new();
// impl EntityFactory<dyn EntityBounds> {
//     /// Provides the one and only [EntityFactory].
//     pub fn global() -> &'static Self {
//         FACTORY
//             .get()
//             .expect("EntityFactory has not been initialized")
//     }

//     /// Sets the singleton [EntityFactory].
//     pub fn initialize(mut entity_factory: Self) -> Result<(), Self> {
//         entity_factory.complete_registration();
//         FACTORY.set(entity_factory)
//     }
// }

/// [EntityFactory] accepts [EntityKey]s and creates instruments, controllers,
/// and effects. It makes sure every entity has a proper [Uid].
#[derive(Debug)]
pub struct EntityFactory<E: EntityBounds + ?Sized> {
    entities: HashMap<EntityKey, EntityFactoryFn<E>>,
    keys: HashSet<EntityKey>,

    is_registration_complete: bool,
    sorted_keys: Vec<EntityKey>,
}
impl<E: EntityBounds + ?Sized> Default for EntityFactory<E> {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            keys: Default::default(),
            is_registration_complete: Default::default(),
            sorted_keys: Default::default(),
        }
    }
}
impl<E: EntityBounds + ?Sized> EntityFactory<E> {
    /// Specifies the range of [Uid]s that [EntityFactory] will never issue.
    pub const MAX_RESERVED_UID: usize = 1023;

    /// Registers a new type for the given [EntityKey] using the given closure.
    pub fn register_entity(&mut self, key: EntityKey, f: EntityFactoryFn<E>) {
        if self.is_registration_complete {
            panic!("attempt to register an entity after registration completed");
        }
        if self.keys.insert(key.clone()) {
            self.entities.insert(key, f);
        } else {
            panic!("register_entity({key}): duplicate key. Exiting.");
        }
    }

    /// Registers a new type for the given [EntityKey] using the given closure,
    /// but takes a &str and creates the [EntityKey] from it.
    pub fn register_entity_with_str_key(&mut self, key: &str, f: EntityFactoryFn<E>) {
        self.register_entity(EntityKey::from(key), f)
    }

    /// Tells the factory that we won't be registering any more entities,
    /// allowing it to do some final housekeeping.
    pub fn finalize(mut self) -> Self {
        self.is_registration_complete = true;
        self.sorted_keys = self.keys().iter().cloned().collect();
        self.sorted_keys.sort();
        self
    }

    /// Creates a new entity of the type corresponding to the given [EntityKey]
    /// with the given [Uid].
    pub fn new_entity(&self, key: &EntityKey, uid: Uid) -> Option<Box<E>> {
        if let Some(f) = self.entities.get(key) {
            let mut entity = f(uid);
            entity.set_uid(uid);
            Some(entity)
        } else {
            eprintln!("WARNING: {key} for uid {uid} produced no entity");
            None
        }
    }

    /// Returns the [HashSet] of all [EntityKey]s.
    pub fn keys(&self) -> &HashSet<EntityKey> {
        &self.keys
    }

    /// Returns the [HashMap] for all [EntityKey] and entity pairs.
    pub fn entities(&self) -> &HashMap<EntityKey, EntityFactoryFn<E>> {
        &self.entities
    }

    /// Returns all the [EntityKey]s in sorted order for consistent display in
    /// the UI.
    pub fn sorted_keys(&self) -> &[EntityKey] {
        if !self.is_registration_complete {
            panic!("sorted_keys() can be called only after registration is complete.")
        }
        &self.sorted_keys
    }
}

/// An [EntityStore] owns [Entities](Entity). It implements some [Entity]
/// traits, such as [Configurable], and fans out usage of those traits to the
/// owned entities, making it easier for the owner of an [EntityStore] to treat
/// all its entities as a single [Entity].
#[derive(Debug)]
pub struct EntityStore<E: Entity + ?Sized> {
    sample_rate: SampleRate,
    tempo: Tempo,
    entities: HashMap<Uid, Box<E>>,

    // We store our own copy of this value to make Controls::time_range() easier
    // to implement.
    time_range: TimeRange,
}
impl<E: Entity + ?Sized> Default for EntityStore<E> {
    fn default() -> Self {
        Self {
            sample_rate: Default::default(),
            tempo: Default::default(),
            entities: Default::default(),
            time_range: Default::default(),
        }
    }
}
impl<E: Entity + ?Sized> EntityStore<E> {
    /// Adds an [Entity] to the store.
    pub fn add(&mut self, mut entity: Box<E>, uid: Uid) -> anyhow::Result<()> {
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
    pub fn get(&self, uid: &Uid) -> Option<&Box<E>> {
        self.entities.get(uid)
    }
    /// Returns the specified mutable [Entity].
    pub fn get_mut(&mut self, uid: &Uid) -> Option<&mut Box<E>> {
        self.entities.get_mut(uid)
    }
    /// Removes the specified [Entity] from the store, returning it (and thus
    /// ownership of it) to the caller.
    pub fn remove(&mut self, uid: &Uid) -> Option<Box<E>> {
        self.entities.remove(uid)
    }
    /// Returns all the [Uid]s of owned [Entities](Entity). Order is undefined
    /// and may change frequently.
    pub fn uids(&self) -> hash_map::Keys<'_, Uid, Box<E>> {
        self.entities.keys()
    }
    /// Returns all owned [Entities](Entity) as an iterator. Order is undefined.
    pub fn iter(&self) -> hash_map::Values<'_, Uid, Box<E>> {
        self.entities.values()
    }
    /// Returns all owned [Entities](Entity) as an iterator (mutable). Order is
    /// undefined.
    pub fn iter_mut(&mut self) -> hash_map::ValuesMut<'_, Uid, Box<E>> {
        self.entities.values_mut()
    }
    #[allow(missing_docs)]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn as_controllable_mut(&mut self, uid: &Uid) -> Option<&mut dyn Controllable> {
        if let Some(e) = self.get_mut(uid) {
            e.as_controllable_mut()
        } else {
            None
        }
    }

    pub fn as_instrument_mut(&mut self, uid: &Uid) -> Option<&mut dyn IsInstrument> {
        if let Some(e) = self.get_mut(uid) {
            e.as_instrument_mut()
        } else {
            None
        }
    }

    pub fn as_effect_mut(&mut self, uid: &Uid) -> Option<&mut dyn IsEffect> {
        if let Some(e) = self.get_mut(uid) {
            e.as_effect_mut()
        } else {
            None
        }
    }
}
impl<E: Entity + ?Sized> Ticks for EntityStore<E> {
    fn tick(&mut self, tick_count: usize) {
        self.iter_mut().for_each(|t| {
            if let Some(t) = t.as_instrument_mut() {
                t.tick(tick_count)
            }
        });
    }
}
impl<E: Entity + ?Sized> Configurable for EntityStore<E> {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.iter_mut().for_each(|t| {
            t.update_sample_rate(sample_rate);
        });
        self.sample_rate = sample_rate;
    }

    fn tempo(&self) -> Tempo {
        self.tempo
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.iter_mut().for_each(|t| {
            t.update_tempo(tempo);
        });
        self.tempo = tempo;
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.iter_mut().for_each(|t| {
            t.update_time_signature(time_signature);
        });
    }
}
impl<E: Entity + ?Sized> Controls for EntityStore<E> {
    fn time_range(&self) -> Option<TimeRange> {
        if self.is_performing() {
            Some(self.time_range.clone())
        } else {
            None
        }
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.time_range = time_range.clone();
        self.iter_mut().for_each(|t| {
            if let Some(t) = t.as_controller_mut() {
                t.update_time_range(time_range);
            }
        });
    }

    fn work(&mut self, _: &mut ControlEventsFn) {
        unimplemented!("Look at ControlsAsProxy")
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
impl<E: Entity + ?Sized> ControlsAsProxy for EntityStore<E> {
    fn work_as_proxy(&mut self, control_events_fn: &mut ControlProxyEventsFn) {
        self.entities.iter_mut().for_each(|(uid, entity)| {
            if let Some(e) = entity.as_controller_mut() {
                e.work(&mut |message| {
                    control_events_fn(*uid, message);
                });
            }
        });
    }
}
impl<E: Entity + ?Sized> Serializable for EntityStore<E> {
    fn after_deser(&mut self) {
        self.entities.iter_mut().for_each(|(_, t)| t.after_deser());
    }
}
impl<E: Entity + ?Sized> PartialEq for EntityStore<E> {
    fn eq(&self, other: &Self) -> bool {
        self.time_range == other.time_range && {
            self.entities.len() == other.entities.len() && {
                self.entities.iter().all(|(self_uid, _self_entity)| {
                    if let Some(_other_entity) = other.entities.get(self_uid) {
                        // TODO: how should we compare entities for equality?
                        true
                    } else {
                        false
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EntityStore;
    use crate::prelude::*;
    use ensnare_core::prelude::*;
    use ensnare_proc_macros::{IsEntity, Metadata, Params};

    #[derive(Debug, Default, IsEntity, Metadata, Params)]
    #[entity("instrument")]
    struct ExampleEntity {
        pub uid: Uid,
        #[params]
        pub sample_rate: SampleRate,
    }
    impl Displays for ExampleEntity {}
    impl HandlesMidi for ExampleEntity {}
    impl Controllable for ExampleEntity {}
    impl Configurable for ExampleEntity {
        fn sample_rate(&self) -> SampleRate {
            self.sample_rate
        }

        fn update_sample_rate(&mut self, sample_rate: SampleRate) {
            self.sample_rate = sample_rate;
        }
    }
    impl Serializable for ExampleEntity {}
    impl Generates<StereoSample> for ExampleEntity {
        fn value(&self) -> StereoSample {
            StereoSample::default()
        }

        fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
            values.fill(StereoSample::default())
        }
    }
    impl Ticks for ExampleEntity {}
    impl From<ExampleEntityParams> for ExampleEntity {
        fn from(value: ExampleEntityParams) -> Self {
            Self {
                sample_rate: value.sample_rate,
                ..Default::default()
            }
        }
    }
    impl From<ExampleEntity> for ExampleEntityParams {
        fn from(value: ExampleEntity) -> Self {
            Self {
                sample_rate: value.sample_rate,
            }
        }
    }

    #[test]
    fn store_is_responsible_for_sample_rate() {
        let mut t = EntityStore::default();
        assert_eq!(t.sample_rate, SampleRate::DEFAULT);
        t.update_sample_rate(SampleRate(44444));

        let entity = Box::new(ExampleEntity::default());
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
        let one = Box::new(ExampleEntity::default());
        assert!(
            t.add(one, uid_1).is_ok(),
            "adding a unique UID should succeed"
        );

        let two = Box::new(ExampleEntity::default());
        assert!(
            t.add(two, uid_1).is_err(),
            "adding a duplicate UID should fail"
        );

        let uid_2 = Uid(10000);
        let two = Box::new(ExampleEntity::default());
        assert!(
            t.add(two, uid_2).is_ok(),
            "Adding a second unique UID should succeed."
        );
    }

    #[test]
    fn entity_store_partial_eq_excludes_sample_rate() {
        let es1 = EntityStore::<dyn Entity>::default();
        let mut es2 = EntityStore::default();

        assert_eq!(es1, es2, "Two default EntityStores should be equal");

        es2.update_sample_rate(SampleRate(es2.sample_rate.0 + 1));
        assert_eq!(es1, es2, "Two EntityStores that differ only by sample rate should still be equal, because sample rate is a DAW attribute, not a project attribute");

        es2.update_tempo(Tempo(es2.tempo().0 + 1.1));
        assert_ne!(es1, es2, "Tempo is part of PartialEq");
    }
}
