// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Provides a programmatic way to load music samples.

use crate::midi::{GeneralMidiPercussionCode, MidiNote};
use derive_more::Display;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

static INSTANCE: OnceCell<SampleLibrary> = OnceCell::new();
static KIT_INSTANCE: OnceCell<KitLibrary> = OnceCell::new();

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

        // Initial random set
        for (name, path) in [
            ("Pluck", "stereo-pluck.wav"),
            ("Mellotron", "mellotron-woodwinds-c4.wav"),
            ("Vinyl Scratch", "vinyl-scratch.wav"),
        ] {
            r.push_sample(name, None, path.into());
        }

        // 707
        for (name, path) in [
            ("Cowbell-r2", "Cowbell R2.wav"),
            ("Crash-r1", "Crash R1.wav"),
            ("Tom-2-r1", "Tom 2 R1.wav"),
            ("Tambourine-r1", "Tambourine R1.wav"),
            ("Tom-3-r1", "Tom 3 R1.wav"),
            ("Kick-2-r3", "Kick 2 R3.wav"),
            ("Rim-r3", "Rim R3.wav"),
            ("Hat-closed-r1", "Hat Closed R1.wav"),
            ("Tambourine-r4", "Tambourine R4.wav"),
            ("Snare-1-r3", "Snare 1 R3.wav"),
            ("Kick-2-r1", "Kick 2 R1.wav"),
            ("Crash-r3", "Crash R3.wav"),
            ("Hat-open-r4", "Hat Open R4.wav"),
            ("Hat-open-r1", "Hat Open R1.wav"),
            ("Hat-closed-r2", "Hat Closed R2.wav"),
            ("Ride-r2", "Ride R2.wav"),
            ("Tom-2-r4", "Tom 2 R4.wav"),
            ("Tom-1-r2", "Tom 1 R2.wav"),
            ("Crash-r4", "Crash R4.wav"),
            ("Cowbell-r4", "Cowbell R4.wav"),
            ("Ride-r1", "Ride R1.wav"),
            ("Snare-1-r2", "Snare 1 R2.wav"),
            ("Clap-r2", "Clap R2.wav"),
            ("Clap-r3", "Clap R3.wav"),
            ("Hat-open-r3", "Hat Open R3.wav"),
            ("Kick-2-r4", "Kick 2 R4.wav"),
            ("Clap-r1", "Clap R1.wav"),
            ("Tambourine-r3", "Tambourine R3.wav"),
            ("Tambourine-r2", "Tambourine R2.wav"),
            ("Ride-r4", "Ride R4.wav"),
            ("Cowbell-r3", "Cowbell R3.wav"),
            ("Tom-3-r2", "Tom 3 R2.wav"),
            ("Rim-r2", "Rim R2.wav"),
            ("Rim-r1", "Rim R1.wav"),
            ("Tom-1-r1", "Tom 1 R1.wav"),
            ("Tom-2-r2", "Tom 2 R2.wav"),
            ("Tom-3-r3", "Tom 3 R3.wav"),
            ("Kick-1-r3", "Kick 1 R3.wav"),
            ("Snare-1-r4", "Snare 1 R4.wav"),
            ("Tom-2-r3", "Tom 2 R3.wav"),
            ("Hat-closed-r4", "Hat Closed R4.wav"),
            ("Snare-2-r4", "Snare 2 R4.wav"),
            ("Kick-1-r4", "Kick 1 R4.wav"),
            ("Snare-2-r2", "Snare 2 R2.wav"),
            ("Kick-1-r1", "Kick 1 R1.wav"),
            ("Kick-1-r2", "Kick 1 R2.wav"),
            ("Hat-open-r2", "Hat Open R2.wav"),
            ("Crash-r2", "Crash R2.wav"),
            ("Hat-closed-r3", "Hat Closed R3.wav"),
            ("Tom-1-r4", "Tom 1 R4.wav"),
            ("Ride-r3", "Ride R3.wav"),
            ("Clap-r4", "Clap R4.wav"),
            ("Tom-1-r3", "Tom 1 R3.wav"),
            ("Rim-r4", "Rim R4.wav"),
            ("Kick-2-r2", "Kick 2 R2.wav"),
            ("Snare-2-r3", "Snare 2 R3.wav"),
            ("Tom-3-r4", "Tom 3 R4.wav"),
            ("Cowbell-r1", "Cowbell R1.wav"),
            ("Snare-1-r1", "Snare 1 R1.wav"),
            ("Snare-2-r1", "Snare 2 R1.wav"),
        ] {
            r.push_sample(name, Some("elphnt.io/707".into()), path.into());
        }

        r
    }
}
impl SampleLibrary {
    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn path(&self, index: SampleIndex) -> Option<PathBuf> {
        let index = index.0;
        if index < self.samples.len() {
            Some(self.samples[index].path.clone())
        } else {
            None
        }
    }

