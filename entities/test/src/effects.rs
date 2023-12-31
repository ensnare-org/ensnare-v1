// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::{IsEntity2, Metadata, Params};
use serde::{Deserialize, Serialize};

/// The smallest possible [IsEntity2].
#[derive(Debug, Default, IsEntity2, Metadata, Params, Serialize, Deserialize)]
#[entity2(
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
pub struct TestEffect {
    uid: Uid,
}
impl TestEffect {
    pub fn new_with(uid: Uid, _params: &TestEffectParams) -> Self {
        Self { uid }
    }
}
