// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    generators::{Envelope, Oscillator},
    modulators::Dca,
    prelude::*,
    utils::Paths,
};
use ensnare_cores::LfoRouting;
use ensnare_cores_egui::instruments::{fm::fm_synth, sampler, welsh};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument,
    InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(
    Debug, InnerControllable, InnerConfigurable, IsEntity, Metadata, Serialize, Deserialize,
)]
#[entity(
    Controls,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    Ticks,
    TransformsAudio
)]

pub struct Drumkit {
    uid: Uid,
    inner: ensnare_cores::Drumkit,
}
impl Displays for Drumkit {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Drumkit {
    pub fn new_with(uid: Uid, name: &str, paths: &Paths) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Drumkit::new_with(name, paths),
        }
    }
}

#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct FmSynth {
    uid: Uid,
    inner: ensnare_cores::FmSynth,
}
impl Displays for FmSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(fm_synth(&mut self.inner))
    }
}
impl FmSynth {
    pub fn new_with(
        uid: Uid,
        carrier_oscillator: Oscillator,
        carrier_envelope: Envelope,
        modulator_oscillator: Oscillator,
        modulator_envelope: Envelope,
        depth: Normal,
        ratio: Ratio,
        beta: ParameterType,
        dca: Dca,
    ) -> Self {
        Self {
            uid,
            inner: ensnare_cores::FmSynth::new_with(
                carrier_oscillator,
                carrier_envelope,
                modulator_oscillator,
                modulator_envelope,
                depth,
                ratio,
                beta,
                dca,
            ),
        }
    }
}

#[derive(
    Debug,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(GeneratesStereoSample, Ticks, Controls, TransformsAudio)]
pub struct Sampler {
    uid: Uid,
    inner: ensnare_cores::Sampler,
}
impl Displays for Sampler {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(sampler(&mut self.inner))
    }
}
impl Sampler {
    pub fn new_with(uid: Uid, path: PathBuf, root: Option<FrequencyHz>) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Sampler::new_with(path, root),
        }
    }

    pub fn load(&mut self, paths: &Paths) -> anyhow::Result<()> {
        self.inner.load(paths)
    }
}

#[derive(
    Debug,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct WelshSynth {
    uid: Uid,
    inner: ensnare_cores::WelshSynth,
}
impl Displays for WelshSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(welsh(self.uid, &mut self.inner))
    }
}
impl WelshSynth {
    pub fn new_with(
        uid: Uid,
        oscillator_1: Oscillator,
        oscillator_2: Oscillator,
        oscillator_2_sync: bool,
        oscillator_mix: Normal,
        amp_envelope: Envelope,
        dca: Dca,
        lfo: Oscillator,
        lfo_routing: LfoRouting,
        lfo_depth: Normal,
        filter: ensnare_cores::BiQuadFilterLowPass24db,
        filter_cutoff_start: Normal,
        filter_cutoff_end: Normal,
        filter_envelope: Envelope,
    ) -> Self {
        Self {
            uid,
            inner: ensnare_cores::WelshSynth::new_with(
                oscillator_1,
                oscillator_2,
                oscillator_2_sync,
                oscillator_mix,
                amp_envelope,
                dca,
                lfo,
                lfo_routing,
                lfo_depth,
                filter,
                filter_cutoff_start,
                filter_cutoff_end,
                filter_envelope,
            ),
        }
    }
}
