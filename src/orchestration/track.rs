// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use synonym::Synonym;

/// Newtype for track title string.
#[derive(Synonym, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[synonym(skip(Default))]
#[serde(rename_all = "kebab-case")]
pub struct TrackTitle(#[derivative(Default(value = "\"Untitled\".to_string()"))] pub String);

/// Identifies a track.
#[derive(Synonym, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[synonym(skip(Default))]
#[serde(rename_all = "kebab-case")]
pub struct TrackUid(#[derivative(Default(value = "1"))] pub usize);
impl IsUid for TrackUid {
    fn as_usize(&self) -> usize {
        self.0
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
