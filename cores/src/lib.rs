// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Cores are basic musical devices.

use anyhow::{anyhow, Result};
use ensnare_core::{
    composition::{Pattern, PatternBuilder, PatternUid},
    prelude::*,
    selection_set::SelectionSet,
    traits::Sequences,
    types::ColorScheme,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use controllers::*;
pub use effects::*;
pub use instruments::*;

pub mod controllers;
pub mod effects;
pub mod instruments;
pub mod toys;
