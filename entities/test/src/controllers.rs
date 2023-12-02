// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    traits::{Configurable, Controls, HandlesMidi, Serializable},
    uid::Uid,
};
use ensnare_entity::traits::Displays;
use ensnare_proc_macros::{IsEntity, Metadata, Params};

/// The smallest possible [IsEntity].
#[derive(Debug, Default, IsEntity, Metadata, Params)]
#[entity("controller", "skip_inner")]
pub struct TestController {
    uid: Uid,
}
impl TestController {
    pub fn new_with(uid: Uid, _: &TestControllerParams) -> Self {
        Self { uid }
    }
}
impl Displays for TestController {}
impl HandlesMidi for TestController {}
impl Controls for TestController {}
impl Configurable for TestController {}
impl Serializable for TestController {}
