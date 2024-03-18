// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Provides a programmatic way to load music samples.

use crate::midi::MidiNote;
use derive_more::Display;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

static INSTANCE: OnceCell<SampleLibrary> = OnceCell::new();
static KIT_INSTANCE: OnceCell<KitLibrary> = OnceCell::new();

pub fn init_sample_libraries() {
    let mut sample_library = SampleLibrary::default();
    let kit_library = KitLibrary::new_with(&mut sample_library);
    SampleLibrary::set_instance(sample_library);
    KitLibrary::set_instance(kit_library);
}

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

        // // 808
        // for (name, path) in [
        // ] {
        //     r.push_sample(name, Some("drumkits/808".into()), path.into());
        // }

        // // 909
        // for (name, path) in [
        // ] {
        //     r.push_sample(name, Some("drumkits/909".into()), path.into());
        // }

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

    fn push_sample(&mut self, name: &str, prefix: Option<&Path>, path: PathBuf) -> SampleIndex {
        let path = if let Some(prefix) = prefix {
            prefix.join(path)
        } else {
            path
        };
        let index = self.samples.len().into();
        self.names.push(name.to_string());
        self.samples.push(SampleItem {
            name: name.to_string(),
            path,
        });
        index
    }

    pub fn set_instance(instance: Self) {
        INSTANCE.set(instance);
    }

    pub fn global() -> &'static Self {
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
    pub(crate) note: MidiNote,
    pub(crate) index: SampleIndex,
}
impl KitItem {
    fn new_with(name: &str, note: MidiNote, index: SampleIndex) -> Self {
        Self {
            name: name.to_string(),
            note,
            index,
        }
    }
}

#[derive(Debug)]
pub struct Kit {
    pub name: String,
    pub items: Vec<KitItem>,
}

#[derive(Debug)]
pub struct KitLibrary {
    names: Vec<String>,
    kits: Vec<Kit>,
}
impl KitLibrary {
    const CLAP: &'static str = "Hand Clap";
    const CLAVES: &'static str = "Claves";
    const CONGA_HIGH: &'static str = "High Conga";
    const CONGA_LOW: &'static str = "Low Conga";
    const CONGA_MEDIUM: &'static str = "Medium Conga";
    const COWBELL: &'static str = "Cowbell";
    const CYMBAL_CRASH: &'static str = "Cymbal Crash";
    const CYMBAL_RIDE: &'static str = "Cymbal Ride";
    const HI_HAT_CLOSED: &'static str = "Closed Hat";
    const HI_HAT_CLOSED_TO_OPEN: &'static str = "Closed-to-Open Hat";
    const HI_HAT_OPEN: &'static str = "Open Hat";
    const HI_HAT_OPEN_TO_CLOSED: &'static str = "Open-to-Closed Hat";
    const KICK: &'static str = "Kick";
    const KICK_1: &'static str = "Kick 1";
    const KICK_2: &'static str = "Kick 2";
    const MARACA: &'static str = "Maraca";
    const RIMSHOT: &'static str = "Rimshot";
    const SNARE: &'static str = "Snare";
    const SNARE_1: &'static str = "Snare 1";
    const SNARE_2: &'static str = "Snare 2";
    const TAMBOURINE: &'static str = "Tambourine";
    const TOM_HIGH: &'static str = "High Tom";
    const TOM_LOW: &'static str = "Low Tom";
    const TOM_MEDIUM: &'static str = "Med Tom";

    /// Where drumkits usually begin on the MIDI note scale.
    const MIDI_NOTE_BASE: usize = MidiNote::C2 as usize;

    pub fn new_with(sample_library: &mut SampleLibrary) -> Self {
        let mut r: Self = Self {
            names: Default::default(),
            kits: Default::default(),
        };
        r.build_707(sample_library);
        r.build_808(sample_library);
        r.build_909(sample_library);
        r
    }

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

    fn build_kit(
        &mut self,
        sample_library: &mut SampleLibrary,
        kit_name: &str,
        path_prefix: &Path,
        kit_items: &[(&str, &str)],
    ) {
        let items =
            kit_items
                .iter()
                .enumerate()
                .fold(Vec::default(), |mut v, (i, (name, path))| {
                    let library_index = sample_library.push_sample(
                        &format!("{}-{}", kit_name, name),
                        Some(path_prefix.into()),
                        path.into(),
                    );
                    v.push(KitItem {
                        name: name.to_string(),
                        note: MidiNote::from(Self::MIDI_NOTE_BASE + i),
                        index: library_index,
                    });
                    v
                });

        self.names.push(kit_name.to_string());
        self.kits.push(Kit {
            name: kit_name.to_string(),
            items,
        });
    }

