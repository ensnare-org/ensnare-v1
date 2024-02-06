// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Useful things that don't have anything to do with digital audio.

pub mod prelude {
    pub use super::{ChannelPair, ModSerial};
}

pub use channel_pair::ChannelPair;
pub use mod_serial::ModSerial;
pub use paths::{FileType, Paths};
pub use rng::Rng;
pub use selection_set::SelectionSet;

mod channel_pair;
mod mod_serial;
pub mod paths;
pub mod rng;
pub mod selection_set;
