// Copyright (c) 2023 Mike Tsao. All rights reserved.

use std::{fmt::Display, hash::Hash, marker::PhantomData, sync::atomic::AtomicUsize};

/// An optional Uid trait.
pub trait IsUid: Eq + Hash + Clone + Copy + From<usize> {}

/// A [Uid] is an [Entity](crate::traits::Entity) identifier that is unique
/// within the current project.
#[derive(
    Clone, Copy, Debug, Default, derive_more::Display, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub struct Uid(pub usize);
impl IsUid for Uid {}
impl From<usize> for Uid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

pub type EntityUidFactory = UidFactory<Uid>;
impl UidFactory<Uid> {
    pub const FIRST_UID: AtomicUsize = AtomicUsize::new(1024);
}
impl Default for UidFactory<Uid> {
    fn default() -> Self {
        Self {
            next_uid_value: Self::FIRST_UID,
            _phantom: Default::default(),
        }
    }
}

/// Generates unique [Uid]s.
#[derive(Debug)]
pub struct UidFactory<U: IsUid> {
    pub(crate) next_uid_value: AtomicUsize,
    pub(crate) _phantom: PhantomData<U>,
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

/// Identifies a [Track].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TrackUid(pub usize);
impl Default for TrackUid {
    fn default() -> Self {
        Self(1)
    }
}
impl IsUid for TrackUid {}
impl From<usize> for TrackUid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for TrackUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

pub type TrackUidFactory = UidFactory<TrackUid>;
impl UidFactory<TrackUid> {
    pub const FIRST_UID: AtomicUsize = AtomicUsize::new(1);
}
impl Default for UidFactory<TrackUid> {
    fn default() -> Self {
        Self {
            next_uid_value: Self::FIRST_UID,
            _phantom: Default::default(),
        }
    }
}
