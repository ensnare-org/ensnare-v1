// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::controllers::ControlTripParams;
use ensnare_cores::{ArpeggiatorParams, LivePatternSequencerParams};
use ensnare_entities::controllers::{Arpeggiator, ControlTrip, LivePatternSequencer};
use ensnare_entity::traits::EntityBounds;
use serde::{Deserialize, Serialize};

pub trait EntityWrapper: EntityBounds + MakesParams {}

pub trait MakesParams {
    fn make_params(&self) -> anyhow::Result<Box<EntityParams>>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum EntityParams {
    Arpeggiator(ArpeggiatorParams),
    LivePatternSequencer(LivePatternSequencerParams),
    ControlTrip(ControlTripParams),
}

impl TryFrom<&Box<dyn EntityWrapper>> for Box<EntityParams> {
    type Error = anyhow::Error;

    fn try_from(value: &Box<dyn EntityWrapper>) -> Result<Self, Self::Error> {
        value.make_params()
    }
}

impl EntityWrapper for Arpeggiator {}
impl MakesParams for Arpeggiator {
    fn make_params(&self) -> anyhow::Result<Box<EntityParams>> {
        anyhow::Ok(Box::new(EntityParams::Arpeggiator(
            self.try_into().unwrap(),
        )))
    }
}

impl EntityWrapper for LivePatternSequencer {}
impl MakesParams for LivePatternSequencer {
    fn make_params(&self) -> anyhow::Result<Box<EntityParams>> {
        anyhow::Ok(Box::new(EntityParams::LivePatternSequencer(
            self.try_into().unwrap(),
        )))
    }
}

impl EntityWrapper for ControlTrip {}
impl MakesParams for ControlTrip {
    fn make_params(&self) -> anyhow::Result<Box<EntityParams>> {
        anyhow::Ok(Box::new(EntityParams::ControlTrip(
            self.try_into().unwrap(),
        )))
    }
}
