// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::time::MusicalTime;
use derive_more::Display as DeriveDisplay;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::FromRepr;

pub use midly::live::LiveEvent;
pub use midly::{
    num::{u4, u7},
    MidiMessage,
};
/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::midi::{
        new_note_off, new_note_on, u4, u7, MidiChannel, MidiEvent, MidiMessage, MidiNote,
    };
}

/// Newtype for MIDI channel.
#[derive(
    Clone, Copy, Debug, Default, DeriveDisplay, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub struct MidiChannel(pub u8);
#[allow(missing_docs)]
impl MidiChannel {
    pub const MIN_VALUE: u8 = 0;
    pub const MAX_VALUE: u8 = 15; // inclusive

    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}
impl From<u4> for MidiChannel {
    fn from(value: u4) -> Self {
        Self(value.as_int())
    }
}
impl From<u8> for MidiChannel {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MidiChannel> for u8 {
    fn from(value: MidiChannel) -> Self {
        value.0
    }
}

/// There are two different mappings of piano notes to MIDI numbers. They both
/// agree that Midi note 0 is a C, but they otherwise differ by an octave. I
/// originally picked C4=60, because that was the top Google search result's
/// answer, but it seems like a slight majority thinks C3=60. I'm going to leave
/// it as-is so that I don't have to rename my test data files. I don't think it
/// matters because we're not actually mapping these to anything user-visible.
///
/// A small disadvantage of C3=60 is that numbers 0-11 don't map to an easily
/// described octave. So I'm calling that octave "Sub0" because I needed
/// something in this enum.
///
/// These also correspond to
/// <https://en.wikipedia.org/wiki/Piano_key_frequencies>
//
// Generated with this Python code:
// ```
// #!/usr/bin/python
//
// NAMES = ["C", "Cs", "D", "Ds", "E", "F", "Fs", "G", "Gs", "A", "As", "B"]
//
// index = 12
// for i in range(0, 10):
//     for name in NAMES:
//         print("%s%d = %d," % (name, i, index))
//         index += 1
// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, FromRepr)]
pub enum MidiNote {
    CSub0 = 0,
    CsSub0 = 1,
    DSub0 = 2,
    DsSub0 = 3,
    ESub0 = 4,
    FSub0 = 5,
    FsSub0 = 6,
    GSub0 = 7,
    GsSub0 = 8,
    ASub0 = 9,
    AsSub0 = 10,
    BSub0 = 11,
    C0 = 12,
    Cs0 = 13,
    D0 = 14,
    Ds0 = 15,
    E0 = 16,
    F0 = 17,
    Fs0 = 18,
    G0 = 19,
    Gs0 = 20,
    A0 = 21,
    As0 = 22,
    B0 = 23,
    C1 = 24,
    Cs1 = 25,
    D1 = 26,
    Ds1 = 27,
    E1 = 28,
    F1 = 29,
    Fs1 = 30,
    G1 = 31,
    Gs1 = 32,
    A1 = 33,
    As1 = 34,
    B1 = 35,
    C2 = 36,
    Cs2 = 37,
    D2 = 38,
    Ds2 = 39,
    E2 = 40,
    F2 = 41,
    Fs2 = 42,
    G2 = 43,
    Gs2 = 44,
    A2 = 45,
    As2 = 46,
    B2 = 47,
    C3 = 48,
    Cs3 = 49,
    D3 = 50,
    Ds3 = 51,
    E3 = 52,
    F3 = 53,
    Fs3 = 54,
    G3 = 55,
    Gs3 = 56,
    A3 = 57,
    As3 = 58,
    B3 = 59,
    #[default]
    C4 = 60,
    Cs4 = 61,
    D4 = 62,
    Ds4 = 63,
    E4 = 64,
    F4 = 65,
    Fs4 = 66,
    G4 = 67,
    Gs4 = 68,
    A4 = 69,
    As4 = 70,
    B4 = 71,
    C5 = 72,
    Cs5 = 73,
    D5 = 74,
    Ds5 = 75,
    E5 = 76,
    F5 = 77,
    Fs5 = 78,
    G5 = 79,
    Gs5 = 80,
    A5 = 81,
    As5 = 82,
    B5 = 83,
    C6 = 84,
    Cs6 = 85,
    D6 = 86,
    Ds6 = 87,
    E6 = 88,
    F6 = 89,
    Fs6 = 90,
    G6 = 91,
    Gs6 = 92,
    A6 = 93,
    As6 = 94,
    B6 = 95,
    C7 = 96,
    Cs7 = 97,
    D7 = 98,
    Ds7 = 99,
    E7 = 100,
    F7 = 101,
    Fs7 = 102,
    G7 = 103,
    Gs7 = 104,
    A7 = 105,
    As7 = 106,
    B7 = 107,
    C8 = 108,
    Cs8 = 109,
    D8 = 110,
    Ds8 = 111,
    E8 = 112,
    F8 = 113,
    Fs8 = 114,
    G8 = 115,
    Gs8 = 116,
    A8 = 117,
    As8 = 118,
    B8 = 119,
    C9 = 120,
    Cs9 = 121,
    D9 = 122,
    Ds9 = 123,
    E9 = 124,
    F9 = 125,
    Fs9 = 126,
    G9 = 127,
}
#[allow(missing_docs)]
impl MidiNote {
    pub const MIN: MidiNote = Self::C0;
    pub const MAX: MidiNote = Self::G9;
}
impl Display for MidiNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.note_name();
        write!(f, "{}", s)
    }
}
impl MidiNote {
    // TODO # rather than #
    pub fn note_name_with_octave(&self) -> &str {
        match *self {
            MidiNote::CSub0 => "C-1",
            MidiNote::CsSub0 => "C#-1",
            MidiNote::DSub0 => "D-1",
            MidiNote::DsSub0 => "D#-1",
            MidiNote::ESub0 => "E-1",
            MidiNote::FSub0 => "F-1",
            MidiNote::FsSub0 => "F#-1",
            MidiNote::GSub0 => "G-1",
            MidiNote::GsSub0 => "G#-1",
            MidiNote::ASub0 => "A-1",
            MidiNote::AsSub0 => "A#-1",
            MidiNote::BSub0 => "B-1",
            MidiNote::C0 => "C0",
            MidiNote::Cs0 => "C#0",
            MidiNote::D0 => "D0",
            MidiNote::Ds0 => "D#0",
            MidiNote::E0 => "E0",
            MidiNote::F0 => "F0",
            MidiNote::Fs0 => "F#0",
            MidiNote::G0 => "G0",
            MidiNote::Gs0 => "G#0",
            MidiNote::A0 => "A0",
            MidiNote::As0 => "A#0",
            MidiNote::B0 => "B0",
            MidiNote::C1 => "C1",
            MidiNote::Cs1 => "C#1",
            MidiNote::D1 => "D1",
            MidiNote::Ds1 => "D#1",
            MidiNote::E1 => "E1",
            MidiNote::F1 => "F1",
            MidiNote::Fs1 => "F#1",
            MidiNote::G1 => "G1",
            MidiNote::Gs1 => "G#1",
            MidiNote::A1 => "A1",
            MidiNote::As1 => "A#1",
            MidiNote::B1 => "B1",
            MidiNote::C2 => "C2",
            MidiNote::Cs2 => "C#2",
            MidiNote::D2 => "D2",
            MidiNote::Ds2 => "D#2",
            MidiNote::E2 => "E2",
            MidiNote::F2 => "F2",
            MidiNote::Fs2 => "F#2",
            MidiNote::G2 => "G2",
            MidiNote::Gs2 => "G#2",
            MidiNote::A2 => "A2",
            MidiNote::As2 => "A#2",
            MidiNote::B2 => "B2",
            MidiNote::C3 => "C3",
            MidiNote::Cs3 => "C#3",
            MidiNote::D3 => "D3",
            MidiNote::Ds3 => "D#3",
            MidiNote::E3 => "E3",
            MidiNote::F3 => "F3",
            MidiNote::Fs3 => "F#3",
            MidiNote::G3 => "G3",
            MidiNote::Gs3 => "G#3",
            MidiNote::A3 => "A3",
            MidiNote::As3 => "A#3",
            MidiNote::B3 => "B3",
            MidiNote::C4 => "C4",
            MidiNote::Cs4 => "C#4",
            MidiNote::D4 => "D4",
            MidiNote::Ds4 => "D#4",
            MidiNote::E4 => "E4",
            MidiNote::F4 => "F4",
            MidiNote::Fs4 => "F#4",
            MidiNote::G4 => "G4",
            MidiNote::Gs4 => "G#4",
            MidiNote::A4 => "A4",
            MidiNote::As4 => "A#4",
            MidiNote::B4 => "B4",
            MidiNote::C5 => "C5",
            MidiNote::Cs5 => "C#5",
            MidiNote::D5 => "D5",
            MidiNote::Ds5 => "D#5",
            MidiNote::E5 => "E5",
            MidiNote::F5 => "F5",
            MidiNote::Fs5 => "F#5",
            MidiNote::G5 => "G5",
            MidiNote::Gs5 => "G#5",
            MidiNote::A5 => "A5",
            MidiNote::As5 => "A#5",
            MidiNote::B5 => "B5",
            MidiNote::C6 => "C6",
            MidiNote::Cs6 => "C#6",
            MidiNote::D6 => "D6",
            MidiNote::Ds6 => "D#6",
            MidiNote::E6 => "E6",
            MidiNote::F6 => "F6",
            MidiNote::Fs6 => "F#6",
            MidiNote::G6 => "G6",
            MidiNote::Gs6 => "G#6",
            MidiNote::A6 => "A6",
            MidiNote::As6 => "A#6",
            MidiNote::B6 => "B6",
            MidiNote::C7 => "C7",
            MidiNote::Cs7 => "C#7",
            MidiNote::D7 => "D7",
            MidiNote::Ds7 => "D#7",
            MidiNote::E7 => "E7",
            MidiNote::F7 => "F7",
            MidiNote::Fs7 => "F#7",
            MidiNote::G7 => "G7",
            MidiNote::Gs7 => "G#7",
            MidiNote::A7 => "A7",
            MidiNote::As7 => "A#7",
            MidiNote::B7 => "B7",
            MidiNote::C8 => "C8",
            MidiNote::Cs8 => "C#8",
            MidiNote::D8 => "D8",
            MidiNote::Ds8 => "D#8",
            MidiNote::E8 => "E8",
            MidiNote::F8 => "F8",
            MidiNote::Fs8 => "F#8",
            MidiNote::G8 => "G8",
            MidiNote::Gs8 => "G#8",
            MidiNote::A8 => "A8",
            MidiNote::As8 => "A#8",
            MidiNote::B8 => "B8",
            MidiNote::C9 => "C9",
            MidiNote::Cs9 => "C#9",
            MidiNote::D9 => "D9",
            MidiNote::Ds9 => "D#9",
            MidiNote::E9 => "E9",
            MidiNote::F9 => "F9",
            MidiNote::Fs9 => "F#9",
            MidiNote::G9 => "G9",
        }
    }

