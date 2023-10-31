// Copyright (c) 2023 Mike Tsao. All rights reserved.

use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, marker::PhantomData, sync::atomic::AtomicUsize};

/// An optional Uid trait.
pub trait IsUid: Eq + Hash + Clone + Copy + From<usize> {}

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
impl IsUid for Uid {}
impl From<usize> for Uid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Generates unique [Uid]s.
#[derive(Debug, Serialize, Deserialize)]
pub struct UidFactory<U: IsUid> {
    next_uid_value: AtomicUsize,
    _phantom: PhantomData<U>,
}
impl<U: IsUid> Default for UidFactory<U> {
    fn default() -> Self {
        Self {
            next_uid_value: AtomicUsize::new(1),
            _phantom: Default::default(),
        }
    }
}
impl<U: IsUid> UidFactory<U> {
    /// Creates a new UidFactory starting with the given [Uid] value.
    pub fn new(first_uid: usize) -> Self {
        Self {
            next_uid_value: AtomicUsize::new(first_uid),
            _phantom: Default::default(),
        }
    }

    /// Generates the next unique [Uid].
    pub fn mint_next(&self) -> U {
        let uid_value = self
            .next_uid_value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        U::from(uid_value)
    }
}
