// Copyright (c) 2023 Mike Tsao. All rights reserved.

use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// An optional Uid trait.
pub trait IsUid: Eq + Hash + Clone + Copy {
    /// Changes the current Uid to the next one. Does not guarantee uniqueness.
    fn increment(&mut self) -> &Self;
}

/// A [Uid] is an identifier that's unique within the current project.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
pub struct Uid(pub usize);
impl IsUid for Uid {
    fn increment(&mut self) -> &Self {
        self.0 += 1;
        self
    }
}

/// Generates unique [Uid]s. This factory is not threadsafe.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UidFactory<U: IsUid + Clone> {
    previous_uid: U,
}
impl<U: IsUid + Clone> UidFactory<U> {
    /// Generates the next unique [Uid].
    pub fn mint_next(&mut self) -> U {
        *self.previous_uid.increment()
    }
}
