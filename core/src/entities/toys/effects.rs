// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{prelude::*, traits::prelude::*, widgets::core::drag_normal};
use ensnare_proc_macros::{Control, IsEffect, Params, Uid};
use serde::{Deserialize, Serialize};

/// An [IsEffect](ensnare::traits::IsEffect) that applies a negative gain.
#[derive(Debug, Default, Control, IsEffect, Params, Uid, Serialize, Deserialize)]
pub struct ToyEffect {
    uid: Uid,

    /// The [ToyEffect] transformation is signal * -magnitude.
    #[control]
    #[params]
    magnitude: Normal,

    #[serde(skip)]
    sample_rate: SampleRate,
}

impl ToyEffect {
    pub fn new_with(params: &ToyEffectParams) -> Self {
        Self {
            uid: Default::default(),
            magnitude: params.magnitude,
            sample_rate: Default::default(),
        }
    }

    pub fn set_magnitude(&mut self, magnitude: Normal) {
        self.magnitude = magnitude;
    }

    pub fn magnitude(&self) -> Normal {
        self.magnitude
    }
}
impl TransformsAudio for ToyEffect {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        input_sample * self.magnitude * -1.0
    }
}
impl Configurable for ToyEffect {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}
impl Serializable for ToyEffect {}

impl Displays for ToyEffect {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(drag_normal(&mut self.magnitude, "Magnitude: "))
    }
}
