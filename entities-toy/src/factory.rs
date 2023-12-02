// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{controllers::*, effects::*, instruments::*};
use ensnare_cores::toys::{
    ToyControllerParams, ToyEffectParams, ToyInstrumentParams, ToySynthParams,
};
use ensnare_entity::{prelude::*, traits::EntityBounds};

/// Registers toy entities for the given [EntityFactory]. Toy entities are very
/// simple but working instruments. They're helpful when you think you're going
/// nuts because nothing is working, so you want something that doesn't have
/// lots of settings.
pub struct ToyEntities {}
impl ToyEntities {
    /// Registers all the entities in this collection.
    pub fn register(
        mut factory: EntityFactory<dyn EntityBounds>,
    ) -> EntityFactory<dyn EntityBounds> {
        factory.register_entity(EntityKey::from(ToySynth::ENTITY_KEY), |uid| {
            Box::new(ToySynth::new_with(uid, &ToySynthParams::default()))
        });
        factory.register_entity(EntityKey::from(ToyInstrument::ENTITY_KEY), |uid| {
            Box::new(ToyInstrument::new_with(
                uid,
                &ToyInstrumentParams::default(),
            ))
        });
        factory.register_entity(EntityKey::from(ToyController::ENTITY_KEY), |uid| {
            Box::new(ToyController::new_with(
                uid,
                &ToyControllerParams::default(),
            ))
        });
        factory.register_entity(EntityKey::from(ToyEffect::ENTITY_KEY), |uid| {
            Box::new(ToyEffect::new_with(uid, &ToyEffectParams::default()))
        });

        factory
    }
}