    pub fn note_name(&self) -> &str {
        match *self {
            MidiNote::CSub0 => "C",
            MidiNote::CsSub0 => "C#",
            MidiNote::DSub0 => "D",
            MidiNote::DsSub0 => "D#",
            MidiNote::ESub0 => "E",
            MidiNote::FSub0 => "F",
            MidiNote::FsSub0 => "F#",
            MidiNote::GSub0 => "G",
            MidiNote::GsSub0 => "G#",
            MidiNote::ASub0 => "A",
            MidiNote::AsSub0 => "A#",
            MidiNote::BSub0 => "B",
            MidiNote::C0 => "C",
            MidiNote::Cs0 => "C#",
            MidiNote::D0 => "D",
            MidiNote::Ds0 => "D#",
            MidiNote::E0 => "E",
            MidiNote::F0 => "F",
            MidiNote::Fs0 => "F#",
            MidiNote::G0 => "G",
            MidiNote::Gs0 => "G#",
            MidiNote::A0 => "A",
            MidiNote::As0 => "A#",
            MidiNote::B0 => "B",
            MidiNote::C1 => "C",
            MidiNote::Cs1 => "C#",
            MidiNote::D1 => "D",
            MidiNote::Ds1 => "D#",
            MidiNote::E1 => "E",
            MidiNote::F1 => "F",
            MidiNote::Fs1 => "F#",
            MidiNote::G1 => "G",
            MidiNote::Gs1 => "G#",
            MidiNote::A1 => "A",
            MidiNote::As1 => "A#",
            MidiNote::B1 => "B",
            MidiNote::C2 => "C",
            MidiNote::Cs2 => "C#",
            MidiNote::D2 => "D",
            MidiNote::Ds2 => "D#",
            MidiNote::E2 => "E",
            MidiNote::F2 => "F",
            MidiNote::Fs2 => "F#",
            MidiNote::G2 => "G",
            MidiNote::Gs2 => "G#",
            MidiNote::A2 => "A",
            MidiNote::As2 => "A#",
            MidiNote::B2 => "B",
            MidiNote::C3 => "C",
            MidiNote::Cs3 => "C#",
            MidiNote::D3 => "D",
            MidiNote::Ds3 => "D#",
            MidiNote::E3 => "E",
            MidiNote::F3 => "F",
            MidiNote::Fs3 => "F#",
            MidiNote::G3 => "G",
            MidiNote::Gs3 => "G#",
            MidiNote::A3 => "A",
            MidiNote::As3 => "A#",
            MidiNote::B3 => "B",
            MidiNote::C4 => "C",
            MidiNote::Cs4 => "C#",
            MidiNote::D4 => "D",
            MidiNote::Ds4 => "D#",
            MidiNote::E4 => "E",
            MidiNote::F4 => "F",
            MidiNote::Fs4 => "F#",
            MidiNote::G4 => "G",
            MidiNote::Gs4 => "G#",
            MidiNote::A4 => "A",
            MidiNote::As4 => "A#",
            MidiNote::B4 => "B",
            MidiNote::C5 => "C",
            MidiNote::Cs5 => "C#",
            MidiNote::D5 => "D",
            MidiNote::Ds5 => "D#",
            MidiNote::E5 => "E",
            MidiNote::F5 => "F",
            MidiNote::Fs5 => "F#",
            MidiNote::G5 => "G",
            MidiNote::Gs5 => "G#",
            MidiNote::A5 => "A",
            MidiNote::As5 => "A#",
            MidiNote::B5 => "B",
            MidiNote::C6 => "C",
            MidiNote::Cs6 => "C#",
            MidiNote::D6 => "D",
            MidiNote::Ds6 => "D#",
            MidiNote::E6 => "E",
            MidiNote::F6 => "F",
            MidiNote::Fs6 => "F#",
            MidiNote::G6 => "G",
            MidiNote::Gs6 => "G#",
            MidiNote::A6 => "A",
            MidiNote::As6 => "A#",
            MidiNote::B6 => "B",
            MidiNote::C7 => "C",
            MidiNote::Cs7 => "C#",
            MidiNote::D7 => "D",
            MidiNote::Ds7 => "D#",
            MidiNote::E7 => "E",
            MidiNote::F7 => "F",
            MidiNote::Fs7 => "F#",
            MidiNote::G7 => "G",
            MidiNote::Gs7 => "G#",
            MidiNote::A7 => "A",
            MidiNote::As7 => "A#",
            MidiNote::B7 => "B",
            MidiNote::C8 => "C",
            MidiNote::Cs8 => "C#",
            MidiNote::D8 => "D",
            MidiNote::Ds8 => "D#",
            MidiNote::E8 => "E",
            MidiNote::F8 => "F",
            MidiNote::Fs8 => "F#",
            MidiNote::G8 => "G",
            MidiNote::Gs8 => "G#",
            MidiNote::A8 => "A",
            MidiNote::As8 => "A#",
            MidiNote::B8 => "B",
            MidiNote::C9 => "C",
            MidiNote::Cs9 => "C#",
            MidiNote::D9 => "D",
            MidiNote::Ds9 => "D#",
            MidiNote::E9 => "E",
            MidiNote::F9 => "F",
            MidiNote::Fs9 => "F#",
            MidiNote::G9 => "G",
        }
    }

