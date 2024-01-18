// Copyright (c) 2024 Mike Tsao. All rights reserved.

use super::PatternUid;
use crate::time::MusicalTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Arrangement {
    pub pattern_uid: PatternUid,
    pub position: MusicalTime,
    pub duration: MusicalTime,
}
