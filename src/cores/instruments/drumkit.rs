// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::sampler::{SamplerCore, SamplerVoice};
use crate::{
    elements::VoicePerNoteStore,
    midi::{prelude::*, GeneralMidiPercussionProgram},
    prelude::*,
    util::Paths,
};
use anyhow::anyhow;
use delegate::delegate;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};

#[derive(Control, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DrumkitCore {
    name: String,

    #[serde(skip)]
    paths: Paths,
    #[serde(skip)]
    inner_synth: Synthesizer<SamplerVoice>,
}
impl std::fmt::Debug for DrumkitCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drumkit")
            .field("name", &self.name)
            .field("paths", &self.paths)
            .finish()
    }
}

impl Generates<StereoSample> for DrumkitCore {
    fn value(&self) -> StereoSample {
        self.inner_synth.value()
    }

    fn generate(&mut self, values: &mut [StereoSample]) {
        self.inner_synth.generate(values);
    }
}
impl Serializable for DrumkitCore {}
impl Configurable for DrumkitCore {
    delegate! {
        to self.inner_synth {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl Ticks for DrumkitCore {
    fn tick(&mut self, tick_count: usize) {
        self.inner_synth.tick(tick_count);
    }
}
impl HandlesMidi for DrumkitCore {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
        self.inner_synth
            .handle_midi_message(channel, message, midi_messages_fn)
    }
}
impl DrumkitCore {
    fn new_from_files(paths: &Paths, kit_name: &str) -> Self {
        let samples = vec![
            (GeneralMidiPercussionProgram::AcousticBassDrum, "Kick 1 R1"),
            (GeneralMidiPercussionProgram::ElectricBassDrum, "Kick 2 R1"),
            (GeneralMidiPercussionProgram::ClosedHiHat, "Hat Closed R1"),
            (GeneralMidiPercussionProgram::PedalHiHat, "Hat Closed R2"),
            (GeneralMidiPercussionProgram::HandClap, "Clap R1"),
            (GeneralMidiPercussionProgram::RideBell, "Cowbell R1"),
            (GeneralMidiPercussionProgram::CrashCymbal1, "Crash R1"),
            (GeneralMidiPercussionProgram::CrashCymbal2, "Crash R2"),
            (GeneralMidiPercussionProgram::OpenHiHat, "Hat Open R1"),
            (GeneralMidiPercussionProgram::RideCymbal1, "Ride R1"),
            (GeneralMidiPercussionProgram::RideCymbal2, "Ride R2"),
            (GeneralMidiPercussionProgram::SideStick, "Rim R1"),
            (GeneralMidiPercussionProgram::AcousticSnare, "Snare 1 R1"),
            (GeneralMidiPercussionProgram::ElectricSnare, "Snare 2 R1"),
            (GeneralMidiPercussionProgram::Tambourine, "Tambourine R1"),
            (GeneralMidiPercussionProgram::LowTom, "Tom 1 R1"),
            (GeneralMidiPercussionProgram::LowMidTom, "Tom 1 R2"),
            (GeneralMidiPercussionProgram::HiMidTom, "Tom 2 R1"),
            (GeneralMidiPercussionProgram::HighTom, "Tom 3 R1"),
            (GeneralMidiPercussionProgram::HighAgogo, "Cowbell R3"),
            (GeneralMidiPercussionProgram::LowAgogo, "Cowbell R4"),
        ];

        let sample_dirs = vec!["elphnt.io", "707"];

        let voice_store = VoicePerNoteStore::<SamplerVoice>::new_with_voices(
            samples.into_iter().flat_map(|(program, asset_name)| {
                let filename =
                    paths.build_sample(&sample_dirs, Path::new(&format!("{asset_name}.wav")));
                if let Ok(file) = paths.search_and_open(filename.as_path()) {
                    if let Ok(samples) = SamplerCore::read_samples_from_file(&file) {
                        let program = program as u8;
                        Ok((
                            u7::from(program),
                            SamplerVoice::new_with_samples(
                                Arc::new(samples),
                                MidiNote::from_repr(program as usize).unwrap().into(),
                            ),
                        ))
                    } else {
                        Err(anyhow!("Unable to load sample from file {:?}.", filename))
                    }
                } else {
                    Err(anyhow!("Couldn't find filename {:?} in hives", filename))
                }
            }),
        );

        Self {
            inner_synth: Synthesizer::<SamplerVoice>::new_with(Box::new(voice_store)),
            paths: paths.clone(),
            name: kit_name.to_string(),
        }
    }

    pub fn new_with(name: &str, paths: &Paths) -> Self {
        // TODO: we're hardcoding samples/. Figure out a way to use the
        // system.
        Self::new_from_files(paths, name)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}
