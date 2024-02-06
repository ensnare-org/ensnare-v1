// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::core::prelude::*;
use crate::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable, IsEntity, Metadata,
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
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(
    Controllable,
    Displays,
    GeneratesStereoSample,
    SkipInner,
    Ticks,
    TransformsAudio
)]
pub struct TestControllerAlwaysSendsMidiMessage {
    uid: Uid,
    #[serde(skip)]
    inner: crate::cores::instruments::TestControllerAlwaysSendsMidiMessage,
}
impl TestControllerAlwaysSendsMidiMessage {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: crate::cores::instruments::TestControllerAlwaysSendsMidiMessage::default(),
        }
    }
}
