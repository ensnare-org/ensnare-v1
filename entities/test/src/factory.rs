// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::controllers::TestController;
use super::effects::TestEffect;
use super::instruments::{TestInstrument, TestInstrumentCountsMidiMessages};
use ensnare_entity::factory::EntityFactory;

/// Registers all [EntityFactory]'s test entities. Test entities are generally
/// simple, and provide instrumentation rather than useful audio functionality.
#[must_use]
pub fn register_test_entities(mut factory: EntityFactory) -> EntityFactory {
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