    pub fn octave(&self) -> i8 {
        match *self {
            MidiNote::CSub0 => -1,
            MidiNote::CsSub0 => -1,
            MidiNote::DSub0 => -1,
            MidiNote::DsSub0 => -1,
            MidiNote::ESub0 => -1,
            MidiNote::FSub0 => -1,
            MidiNote::FsSub0 => -1,
            MidiNote::GSub0 => -1,
            MidiNote::GsSub0 => -1,
            MidiNote::ASub0 => -1,
            MidiNote::AsSub0 => -1,
            MidiNote::BSub0 => -1,
            MidiNote::C0 => 0,
            MidiNote::Cs0 => 0,
            MidiNote::D0 => 0,
            MidiNote::Ds0 => 0,
            MidiNote::E0 => 0,
            MidiNote::F0 => 0,
            MidiNote::Fs0 => 0,
            MidiNote::G0 => 0,
            MidiNote::Gs0 => 0,
            MidiNote::A0 => 0,
            MidiNote::As0 => 0,
            MidiNote::B0 => 0,
            MidiNote::C1 => 1,
            MidiNote::Cs1 => 1,
            MidiNote::D1 => 1,
            MidiNote::Ds1 => 1,
            MidiNote::E1 => 1,
            MidiNote::F1 => 1,
            MidiNote::Fs1 => 1,
            MidiNote::G1 => 1,
            MidiNote::Gs1 => 1,
            MidiNote::A1 => 1,
            MidiNote::As1 => 1,
            MidiNote::B1 => 1,
            MidiNote::C2 => 2,
            MidiNote::Cs2 => 2,
            MidiNote::D2 => 2,
            MidiNote::Ds2 => 2,
            MidiNote::E2 => 2,
            MidiNote::F2 => 2,
            MidiNote::Fs2 => 2,
            MidiNote::G2 => 2,
            MidiNote::Gs2 => 2,
            MidiNote::A2 => 2,
            MidiNote::As2 => 2,
            MidiNote::B2 => 2,
            MidiNote::C3 => 3,
            MidiNote::Cs3 => 3,
            MidiNote::D3 => 3,
            MidiNote::Ds3 => 3,
            MidiNote::E3 => 3,
            MidiNote::F3 => 3,
            MidiNote::Fs3 => 3,
            MidiNote::G3 => 3,
            MidiNote::Gs3 => 3,
            MidiNote::A3 => 3,
            MidiNote::As3 => 3,
            MidiNote::B3 => 3,
            MidiNote::C4 => 4,
            MidiNote::Cs4 => 4,
            MidiNote::D4 => 4,
            MidiNote::Ds4 => 4,
            MidiNote::E4 => 4,
            MidiNote::F4 => 4,
            MidiNote::Fs4 => 4,
            MidiNote::G4 => 4,
            MidiNote::Gs4 => 4,
            MidiNote::A4 => 4,
            MidiNote::As4 => 4,
            MidiNote::B4 => 4,
            MidiNote::C5 => 5,
            MidiNote::Cs5 => 5,
            MidiNote::D5 => 5,
            MidiNote::Ds5 => 5,
            MidiNote::E5 => 5,
            MidiNote::F5 => 5,
            MidiNote::Fs5 => 5,
            MidiNote::G5 => 5,
            MidiNote::Gs5 => 5,
            MidiNote::A5 => 5,
            MidiNote::As5 => 5,
            MidiNote::B5 => 5,
            MidiNote::C6 => 6,
            MidiNote::Cs6 => 6,
            MidiNote::D6 => 6,
            MidiNote::Ds6 => 6,
            MidiNote::E6 => 6,
            MidiNote::F6 => 6,
            MidiNote::Fs6 => 6,
            MidiNote::G6 => 6,
            MidiNote::Gs6 => 6,
            MidiNote::A6 => 6,
            MidiNote::As6 => 6,
            MidiNote::B6 => 6,
            MidiNote::C7 => 7,
            MidiNote::Cs7 => 7,
            MidiNote::D7 => 7,
            MidiNote::Ds7 => 7,
            MidiNote::E7 => 7,
            MidiNote::F7 => 7,
            MidiNote::Fs7 => 7,
            MidiNote::G7 => 7,
            MidiNote::Gs7 => 7,
            MidiNote::A7 => 7,
            MidiNote::As7 => 7,
            MidiNote::B7 => 7,
            MidiNote::C8 => 8,
            MidiNote::Cs8 => 8,
            MidiNote::D8 => 8,
            MidiNote::Ds8 => 8,
            MidiNote::E8 => 8,
            MidiNote::F8 => 8,
            MidiNote::Fs8 => 8,
            MidiNote::G8 => 8,
            MidiNote::Gs8 => 8,
            MidiNote::A8 => 8,
            MidiNote::As8 => 8,
            MidiNote::B8 => 8,
            MidiNote::C9 => 9,
            MidiNote::Cs9 => 9,
            MidiNote::D9 => 9,
            MidiNote::Ds9 => 9,
            MidiNote::E9 => 9,
            MidiNote::F9 => 9,
            MidiNote::Fs9 => 9,
            MidiNote::G9 => 9,
        }
    }
}

