// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Provides a programmatic way to load music samples.

use derive_more::Display;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

static INSTANCE: OnceCell<SampleLibrary> = OnceCell::new();

#[derive(Debug, Default, Copy, Clone, Display, Serialize, Deserialize, PartialEq)]
pub struct SampleIndex(pub usize);
impl From<usize> for SampleIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SampleSource {
    SampleLibrary(SampleIndex),
    Path(PathBuf),
}
impl Default for SampleSource {
    fn default() -> Self {
        SampleSource::SampleLibrary(SampleIndex::default())
    }
}

#[derive(Debug, Default)]
pub struct SampleLibrary {}
impl SampleLibrary {
    pub fn choices(&self) -> &[&str] {
        &["Pluck", "Mellotron", "Vinyl Scratch"]
    }

    pub fn path(&self, index: SampleIndex) -> Option<PathBuf> {
        match index.0 {
            0 => Some(PathBuf::from("stereo-pluck.wav")),
            1 => Some(PathBuf::from("mellotron-woodwinds-c4.wav")),
            2 => Some(PathBuf::from("vinyl-scratch.wav")),
            _ => None,
        }
    }

    pub fn set_instance(instance: Self) {
        INSTANCE.set(instance);
    }

    pub(crate) fn global() -> &'static Self {
        INSTANCE.get().expect("SampleLibrary is not initialized")
    }
}
