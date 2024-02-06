// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Unique identifiers for various system structs, and factories that help
//! ensure they are in fact unique.

use core::sync::atomic::Ordering;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, marker::PhantomData, sync::atomic::AtomicUsize};

/// An identifier that is unique within the current project.
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

/// An optional Uid trait.
pub trait IsUid: Eq + Hash + Clone + From<usize> {
    fn as_usize(&self) -> usize;
}

/// Generates unique uids.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UidFactory<U: IsUid> {
    pub(crate) next_uid_value: AtomicUsize,
    #[serde(skip)]
    pub(crate) _phantom: PhantomData<U>,
}
impl<U: IsUid> UidFactory<U> {
    /// Creates a new [UidFactory] starting with the given value.
    pub fn new(first_uid: usize) -> Self {
        Self {
            next_uid_value: AtomicUsize::new(first_uid),
            _phantom: Default::default(),
        }
    }

    /// Generates the next unique uid.
    pub fn mint_next(&self) -> U {
        let uid_value = self.next_uid_value.fetch_add(1, Ordering::Relaxed);
        U::from(uid_value)
    }

    /// Notifies the factory that a uid exists that might have been created
    /// elsewhere (for example, during deserialization of a project). This gives
    /// the factory an opportunity to adjust `next_uid_value` to stay consistent
    /// with all known uids.
    pub fn notify_externally_minted_uid(&self, uid: U) {
        if uid.as_usize() >= self.next_uid_value.load(Ordering::Relaxed) {
            self.next_uid_value
                .store(uid.as_usize() + 1, Ordering::Relaxed);
        }
    }
}
impl<U: IsUid> PartialEq for UidFactory<U> {
    fn eq(&self, other: &Self) -> bool {
        self.next_uid_value.load(Ordering::Relaxed) == other.next_uid_value.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use derivative::Derivative;
    use std::collections::HashSet;

    #[derive(Copy, Clone, Debug, Derivative, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[derivative(Default)]
    pub struct TestUid(#[derivative(Default(value = "1"))] pub usize);
    impl IsUid for TestUid {
        fn as_usize(&self) -> usize {
            self.0
        }
    }
    impl From<usize> for TestUid {
        fn from(value: usize) -> Self {
            Self(value)
        }
    }
    impl UidFactory<TestUid> {
        pub const FIRST_UID: AtomicUsize = AtomicUsize::new(1);
    }
    impl Default for UidFactory<TestUid> {
        fn default() -> Self {
            Self {
                next_uid_value: Self::FIRST_UID,
                _phantom: Default::default(),
            }
        }
    }

    #[test]
    fn uid_factory() {
        let f = UidFactory::<TestUid>::default();

        let uid_1 = f.mint_next();
        let uid_2 = f.mint_next();
        assert_ne!(uid_1, uid_2, "Minted Uids should not repeat");

        let uid_3 = TestUid(uid_2.0 + 1);
        let uid_3_expected_duplicate = f.mint_next();
        assert_eq!(
            uid_3, uid_3_expected_duplicate,
            "Minted Uids will repeat if factory doesn't know about them all"
        );

        // This is redundant. Taken from an Orchestrator unit test and adopted here.
        let mut ids: HashSet<TestUid> = HashSet::default();
        for _ in 0..64 {
            let uid = f.mint_next();
            assert!(
                !ids.contains(&uid),
                "added entities should be assigned unique IDs"
            );
            ids.insert(uid);
        }
    }

    #[test]
    fn uid_factory_with_notify_works() {
        let f = UidFactory::<TestUid>::default();

        let uid_1 = f.mint_next();
        let uid_2 = f.mint_next();
        assert_ne!(uid_1, uid_2, "Minted Uids should not repeat");

        let uid_3 = TestUid(uid_2.0 + 1);
        f.notify_externally_minted_uid(uid_3);
        let uid_4 = f.mint_next();
        assert_ne!(
            uid_3, uid_4,
            "Notifying factory should cause it to skip past."
        );

        f.notify_externally_minted_uid(uid_3);
        let uid_5 = f.mint_next();
        assert_eq!(
            uid_5.0,
            uid_4.0 + 1,
            "Notifying factory about value below next should be no-op."
        );
    }
}
