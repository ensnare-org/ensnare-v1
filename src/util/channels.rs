// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Sender};

/// Same idea as [ensnare_services::CrossbeamChannel] but only for bounded of
/// bounds 1.
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
