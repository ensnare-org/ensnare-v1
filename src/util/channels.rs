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
pub struct CrossbeamChannel<T> {
    #[allow(missing_docs)]
    pub sender: Sender<T>,
    #[allow(missing_docs)]
    pub receiver: Receiver<T>,
}
impl<T> Default for CrossbeamChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

/// Same idea but only for bounded of bounds 1.
#[derive(Debug)]
pub struct BoundedCrossbeamChannel<T> {
    #[allow(missing_docs)]
    pub sender: Sender<T>,
    #[allow(missing_docs)]
    pub receiver: Receiver<T>,
}
impl<T> Default for BoundedCrossbeamChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        Self { sender, receiver }
    }
}
