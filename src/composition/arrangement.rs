// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use delegate::delegate;
use serde::{Deserialize, Serialize};
use synonym::Synonym;

/// A [Uid] that identifies an arrangement, which is a sequence of notes that
/// have been positioned on a track.
#[derive(Synonym, Serialize, Deserialize)]
pub struct ArrangementUid(usize);
impl IsUid for ArrangementUid {
    fn as_usize(&self) -> usize {
        self.0
    }
}

/// Mints unique [ArrangementUid]s.
#[derive(Synonym, Debug, Serialize, Deserialize)]
pub struct ArrangementUidFactory(UidFactory<ArrangementUid>);
impl Default for ArrangementUidFactory {
    fn default() -> Self {
        Self(UidFactory::<ArrangementUid>::new(262144))
    }
}
impl ArrangementUidFactory {
    delegate! {
        to self.0 {
            /// Generates the next unique [ArrangementUid].
            pub fn mint_next(&self) -> ArrangementUid;
        }
    }
}
