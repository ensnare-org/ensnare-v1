// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{cores::instruments::TestControllerAlwaysSendsMidiMessageCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerControls, InnerHandlesMidi, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// The smallest possible [IsEntity].
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Displays, GeneratesStereoSample, SkipInner, Ticks, TransformsAudio)]
pub struct TestControllerAlwaysSendsMidiMessage {
    uid: Uid,
    #[serde(skip)]
    inner: TestControllerAlwaysSendsMidiMessageCore,
}
impl TestControllerAlwaysSendsMidiMessage {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: TestControllerAlwaysSendsMidiMessageCore::default(),
        }
    }
}
