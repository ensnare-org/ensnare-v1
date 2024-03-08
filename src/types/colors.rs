// Copyright (c) 2024 Mike Tsao. All rights reserved.

use serde::{Deserialize, Serialize};
use strum_macros::{EnumCount, EnumIter, FromRepr};

#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    EnumCount,
    EnumIter,
    Eq,
    FromRepr,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    Red,
    Vermilion,
    Orange,
    Amber,
    Yellow,
    Lime,
    Chartreuse,
    Ddahal,
    Green,
    Erin,
    Spring,
    Gashyanta,
    Cyan,
    Capri,
    Azure,
    Cerulean,
    Blue,
    Volta,
    Violet,
    Llew,
    Magenta,
    Cerise,
    Rose,
    Crimson,
    #[default]
    Gray1,
    Gray2,
    Gray3,
    Gray4,
    Gray5,
    Gray6,
    Gray7,
    Gray8,
}
