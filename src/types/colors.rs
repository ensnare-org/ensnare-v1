// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use bounded_vec_deque::BoundedVecDeque;
use crossbeam::{
    channel::{Receiver, Sender},
    queue::ArrayQueue,
};
use delegate::delegate;
use derivative::Derivative;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, RangeInclusive, Sub},
    sync::{Arc, RwLock},
};
use std::{
    hash::Hash,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};
use strum_macros::{EnumCount, EnumIter, FromRepr};

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
