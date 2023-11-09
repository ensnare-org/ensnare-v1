// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{IsController, IsEffect, IsInstrument, Metadata};
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;

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
}
