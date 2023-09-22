// Copyright (c) 2023 Mike Tsao. All rights reserved.

use anyhow::anyhow;
use atomic_counter::{AtomicCounter, RelaxedCounter};
use derive_more::Display;
use ensnare_core::{
    prelude::*,
    traits::{Configurable, ControlEventsFn, Controls, Entity, Serializable, Ticks},
    uid::Uid,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    option::Option,
};

#[cfg(test)]
use ensnare_core::{
    entities::{EntityFactory, Key},
    midi::MidiChannel,
};
#[cfg(test)]
use ensnare_toys::{
    ToyController, ToyControllerParams, ToyEffect, ToyInstrument, ToyInstrumentParams,
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
#[cfg(test)]
#[must_use]
pub fn register_test_factory_entities(mut factory: EntityFactory) -> EntityFactory {
    factory.register_entity(Key::from("instrument"), || {
        Box::new(ToyInstrument::new_with(&ToyInstrumentParams::default()))
    });
    factory.register_entity(Key::from("controller"), || {
        Box::new(ToyController::new_with(
            &ToyControllerParams::default(),
            MidiChannel::from(0),
        ))
    });
    factory.register_entity(Key::from("effect"), || Box::new(ToyEffect::default()));

    factory.complete_registration();

    factory
}
