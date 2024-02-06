// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Sender};

/// A convenience struct to bundle both halves of a [crossbeam_channel]
/// together.
///
/// This is actually for more than just convenience: because Serde needs to be
/// able to assign defaults to individual fields on a struct by calling
/// stateless functions, we have to create both sender and receiver at once in a
/// single field.
#[derive(Debug)]
pub struct ChannelPair<T> {
    #[allow(missing_docs)]
    pub sender: Sender<T>,
    #[allow(missing_docs)]
    pub receiver: Receiver<T>,
}
impl<T> Default for ChannelPair<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}
