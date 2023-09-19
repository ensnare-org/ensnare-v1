// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::core::ParameterType;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Beats per minute.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tempo(pub ParameterType);
impl Default for Tempo {
    fn default() -> Self {
        Self(128.0)
    }
}
impl fmt::Display for Tempo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:0.2} BPM", self.0))
    }
}
impl From<u16> for Tempo {
    fn from(value: u16) -> Self {
        Self(value as ParameterType)
    }
}
impl From<ParameterType> for Tempo {
    fn from(value: ParameterType) -> Self {
        Self(value)
    }
}
impl Tempo {
    /// The largest value we'll allow.
    pub const MAX_VALUE: ParameterType = 1024.0;

    /// The smallest value we'll allow. Note that zero is actually a degenerate
    /// case... maybe we should be picking 0.1 or similar.
    pub const MIN_VALUE: ParameterType = 0.0;

    #[allow(missing_docs)]
    /// A getter for the raw value.
    pub fn value(&self) -> ParameterType {
        self.0
    }
    /// Beats per second.
    pub fn bps(&self) -> ParameterType {
        self.0 / 60.0
    }
}
