// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::time::MusicalTime;

/// A [ViewRange] indicates a musical time range. It's used to determine what a
/// widget should show when it's rendering something in a timeline.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewRange(pub std::ops::Range<MusicalTime>);
