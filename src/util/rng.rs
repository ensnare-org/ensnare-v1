// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Provides a random-number generator for debugging and testing.

use byteorder::{BigEndian, ByteOrder};
use delegate::delegate;

/// A pseudorandom number generator (PRNG) for applications that don't require
/// cryptographically secure random numbers.
#[derive(Debug)]
pub struct Rng(oorandom::Rand64);
impl Default for Rng {
    fn default() -> Self {
        // We want to panic if this fails, because it indicates that a core OS
        // facility isn't functioning.
        Self::new_with_seed(Self::generate_seed().unwrap())
    }
}
#[allow(missing_docs)]
impl Rng {
    /// Pass the same number to [Rng::new_with_seed()] to get the same stream
    /// back again. Good for reproducing test failures.
    pub fn new_with_seed(seed: u128) -> Self {
        Self(oorandom::Rand64::new(seed))
    }

    /// Create a sufficiently high-quality random number that's suitable for
    /// [Rng].
    pub fn generate_seed() -> anyhow::Result<u128> {
        let mut bytes = [0u8; 16];

        getrandom::getrandom(&mut bytes)?;
        Ok(BigEndian::read_u128(&bytes))
    }

    delegate! {
        to self.0 {
            pub fn rand_u64(&mut self) -> u64;
            pub fn rand_i64(&mut self) -> i64;
            pub fn rand_float(&mut self) -> f64;
            pub fn rand_range(&mut self, range: core::ops::Range<u64>) -> u64;
        }
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
