// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Provides a programmatic way to load music samples.

use derive_more::Display;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::midi::{GeneralMidiPercussionProgram, MidiNote};

static INSTANCE: OnceCell<SampleLibrary> = OnceCell::new();
static KIT_INSTANCE: OnceCell<KitSampleLibrary> = OnceCell::new();

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

#[derive(Debug)]
pub struct SampleItem {
    name: String,
    path: PathBuf,
}

#[derive(Debug)]
pub struct SampleLibrary {
    names: Vec<String>,
    samples: Vec<SampleItem>,
}
impl Default for SampleLibrary {
    fn default() -> Self {
        let mut r = Self {
            names: Vec::default(),
            samples: Vec::default(),
        };
        r.push_sample("Pluck", "stereo-pluck.wav".into());
        r.push_sample("Mellotron", "mellotron-woodwinds-c4.wav".into());
        r.push_sample("Vinyl Scratch", "vinyl-scratch.wav".into());
        r
    }
}
impl SampleLibrary {
    pub fn choices(&self) -> &[String] {
        &self.names
    }

    pub fn path(&self, index: SampleIndex) -> Option<PathBuf> {
        match index.0 {
            0 => Some(PathBuf::from("stereo-pluck.wav")),
            1 => Some(PathBuf::from("mellotron-woodwinds-c4.wav")),
            2 => Some(PathBuf::from("vinyl-scratch.wav")),
            _ => None,
        }
    }

    fn push_sample(&mut self, name: &str, path: PathBuf) {
        self.names.push(name.to_string());
        self.samples.push(SampleItem {
            name: name.to_string(),
            path,
        });
    }

    pub fn set_instance(instance: Self) {
        INSTANCE.set(instance);
    }

    pub(crate) fn global() -> &'static Self {
        INSTANCE.get().expect("SampleLibrary is not initialized")
    }
}

#[derive(Debug, Default, Copy, Clone, Display, Serialize, Deserialize, PartialEq)]
pub struct KitIndex(pub usize);
impl From<usize> for KitIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct KitItem {
    name: String,
    key: MidiNote,
    index: SampleIndex,
}
impl KitItem {
    fn new_with(name: &str, note: MidiNote, index: SampleIndex) -> Self {
        Self {
            name: name.to_string(),
            key: note,
            index,
        }
    }
}

#[derive(Debug)]
pub struct KitSampleLibrary {
    kits: Vec<Vec<KitItem>>,
}
impl Default for KitSampleLibrary {
    fn default() -> Self {
        Self {
            kits: vec![Self::build_707(), Self::build_808(), Self::build_909()],
        }
    }
}
impl KitSampleLibrary {
    pub fn choices(&self) -> &[&str] {
        &["707", "808", "909"]
    }

    pub fn contents(&self, index: KitIndex) -> Option<&[KitItem]> {
        match index.0 {
            0 => Some(&self.kits[0]),
            1 => Some(&self.kits[1]),
            2 => Some(&self.kits[2]),
            _ => None,
        }
    }

    fn build_707() -> Vec<KitItem> {
        let sample_index_base = 3;
        let names = vec!["Kick"];
        let programs = vec![GeneralMidiPercussionProgram::AcousticBassDrum];
        let v = names.iter().zip(programs.iter()).enumerate().fold(
            Vec::default(),
            |mut v, (i, (n, p))| {
                v.push(KitItem::new_with(
                    n,
                    (*p).into(),
                    SampleIndex(sample_index_base + i),
                ));
                v
            },
        );
        v
    }

    fn build_808() -> Vec<KitItem> {
        Vec::default()
    }

    fn build_909() -> Vec<KitItem> {
        Vec::default()
    }

    pub fn set_instance(instance: Self) {
        KIT_INSTANCE.set(instance);
    }

    pub(crate) fn global() -> &'static Self {
        KIT_INSTANCE.get().expect("KitLibrary is not initialized")
    }
}
