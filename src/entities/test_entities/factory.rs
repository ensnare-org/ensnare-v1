// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::{
    controllers::TestController,
    effects::TestEffect,
    instruments::{TestInstrument, TestInstrumentCountsMidiMessages},
};
use crate::prelude::*;
use ensnare::prelude::*;

/// Registers all [EntityFactory]'s test entities. Test entities are generally
/// simple, and provide instrumentation rather than useful audio functionality.
#[must_use]
pub fn register_test_entities(mut factory: EntityFactory<dyn Entity>) -> EntityFactory<dyn Entity> {
    factory.register_entity_with_str_key(TestInstrument::ENTITY_KEY, |_uid| {
        Box::new(TestInstrument::default())
    });
    factory.register_entity_with_str_key(TestInstrumentCountsMidiMessages::ENTITY_KEY, |_uid| {
        Box::new(TestInstrumentCountsMidiMessages::default())
    });
    factory.register_entity_with_str_key(TestController::ENTITY_KEY, |_uid| {
        Box::new(TestController::default())
    });
    factory.register_entity_with_str_key(TestEffect::ENTITY_KEY, |_uid| {
        Box::new(TestEffect::default())
    });

    factory
}
