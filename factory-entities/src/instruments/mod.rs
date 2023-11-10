// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    prelude::*,
    stuff::{
        drumkit::DrumkitParams, fm::FmSynthParams, sampler::SamplerParams, welsh::WelshSynthParams,
    },
    utils::Paths,
};
use ensnare_egui::instruments::{fm::fm_synth, sampler, welsh};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument,
    InnerSerializable, IsInstrument, Metadata,
};

#[derive(
    Debug,
    InnerControllable,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
pub struct Drumkit {
    uid: Uid,
    inner: ensnare_core::stuff::drumkit::Drumkit,
}
impl Displays for Drumkit {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Drumkit {
    pub fn new_with(uid: Uid, params: &DrumkitParams, paths: &Paths) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::drumkit::Drumkit::new_with(&params, paths),
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
    IsInstrument,
    Metadata,
)]
pub struct FmSynth {
    uid: Uid,
    inner: ensnare_core::stuff::fm::FmSynth,
}
impl Displays for FmSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(fm_synth(&mut self.inner))
    }
}
impl FmSynth {
    pub fn new_with(uid: Uid, params: &FmSynthParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::fm::FmSynth::new_with(params),
        }
    }
}

#[derive(
    Debug,
    InnerControllable,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
pub struct Sampler {
    uid: Uid,
    inner: ensnare_core::stuff::sampler::Sampler,
}
impl Displays for Sampler {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(sampler(&mut self.inner))
    }
}
impl Sampler {
    pub fn new_with(uid: Uid, params: &SamplerParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::sampler::Sampler::new_with(&params),
        }
    }

    pub fn load(&mut self, paths: &Paths) -> anyhow::Result<()> {
        self.inner.load(paths)
    }
}

#[derive(
    Debug,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsInstrument,
    Metadata,
)]
pub struct WelshSynth {
    uid: Uid,
    inner: ensnare_core::stuff::welsh::WelshSynth,
}
impl Displays for WelshSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(welsh(self.uid, &mut self.inner))
    }
}
impl WelshSynth {
    pub fn new_with(uid: Uid, params: &WelshSynthParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::welsh::WelshSynth::new_with(params),
        }
    }
}