    fn push_sample(&mut self, name: &str, prefix: Option<PathBuf>, path: PathBuf) {
        let path = if let Some(prefix) = prefix {
            prefix.join(path)
        } else {
            path
        };
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
impl KitIndex {
    pub const KIT_707: KitIndex = KitIndex(0);
}
impl From<usize> for KitIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct KitItem {
    pub(crate) name: String,
    pub(crate) key: MidiNote,
    pub(crate) index: SampleIndex,
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
pub struct Kit {
    pub name: String,
    pub library_offset: usize,
    pub items: Vec<KitItem>,
}

#[derive(Debug)]
pub struct KitLibrary {
    names: Vec<String>,
    kits: Vec<Kit>,
}
impl Default for KitLibrary {
    fn default() -> Self {
        let mut r: Self = Self {
            names: Default::default(),
            kits: Default::default(),
        };
        r.build_707();
        r.build_808();
        r.build_909();
        r
    }
}
impl KitLibrary {
    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn kit(&self, index: KitIndex) -> Option<&Kit> {
        let index = index.0;
        if index < self.kits.len() {
            Some(&self.kits[index])
        } else {
            None
        }
    }

    fn build_707(&mut self) {
        let sample_index_base = 3;
        let programs = vec![
            (GeneralMidiPercussionCode::AcousticBassDrum, "Kick 1 R1"),
            (GeneralMidiPercussionCode::ElectricBassDrum, "Kick 2 R1"),
            (GeneralMidiPercussionCode::ClosedHiHat, "Hat Closed R1"),
            (GeneralMidiPercussionCode::PedalHiHat, "Hat Closed R2"),
            (GeneralMidiPercussionCode::HandClap, "Clap R1"),
            (GeneralMidiPercussionCode::RideBell, "Cowbell R1"),
            (GeneralMidiPercussionCode::CrashCymbal1, "Crash R1"),
            (GeneralMidiPercussionCode::CrashCymbal2, "Crash R2"),
            (GeneralMidiPercussionCode::OpenHiHat, "Hat Open R1"),
            (GeneralMidiPercussionCode::RideCymbal1, "Ride R1"),
            (GeneralMidiPercussionCode::RideCymbal2, "Ride R2"),
            (GeneralMidiPercussionCode::SideStick, "Rim R1"),
            (GeneralMidiPercussionCode::AcousticSnare, "Snare 1 R1"),
            (GeneralMidiPercussionCode::ElectricSnare, "Snare 2 R1"),
            (GeneralMidiPercussionCode::Tambourine, "Tambourine R1"),
            (GeneralMidiPercussionCode::LowTom, "Tom 1 R1"),
            (GeneralMidiPercussionCode::LowMidTom, "Tom 1 R2"),
            (GeneralMidiPercussionCode::HiMidTom, "Tom 2 R1"),
            (GeneralMidiPercussionCode::HighTom, "Tom 3 R1"),
            (GeneralMidiPercussionCode::HighAgogo, "Cowbell R3"),
            (GeneralMidiPercussionCode::LowAgogo, "Cowbell R4"),
        ];
        let items =
            programs
                .iter()
                .enumerate()
                .fold(Vec::default(), |mut v, (i, (program, name))| {
                    v.push(KitItem::new_with(
                        name,
                        (*program).into(),
                        SampleIndex(sample_index_base + i),
                    ));
                    v
                });
        let name = String::from("707");
        self.names.push(name.clone());
        self.kits.push(Kit {
            name,
            library_offset: 3,
            items,
        });
    }

    fn build_808(&mut self) {}

    fn build_909(&mut self) {}

    pub fn set_instance(instance: Self) {
        KIT_INSTANCE.set(instance);
    }

    pub(crate) fn global() -> &'static Self {
        KIT_INSTANCE.get().expect("KitLibrary is not initialized")
    }
}
