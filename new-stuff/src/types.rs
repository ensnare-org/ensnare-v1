// Copyright (c) 2024 Mike Tsao. All rights reserved.

use ensnare_core::{control::ControlIndex, piano_roll::PatternUid, time::MusicalTime, uid::Uid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ArrangementUid(usize);
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlLink {
    pub uid: Uid,
    pub param: ControlIndex,
}
