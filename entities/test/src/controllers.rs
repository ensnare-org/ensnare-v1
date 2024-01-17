// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::{IsEntity, Metadata};
use serde::{Deserialize, Serialize};

/// The smallest possible [IsEntity].
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controllable,
    Controls,
    Displays,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    SkipInner,
    Ticks,
    TransformsAudio
)]
pub struct TestController {
    uid: Uid,
}
impl TestController {
    pub fn new_with(uid: Uid) -> Self {
        Self { uid }
    }
}
