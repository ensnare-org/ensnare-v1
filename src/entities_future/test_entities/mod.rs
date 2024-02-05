// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Test instruments and effects.

pub use controllers::TestController;
pub use effects::{TestEffect, TestEffectNegatesInput};
pub use factory::register_test_entities;
pub use instruments::{TestAudioSource, TestInstrument, TestInstrumentCountsMidiMessages};

mod controllers;
mod effects;
mod factory;
mod instruments;
