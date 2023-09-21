// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub use midly::live::LiveEvent;
pub use midly::{
    num::{u4, u7},
    MidiMessage,
};

use derive_more::Display as DeriveDisplay;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, DeriveDisplay, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct MidiChannel(pub u8);
impl MidiChannel {
    pub const MAX: u8 = 16;

    pub const fn value(&self) -> u8 {
        self.0
    }

    pub const fn new(value: u8) -> Self {
        Self { 0: value }
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

pub type MidiMessagesFn<'a> = dyn FnMut(MidiChannel, MidiMessage) + 'a;

/// Takes standard MIDI messages. Implementers can ignore MidiChannel if it's
/// not important, as the virtual cabling model tries to route only relevant
/// traffic to individual devices.
pub trait HandlesMidi {
    #[allow(unused_variables)]
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
    }
}
