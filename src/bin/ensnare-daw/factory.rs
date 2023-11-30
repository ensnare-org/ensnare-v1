// Copyright (c) 2023 Mike Tsao. All rights reserved.

use delegate::delegate;
use ensnare::{prelude::*, all_entities::EntityWrapper};
use ensnare_cores::ArpeggiatorParams;
use ensnare_entities::controllers::Arpeggiator;

#[derive(Debug, Default)]
pub(crate) struct EnsnareEntityFactory(EntityFactory<(dyn EntityWrapper + 'static)>);
impl EnsnareEntityFactory {
    pub(crate) fn register_entities() -> Self {
        let mut factory = Self::default();

        factory
            .0
            .register_entity_with_str_key(Arpeggiator::ENTITY_KEY, |uid| {
                Box::new(Arpeggiator::new_with(uid, &ArpeggiatorParams::default()))
            });

        factory
    }

    delegate! {
        to self.0 {
            pub fn new_entity(&self, key: &EntityKey, uid: Uid) -> Option<Box<dyn EntityWrapper + 'static>>;
        }
    }
}
