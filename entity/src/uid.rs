// Copyright (c) 2024 Mike Tsao. All rights reserved.

use delegate::delegate;
use ensnare_core::uid::{IsUid, UidFactory};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityUidFactory(pub UidFactory<Uid>);
impl EntityUidFactory {
    const FIRST_UID: usize = 1024;

    delegate! {
        to self.0 {
            pub fn mint_next(&self) -> Uid;
        }
    }
}
impl Default for EntityUidFactory {
    fn default() -> Self {
        Self(UidFactory::<Uid>::new(Self::FIRST_UID))
    }
}
