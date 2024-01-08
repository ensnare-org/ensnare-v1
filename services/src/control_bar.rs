// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_cores_egui::widgets::audio::CircularSampleBuffer;

#[derive(Debug, Default)]
pub enum ControlBarDisplayMode {
    #[default]
    Time,
    Frequency,
}

/// [ControlBar] is the UI component at the top of the main window. Transport,
/// MIDI status, etc.
#[derive(Debug, Default)]
pub struct ControlBar {
    pub saw_midi_in_activity: bool,
    pub saw_midi_out_activity: bool,
    pub sample_buffer: CircularSampleBuffer,
    pub sample_channels: ChannelPair<[Sample; 64]>,
    pub display_mode: ControlBarDisplayMode,
    pub fft_buffer: Vec<f32>,
}
impl ControlBar {
    /// Tell [ControlBar] that the system just saw an incoming MIDI message.
    pub fn tickle_midi_in(&mut self) {
        self.saw_midi_in_activity = true;
    }

    /// Tell [ControlPanel] that the system just produced an outgoing MIDI message.
    pub fn tickle_midi_out(&mut self) {
        self.saw_midi_out_activity = true;
    }
}