/// Convenience function to make a note-on [MidiMessage].
pub fn new_note_on(note: u8, vel: u8) -> MidiMessage {
    MidiMessage::NoteOn {
        key: u7::from(note),
        vel: u7::from(vel),
    }
}

/// Convenience function to make a note-off [MidiMessage].
pub fn new_note_off(note: u8, vel: u8) -> MidiMessage {
    MidiMessage::NoteOff {
        key: u7::from(note),
        vel: u7::from(vel),
    }
}

/// The General MIDI instruments. https://en.wikipedia.org/wiki/General_MIDI
#[allow(missing_docs)]
#[derive(DeriveDisplay, Debug)]
pub enum GeneralMidiProgram {
    AcousticGrand = 0,
    BrightAcoustic = 1,
    ElectricGrand = 2,
    HonkyTonk = 3,
    ElectricPiano1 = 4,
    ElectricPiano2 = 5,
    Harpsichord = 6,
    Clav = 7,
    Celesta = 8,
    Glockenspiel = 9,
    MusicBox = 10,
    Vibraphone = 11,
    Marimba = 12,
    Xylophone = 13,
    TubularBells = 14,
    Dulcimer = 15,
    DrawbarOrgan = 16,
    PercussiveOrgan = 17,
    RockOrgan = 18,
    ChurchOrgan = 19,
    ReedOrgan = 20,
    Accordion = 21,
    Harmonica = 22,
    TangoAccordion = 23,
    AcousticGuitarNylon = 24,
    AcousticGuitarSteel = 25,
    ElectricGuitarJazz = 26,
    ElectricGuitarClean = 27,
    ElectricGuitarMuted = 28,
    OverdrivenGuitar = 29,
    DistortionGuitar = 30,
    GuitarHarmonics = 31,
    AcousticBass = 32,
    ElectricBassFinger = 33,
    ElectricBassPick = 34,
    FretlessBass = 35,
    SlapBass1 = 36,
    SlapBass2 = 37,
    SynthBass1 = 38,
    SynthBass2 = 39,
    Violin = 40,
    Viola = 41,
    Cello = 42,
    Contrabass = 43,
    TremoloStrings = 44,
    PizzicatoStrings = 45,
    OrchestralHarp = 46,
    Timpani = 47,
    StringEnsemble1 = 48,
    StringEnsemble2 = 49,
    Synthstrings1 = 50,
    Synthstrings2 = 51,
    ChoirAahs = 52,
    VoiceOohs = 53,
    SynthVoice = 54,
    OrchestraHit = 55,
    Trumpet = 56,
    Trombone = 57,
    Tuba = 58,
    MutedTrumpet = 59,
    FrenchHorn = 60,
    BrassSection = 61,
    Synthbrass1 = 62,
    Synthbrass2 = 63,
    SopranoSax = 64,
    AltoSax = 65,
    TenorSax = 66,
    BaritoneSax = 67,
    Oboe = 68,
    EnglishHorn = 69,
    Bassoon = 70,
    Clarinet = 71,
    Piccolo = 72,
    Flute = 73,
    Recorder = 74,
    PanFlute = 75,
    BlownBottle = 76,
    Shakuhachi = 77,
    Whistle = 78,
    Ocarina = 79,
    Lead1Square = 80,
    Lead2Sawtooth = 81,
    Lead3Calliope = 82,
    Lead4Chiff = 83,
    Lead5Charang = 84,
    Lead6Voice = 85,
    Lead7Fifths = 86,
    Lead8BassLead = 87,
    Pad1NewAge = 88,
    Pad2Warm = 89,
    Pad3Polysynth = 90,
    Pad4Choir = 91,
    Pad5Bowed = 92,
    Pad6Metallic = 93,
    Pad7Halo = 94,
    Pad8Sweep = 95,
    Fx1Rain = 96,
    Fx2Soundtrack = 97,
    Fx3Crystal = 98,
    Fx4Atmosphere = 99,
    Fx5Brightness = 100,
    Fx6Goblins = 101,
    Fx7Echoes = 102,
    Fx8SciFi = 103,
    Sitar = 104,
    Banjo = 105,
    Shamisen = 106,
    Koto = 107,
    Kalimba = 108,
    Bagpipe = 109,
    Fiddle = 110,
    Shanai = 111,
    TinkleBell = 112,
    Agogo = 113,
    SteelDrums = 114,
    Woodblock = 115,
    TaikoDrum = 116,
    MelodicTom = 117,
    SynthDrum = 118,
    ReverseCymbal = 119,
    GuitarFretNoise = 120,
    BreathNoise = 121,
    Seashore = 122,
    BirdTweet = 123,
    TelephoneRing = 124,
    Helicopter = 125,
    Applause = 126,
    Gunshot = 127,
}