    fn build_707(&mut self, sample_library: &mut SampleLibrary) {
        self.build_kit(
            sample_library,
            "707",
            &Path::new("drumkits/707"),
            &[
                (Self::KICK_1, "Kick 1 R1.wav"),
                (Self::KICK_2, "Kick 2 R1.wav"),
                (Self::SNARE_1, "Snare 1 R1.wav"),
                (Self::SNARE_2, "Snare 2 R1.wav"),
                (Self::TOM_LOW, "Tom 1 R1.wav"),
                (Self::TOM_MEDIUM, "Tom 2 R1.wav"),
                (Self::TOM_HIGH, "Tom 3 R1.wav"),
                (Self::RIMSHOT, "Rim R1.wav"),
                (Self::COWBELL, "Cowbell R1.wav"),
                (Self::CLAP, "Clap R1.wav"),
                (Self::TAMBOURINE, "Tambourine R1.wav"),
                (Self::HI_HAT_CLOSED, "Hat Closed R1.wav"),
                (Self::HI_HAT_OPEN, "Hat Open R1.wav"),
                (Self::CYMBAL_CRASH, "Crash R1.wav"),
                (Self::CYMBAL_RIDE, "Ride R1.wav"),
            ],
        );
    }

    fn build_808(&mut self, sample_library: &mut SampleLibrary) {
        self.build_kit(
            sample_library,
            "808",
            &Path::new("drumkits/808"),
            &[
                (Self::KICK_1, "BD0025.WAV"),
                (Self::KICK_2, "BD0050.WAV"),
                (Self::SNARE, "SD0010.WAV"),
                (Self::TOM_LOW, "LT00.WAV"),
                (Self::TOM_MEDIUM, "MT00.WAV"),
                (Self::TOM_HIGH, "HT00.WAV"),
                (Self::RIMSHOT, "RS.WAV"),
                (Self::COWBELL, "CB.WAV"),
                (Self::CLAP, "CP.WAV"),
                (Self::MARACA, "MA.WAV"),
                (Self::CLAVES, "CL.WAV"),
                (Self::HI_HAT_CLOSED, "CH.WAV"),
                (Self::HI_HAT_OPEN, "OH00.WAV"),
                (Self::CYMBAL_CRASH, "CY0050.WAV"),
                (Self::CONGA_LOW, "LC00.WAV"),
                (Self::CONGA_MEDIUM, "MC00.WAV"),
                (Self::CONGA_HIGH, "HC00.WAV"),
            ],
        );
    }

    fn build_909(&mut self, sample_library: &mut SampleLibrary) {
        self.build_kit(
            sample_library,
            "909",
            &Path::new("drumkits/909"),
            &[
                (Self::KICK, "BT0A0D3.WAV"),
                (Self::SNARE, "ST0T0S7.WAV"),
                (Self::TOM_LOW, "LT0DA.WAV"),
                (Self::TOM_MEDIUM, "MT0DA.WAV"),
                (Self::TOM_HIGH, "HT0DA.WAV"),
                (Self::RIMSHOT, "RIM63.WAV"),
                (Self::CLAP, "HANDCLP2.WAV"),
                (Self::HI_HAT_CLOSED, "HHCDA.WAV"),
                (Self::HI_HAT_OPEN, "HHODA.WAV"),
                (Self::HI_HAT_CLOSED_TO_OPEN, "CLOP4.WAV"),
                (Self::HI_HAT_OPEN_TO_CLOSED, "OPCL1.WAV"),
                (Self::CYMBAL_CRASH, "CSHD2.WAV"),
                (Self::CYMBAL_RIDE, "RIDED2.WAV"),
            ],
        );
    }

    pub fn set_instance(instance: Self) {
        KIT_INSTANCE.set(instance);
    }

    pub(crate) fn global() -> &'static Self {
        KIT_INSTANCE.get().expect("KitLibrary is not initialized")
    }
}
