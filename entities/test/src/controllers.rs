// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    traits::{Configurable, Controls, HandlesMidi, Serializable},
    uid::Uid,
};
use ensnare_entity::traits::Displays;
use ensnare_proc_macros::{IsController, Metadata};

/// The smallest possible [IsController].
#[derive(Debug, Default, IsController, Metadata)]
pub struct TestController {
    uid: Uid,
}
impl Displays for TestController {}
impl HandlesMidi for TestController {}
impl Controls for TestController {}
impl Configurable for TestController {}
impl Serializable for TestController {}
