// Copyright (c) 2024 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ArrangementUid(usize);
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ControlLink {
    pub uid: Uid,
    pub param: ControlIndex,
}
