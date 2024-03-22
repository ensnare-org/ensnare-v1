// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::sampler::{SamplerCore, SamplerVoice};
use crate::{
    elements::VoicePerNoteStore,
    midi::prelude::*,
    prelude::*,
    traits::MidiNoteLabelMetadata,
    util::{
        library::{KitIndex, KitLibrary},
        Paths,
    },
};
use anyhow::anyhow;
use delegate::delegate;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Control, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DrumkitCore {
    kit_index: KitIndex,

    name: String,

    #[serde(skip)]
    inner_synth: Synthesizer<SamplerVoice>,
}
impl core::fmt::Debug for DrumkitCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drumkit")
            .field("index", &self.kit_index)
            .field("name", &self.name)
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

    fn midi_note_label_metadata(&self) -> Option<MidiNoteLabelMetadata> {
        if let Some(kit) = KitLibrary::global().kit(self.kit_index) {
            let mut note_start = MidiNote::MAX;
            let mut note_end = MidiNote::MIN;
            let labels = kit.items.iter().fold(Vec::default(), |mut v, item| {
                v.push(item.name.clone());
                if note_start > item.note {
                    note_start = item.note;
                }
                if note_end < item.note {
                    note_end = item.note;
                }
                v
            });
            Some(MidiNoteLabelMetadata {
                range: note_start..=note_end,
                labels,
            })
        } else {
            None
        }
    }
}
impl DrumkitCore {
    pub fn new_with_kit_index(kit_index: KitIndex) -> Self {
        let voice_store = VoicePerNoteStore::<SamplerVoice>::new_with_voices(
            Vec::<(midly::num::u7, SamplerVoice)>::default().into_iter(),
        );

        let mut r = Self {
            kit_index,
            name: "Unknown".into(),
            inner_synth: Synthesizer::<SamplerVoice>::new_with(Box::new(voice_store)),
        };
        r.create_midi_note_labels();
        r
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        if let Some(kit) = KitLibrary::global().kit(self.kit_index) {
            let voice_store = VoicePerNoteStore::<SamplerVoice>::new_with_voices(
                kit.items.iter().flat_map(|item| {
                    if let Some(path) = SampleLibrary::global().path(item.index) {
                        let path = Paths::global().build_sample(&Vec::default(), path.as_path());
                        if let Ok(file) = Paths::global().search_and_open(path.as_path()) {
                            if let Ok(samples) = SamplerCore::read_samples_from_file(&file) {
                                let note = item.note as u8;
                                Ok((
                                    u7::from(note),
                                    SamplerVoice::new_with_samples(
                                        Arc::new(samples),
                                        MidiNote::from_repr(note as usize).unwrap().into(),
                                    ),
                                ))
                            } else {
                                Err(anyhow!("Unable to load sample from file {:?}.", path))
                            }
                        } else {
                            Err(anyhow!("Couldn't find filename {:?} in hives", path))
                        }
                    } else {
                        Err(anyhow!("Couldn't find path for item"))
                    }
                }),
            );
            self.inner_synth = Synthesizer::<SamplerVoice>::new_with(Box::new(voice_store));

            Ok(())
        } else {
            Err(anyhow!("Couldn't find kit {}", self.kit_index))
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn kit_index(&self) -> KitIndex {
        self.kit_index
    }

    pub(crate) fn set_kit_index(&mut self, kit_index: KitIndex) {
        if kit_index != self.kit_index {
            self.kit_index = kit_index;
            self.load();
        }
    }

    fn create_midi_note_labels(&mut self) -> Option<MidiNoteLabelMetadata> {
        if let Some(kit) = KitLibrary::global().kit(self.kit_index) {
            let mut note_start = MidiNote::MAX;
            let mut note_end = MidiNote::MIN;
            let labels = kit.items.iter().fold(Vec::default(), |mut v, item| {
                v.push(item.name.clone());
                if note_start < item.note {
                    note_start = item.note;
                }
                if note_end > item.note {
                    note_end = item.note;
                }
                v
            });
            Some(MidiNoteLabelMetadata {
                range: note_start..=note_end,
                labels,
            })
        } else {
            None
        }
    }
}
