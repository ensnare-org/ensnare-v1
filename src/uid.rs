// Copyright (c) 2024 Mike Tsao. All rights reserved.

use delegate::delegate;
use serde::{Deserialize, Serialize};

use crate::core::prelude::IsUid;

/// A [Uid] is an [Entity](crate::traits::Entity) identifier that is unique
/// within the current project.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    derive_more::Display,
)]
#[serde(rename_all = "kebab-case")]
pub struct Uid(pub usize);
impl IsUid for Uid {
    fn as_usize(&self) -> usize {
        self.0
    }
}
impl From<usize> for Uid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
