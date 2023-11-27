// Copyright (c) 2023 Mike Tsao. All rights reserved.

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Rng(oorandom::Rand64);
impl Default for Rng {
    fn default() -> Self {
        // This is an awful source of entropy, but it's fine for this use case
        // where we just want a different fake struct each time.
        Self(oorandom::Rand64::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ))
    }
}
impl Rng {
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
}
