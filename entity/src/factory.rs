// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::traits::{Entity, IsEffect, IsInstrument};
use anyhow::anyhow;
use derive_more::Display;
use ensnare_core::prelude::*;
use once_cell::sync::OnceCell;
use std::{
    collections::{hash_map, HashMap, HashSet},
    option::Option,
};

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

/// [EntityFactory] accepts [EntityKey]s and creates instruments, controllers,
/// and effects. It makes sure every entity has a proper [Uid].
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

    /// Registers a new type for the given [EntityKey] using the given closure.
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

    /// Registers a new type for the given [EntityKey] using the given closure,
    /// but takes a &str and creates the [EntityKey] from it.
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

    /// Creates a new entity of the type corresponding to the given [EntityKey]
    /// with the given [Uid].
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

    /// Returns the [HashSet] of all [EntityKey]s.
    pub fn keys(&self) -> &HashSet<EntityKey> {
        &self.keys
    }

    /// Returns the [HashMap] for all [EntityKey] and entity pairs.
    pub fn entities(&self) -> &HashMap<EntityKey, EntityFactoryFn> {
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

    /// Sets the singleton [EntityFactory].
    pub fn initialize(entity_factory: Self) -> Result<(), Self> {
        FACTORY.set(entity_factory)
    }
}

/// An [EntityStore] owns [Entities](Entity). It implements some [Entity]
/// traits, such as [Configurable], and fans out usage of those traits to the
/// owned entities, making it easier for the owner of an [EntityStore] to treat
/// all its entities as a single [Entity].
#[derive(Debug, Default)]
pub struct EntityStore {
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
                    //    debug_assert!(claimed_uid.is_none(), "Entities
                    //    controlled by EntityStore should not know or claim to
                    //    know their own Uid");

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

#[cfg(test)]
mod tests {
    use super::{EntityFactory, EntityStore};
    use crate::{
        factory::EntityKey,
        test_entities::{register_test_entities, TestInstrument},
    };
    use ensnare_core::prelude::*;

    #[test]
    fn store_is_responsible_for_sample_rate() {
        let mut t = EntityStore::default();
        assert_eq!(t.sample_rate, SampleRate::DEFAULT);
        t.update_sample_rate(SampleRate(44444));
        let factory = register_test_entities(EntityFactory::default());

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
