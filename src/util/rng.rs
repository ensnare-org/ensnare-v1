// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Provides a random-number generator for debugging and testing.

use std::time::{SystemTime, UNIX_EPOCH};

/// A pseudorandom number generator (PRNG) for applications that don't require
/// cryptographically secure random numbers. Pass the same number to
/// [Rng::new_with_seed()] to get the same stream back again.
#[derive(Debug)]
pub struct Rng(oorandom::Rand64);
impl Default for Rng {
    fn default() -> Self {
        // This is a poor source of entropy if we want the random-number stream
        // to be unpredictable. It's fine for generating different test sets
        // from run to run.
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self::new_with_seed(seed)
    }
}
impl Rng {
    pub fn new_with_seed(seed: u128) -> Self {
        Self(oorandom::Rand64::new(seed))
    }

    pub fn rand_u64(&mut self) -> u64 {
        self.0.rand_u64()
    }

    pub fn rand_i64(&mut self) -> i64 {
        self.0.rand_i64()
    }

    pub fn rand_float(&mut self) -> f64 {
        self.0.rand_float()
    }

    pub fn rand_range(&mut self, range: std::ops::Range<u64>) -> u64 {
        self.0.rand_range(range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainline() {
        let mut r = Rng::default();
        assert_ne!(r.rand_u64(), r.rand_u64());
    }

    #[test]
    fn reproducible_stream() {
        let mut r1 = Rng::new_with_seed(1);
        let mut r2 = Rng::new_with_seed(2);
        assert!(
            (0..100).any(|_| r1.rand_u64() != r2.rand_u64()),
            "RNGs with different seeds should produce different streams (or else you should play the lottery ASAP because you 2^6400 pairs of coin flips just came up the same)."
        );

        let mut r1 = Rng::new_with_seed(1);
        let mut r2 = Rng::new_with_seed(1);
        assert!(
            (0..100).all(|_| r1.rand_u64() == r2.rand_u64()),
            "RNGs with same seeds should produce same streams."
        );
    }
}
