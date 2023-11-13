// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    traits::{Configurable, Controllable, Serializable, TransformsAudio},
    uid::Uid,
};
use ensnare_entity::traits::Displays;
use ensnare_proc_macros::{IsEffect, Metadata};

/// The smallest possible [IsEffect].
#[derive(Debug, Default, IsEffect, Metadata)]
pub struct TestEffect {
    uid: Uid,
}
impl Displays for TestEffect {}
impl Configurable for TestEffect {}
impl Controllable for TestEffect {}
impl Serializable for TestEffect {}
impl TransformsAudio for TestEffect {}