/// The General MIDI percussion instruments. https://en.wikipedia.org/wiki/General_MIDI#Percussion
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum GeneralMidiPercussionProgram {
    AcousticBassDrum = 35,
    ElectricBassDrum = 36,
    SideStick = 37,
    AcousticSnare = 38,
    HandClap = 39,
    ElectricSnare = 40,
    LowFloorTom = 41,
    ClosedHiHat = 42,
    HighFloorTom = 43,
    PedalHiHat = 44,
    LowTom = 45,
    OpenHiHat = 46,
    LowMidTom = 47,
    HiMidTom = 48,
    CrashCymbal1 = 49,
    HighTom = 50,
    RideCymbal1 = 51,
    ChineseCymbal = 52,
    RideBell = 53,
    Tambourine = 54,
    SplashCymbal = 55,
    Cowbell = 56,
    CrashCymbal2 = 57,
    Vibraslap = 58,
    RideCymbal2 = 59,
    HighBongo = 60,
    LowBongo = 61,
    MuteHighConga = 62,
    OpenHighConga = 63,
    LowConga = 64,
    HighTimbale = 65,
    LowTimbale = 66,
    HighAgogo = 67,
    LowAgogo = 68,
    Cabasa = 69,
    Maracas = 70,
    ShortWhistle = 71,
    LongWhistle = 72,
    ShortGuiro = 73,
    LongGuiro = 74,
    Claves = 75,
    HighWoodblock = 76,
    LowWoodblock = 77,
    MuteCuica = 78,
    OpenCuica = 79,
    MuteTriangle = 80,
    OpenTriangle = 81,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MidiEvent {
    pub message: MidiMessage,
    pub time: MusicalTime,
}

#[cfg(obsolete)]
mod obsolete {
    /// [MidiSmfReader] parses MIDI SMF files and programs [MidiTickSequencer] with
    /// the data it finds.
    pub struct MidiSmfReader {}
    impl MidiSmfReader {
        pub fn program_sequencer(sequencer: &mut MidiTickSequencer, data: &[u8]) {
            let parse_result = midly::Smf::parse(data).unwrap();

            struct MetaInfo {
                // Pulses per quarter-note
                ppq: u32,

                // Microseconds per quarter-note
                tempo: u32,

                time_signature_numerator: u8,
                time_signature_denominator_exp: u8,
            }
            let mut meta_info = MetaInfo {
                ppq: match parse_result.header.timing {
                    midly::Timing::Metrical(ticks_per_beat) => ticks_per_beat.as_int() as u32,
                    _ => 0,
                },
                tempo: 0,

                // https://en.wikipedia.org/wiki/Time_signature
                time_signature_numerator: 0,
                time_signature_denominator_exp: 0,
            };
            for (track_number, track) in parse_result.tracks.iter().enumerate() {
                println!("Processing track {track_number}");
                let mut track_time_ticks: usize = 0; // The relative time references start over at zero with each track.

                for t in track.iter() {
                    match t.kind {
                        TrackEventKind::Midi { channel, message } => {
                            let delta = t.delta.as_int() as usize;
                            track_time_ticks += delta;
                            sequencer.insert(MidiTicks(track_time_ticks), channel.into(), message);
                            // TODO: prior version of this code treated vel=0 as
                            // note-off. Do we need to handle that higher up?
                        }

                        TrackEventKind::Meta(meta_message) => match meta_message {
                            midly::MetaMessage::TimeSignature(
                                numerator,
                                denominator_exp,
                                _cc,
                                _bb,
                            ) => {
                                meta_info.time_signature_numerator = numerator;
                                meta_info.time_signature_denominator_exp = denominator_exp;
                                //meta_info.ppq = cc; WHA???
                            }
                            midly::MetaMessage::Tempo(tempo) => {
                                meta_info.tempo = tempo.as_int();
                            }
                            midly::MetaMessage::TrackNumber(track_opt) => {
                                if track_opt.is_none() {
                                    continue;
                                }
                            }
                            midly::MetaMessage::EndOfTrack => {
                                let _time_signature: (u32, u32) = (
                                    meta_info.time_signature_numerator.into(),
                                    2_u32.pow(meta_info.time_signature_denominator_exp.into()),
                                );
                                let ticks_per_quarter_note: f32 = meta_info.ppq as f32;
                                let seconds_per_quarter_note: f32 =
                                    meta_info.tempo as f32 / 1000000.0;
                                let _ticks_per_second =
                                    ticks_per_quarter_note / seconds_per_quarter_note;

                                let _bpm: f32 = (60.0 * 1000000.0) / (meta_info.tempo as f32);

                                // sequencer.set_midi_ticks_per_second(ticks_per_second
                                // as usize);
                            }
                            _ => {}
                        },
                        TrackEventKind::SysEx(_data) => { // TODO
                        }
                        TrackEventKind::Escape(_data) => { // TODO
                        }
                    }
                }
            }
            println!("Done processing MIDI file");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn midi_note_is_complete() {
        for key in 0..127 {
            assert_eq!(MidiNote::from_repr(key).unwrap() as usize, key);
        }
    }

    #[test]
    fn note_to_frequency() {
        // https://www.colincrawley.com/midi-note-to-audio-frequency-calculator/
        assert_eq!(
            FrequencyHz::from(MidiNote::C0),
            16.351_597_831_287_414.into()
        );
        assert_eq!(
            FrequencyHz::from(MidiNote::C4),
            261.625_565_300_598_6.into()
        );
        assert_eq!(
            FrequencyHz::from(MidiNote::D5),
            587.329_535_834_815_1.into()
        );
        assert_eq!(
            FrequencyHz::from(MidiNote::D6),
            1_174.659_071_669_630_3.into()
        );
        assert_eq!(
            FrequencyHz::from(MidiNote::G9),
            12_543.853_951_415_975.into()
        );
    }
}
