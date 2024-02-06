// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derivative::Derivative;
use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Newtype for track title string.
#[derive(Debug, Derivative, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct TrackTitle(#[derivative(Default(value = "\"Untitled\".to_string()"))] pub String);
impl From<&str> for TrackTitle {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Identifies a track.
#[derive(
    Copy,
    Clone,
    Debug,
    Derivative,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub struct TrackUid(#[derivative(Default(value = "1"))] pub usize);
impl IsUid for TrackUid {
    fn as_usize(&self) -> usize {
        self.0
    }
}
impl From<usize> for TrackUid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackUidFactory(UidFactory<TrackUid>);
impl Default for TrackUidFactory {
    fn default() -> Self {
        Self(UidFactory::<TrackUid>::new(1))
    }
}
impl TrackUidFactory {
    delegate! {
        to self.0 {
            pub fn mint_next(&self) -> TrackUid;
        }
    }
}
