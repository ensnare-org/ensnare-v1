// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Ensnare structs that implement the Entity trait.

pub mod controllers;
pub mod effects;
pub mod factory;
pub mod instruments;
pub mod piano_roll;

pub use factory::{BuiltInEntities, BuiltInEntityFactory};

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::BuiltInEntities;
}

#[cfg(test)]
pub mod tests {
    use ensnare_core::uid::Uid;
    use ensnare_entity::{prelude::*, traits::EntityBounds};

    // TODO: this is copied from ensnare_core::entities::factory
    pub fn check_entity_factory(factory: EntityFactory<dyn EntityBounds>) {
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
}
