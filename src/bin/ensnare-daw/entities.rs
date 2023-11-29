// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_cores::ArpeggiatorParams;
use ensnare_entities::controllers::Arpeggiator;
use ensnare_entity::traits::Entity;
use serde::{Deserialize, Serialize};

trait EntityWrapper: Entity + MakesParams {}

trait MakesParams {
    fn make_params(&self) -> anyhow::Result<Box<EntityParams>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EntityParams {
    Arpeggiator(ArpeggiatorParams),
}

impl TryFrom<&Box<dyn EntityWrapper>> for Box<EntityParams> {
    type Error = anyhow::Error;

    fn try_from(value: &Box<dyn EntityWrapper>) -> Result<Self, Self::Error> {
        value.make_params()
    }
}

impl MakesParams for Arpeggiator {
    fn make_params(&self) -> anyhow::Result<Box<EntityParams>> {
        anyhow::Ok(Box::new(EntityParams::Arpeggiator(
            self.try_into().unwrap(),
        )))
    }
}
