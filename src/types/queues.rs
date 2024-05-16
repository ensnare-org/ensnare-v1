// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use bounded_vec_deque::BoundedVecDeque;
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

/// A ring buffer of mono samples used to visualize the generated audio stream.
#[derive(Debug)]
pub struct VisualizationQueue(pub Arc<RwLock<BoundedVecDeque<Sample>>>);
impl Default for VisualizationQueue {
    fn default() -> Self {
        const LEN: usize = 256;
        let mut deque = VecDeque::new();
        deque.resize(LEN, Sample::default());
        Self(Arc::new(RwLock::new(BoundedVecDeque::from_unbounded(
            deque, LEN,
        ))))
    }
}
impl Clone for VisualizationQueue {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
