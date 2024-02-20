// Copyright (c) 2024 Mike Tsao. All rights reserved.

pub use drumkit::{DrumkitWidget, DrumkitWidgetAction};
pub use fm::{FmSynthWidget, FmSynthWidgetAction};
pub use sampler::{SamplerWidget, SamplerWidgetAction};
pub use subtractive::{SubtractiveSynthWidget, SubtractiveSynthWidgetAction};

mod drumkit;
mod fm;
mod sampler;
mod subtractive;
