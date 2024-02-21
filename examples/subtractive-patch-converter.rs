// Copyright (c) 2024 Mike Tsao. All rights reserved.

use ensnare::{
    cores::{
        effects::BiQuadFilterLowPass24dbCoreBuilder,
        instruments::{LfoRouting, SubtractiveSynthCoreBuilder},
    },
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LfoDepth {
    None,
    Pct(f32),
    Cents(f32),
}
impl From<LfoDepth> for Normal {
    fn from(val: LfoDepth) -> Self {
        match val {
            LfoDepth::None => Normal::minimum(),
            LfoDepth::Pct(pct) => Normal::new(pct as f64),
            LfoDepth::Cents(cents) => {
                Normal::new(1.0 - OscillatorSettings::semis_and_cents(0, cents as f64).0)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LfoRoutingType {
    #[default]
    None,
    Amplitude,
    Pitch,
    PulseWidth,
    FilterCutoff,
    #[serde(rename = "resonance")]
    FilterResonance,
    #[serde(rename = "pitch-2")]
    Pitch2,
    #[serde(rename = "pulse-width-2")]
    PulseWidth2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LfoPreset {
    pub routing: LfoRoutingType,
    pub waveform: Waveform,
    pub frequency: f32,
    pub depth: LfoDepth,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OscillatorTune {
    Note(u8),
    Float(ParameterType),
    Osc { octave: i8, semi: i8, cent: i8 },
}
impl From<OscillatorTune> for Ratio {
    fn from(val: OscillatorTune) -> Self {
        match val {
            OscillatorTune::Note(_) => Ratio::from(1.0),
            OscillatorTune::Float(value) => Ratio::from(value),
            OscillatorTune::Osc { octave, semi, cent } => {
                OscillatorSettings::semis_and_cents(octave as i16 * 12 + semi as i16, cent as f64)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OscillatorSettings {
    pub waveform: Waveform,
    pub tune: OscillatorTune,

    #[serde(rename = "mix-pct")]
    pub mix: f32,
}
impl Default for OscillatorSettings {
    fn default() -> Self {
        Self {
            waveform: Waveform::default(),
            tune: OscillatorTune::Osc {
                octave: 0,
                semi: 0,
                cent: 0,
            },
            mix: 1.0,
        }
    }
}
impl OscillatorSettings {
    #[allow(dead_code)]
    pub fn octaves(num: i16) -> Ratio {
        Self::semis_and_cents(num * 12, 0.0)
    }

    pub fn semis_and_cents(semitones: i16, cents: f64) -> Ratio {
        // https://en.wikipedia.org/wiki/Cent_(music)
        Ratio::from(2.0f64.powf((semitones as f64 * 100.0 + cents) / 1200.0))
    }

    // pub fn derive_oscillator(&self) -> Oscillator {
    //     let mut r = Oscillator::new_with(&OscillatorParams::default_with_waveform(
    //         self.waveform.into(),
    //     ));
    //     r.set_frequency_tune(self.tune.into());
    //     r
    // }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolyphonySettings {
    Multi,
    Mono,
    MultiLimit(u8),
    All,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilterPreset {
    pub cutoff_hz: f32,
    pub cutoff_pct: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvelopeParams {
    attack: Seconds,
    decay: Seconds,
    sustain: Normal,
    release: Seconds,
}

#[allow(clippy::from_over_into)]
impl Into<LfoRouting> for LfoRoutingType {
    fn into(self) -> LfoRouting {
        match self {
            LfoRoutingType::None => LfoRouting::None,
            LfoRoutingType::Amplitude => LfoRouting::Amplitude,
            LfoRoutingType::Pitch => LfoRouting::Pitch,
            LfoRoutingType::PulseWidth => LfoRouting::PulseWidth,
            LfoRoutingType::FilterCutoff => LfoRouting::FilterCutoff,
            LfoRoutingType::FilterResonance => LfoRouting::FilterResonance,
            LfoRoutingType::Pitch2 => LfoRouting::Pitch2,
            LfoRoutingType::PulseWidth2 => LfoRouting::PulseWidth2,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WelshPatchSettings {
    pub name: String,
    pub oscillator_1: OscillatorSettings,
    pub oscillator_2: OscillatorSettings,
    pub oscillator_2_track: bool,
    pub oscillator_2_sync: bool,

    pub noise: f32,

    pub lfo: LfoPreset,

    pub glide: f32,
    pub unison: bool,
    pub polyphony: PolyphonySettings,

    // There is meant to be only one filter, but the Welsh book
    // provides alternate settings depending on the kind of filter
    // your synthesizer has.
    pub filter_type_24db: FilterPreset,
    pub filter_type_12db: FilterPreset,
    pub filter_resonance: f32, // This should be an appropriate interpretation of a linear 0..1
    pub filter_envelope_weight: f32,
    pub filter_envelope: EnvelopeParams,

    pub amp_envelope: EnvelopeParams,
}

impl WelshPatchSettings {
    pub fn load_patch(path: &PathBuf) -> anyhow::Result<Self> {
        let mut path = path.clone();
        path.set_extension("json");
        let json = std::fs::read_to_string(&path)?;
        let mut patch = serde_json::from_str::<Self>(&json)?;
        // patch.e.load_path = Some(path);
        Ok(patch)
    }
}

fn main() -> anyhow::Result<()> {
    let paths = std::fs::read_dir(PathBuf::from("assets/patches/welsh/"))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    for path in paths {
        let preset_name = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .replace(".json", "");
        println!("Processing {preset_name:?}");
        let patch = WelshPatchSettings::load_patch(&path)?;

        let (oscillator_1_waveform, oscillator_2_waveform) = if patch.noise != 0.0 {
            (Waveform::Noise, Waveform::None)
        } else {
            (patch.oscillator_1.waveform, patch.oscillator_2.waveform)
        };
        let fixed_frequency_1: Option<FrequencyHz> =
            if let OscillatorTune::Note(midi_note) = patch.oscillator_1.tune {
                Some(MidiNote::from(midi_note).into())
            } else {
                None
            };
        let fixed_frequency_2: Option<FrequencyHz> =
            if let OscillatorTune::Note(midi_note) = patch.oscillator_2.tune {
                println!("YES! {midi_note}");
                Some(MidiNote::from(midi_note).into())
            } else {
                None
            };
        let oscillator_1 = OscillatorBuilder::default()
            .waveform(oscillator_1_waveform)
            .frequency_tune(patch.oscillator_1.tune.into())
            .fixed_frequency(fixed_frequency_1)
            .build()?;
        let oscillator_2 = OscillatorBuilder::default()
            .waveform(oscillator_2_waveform)
            .frequency_tune(patch.oscillator_2.tune.into())
            .fixed_frequency(fixed_frequency_2)
            .build()?;
        let oscillator_mix = if patch.noise != 0.0 {
            patch.noise
        } else {
            if patch.oscillator_1.mix + patch.oscillator_2.mix != 0.0 {
                patch.oscillator_1.mix / (patch.oscillator_1.mix + patch.oscillator_2.mix)
            } else {
                0.5
            }
        };
        let amp_envelope = EnvelopeBuilder::default()
            .attack(patch.amp_envelope.attack.into())
            .decay(patch.amp_envelope.decay.into())
            .sustain(patch.amp_envelope.sustain.into())
            .release(patch.amp_envelope.release.into())
            .build()?;
        let lfo_oscillator = OscillatorBuilder::default()
            .waveform(patch.lfo.waveform)
            .fixed_frequency(Some(patch.lfo.frequency.into()))
            .build()?;
        let filter = BiQuadFilterLowPass24dbCoreBuilder::default()
            .cutoff(patch.filter_type_24db.cutoff_hz.into())
            .passband_ripple(1.0)
            .build()?;
        let filter_envelope = EnvelopeBuilder::default()
            .attack(patch.filter_envelope.attack.into())
            .decay(patch.filter_envelope.decay.into())
            .sustain(patch.filter_envelope.sustain.into())
            .release(patch.filter_envelope.release.into())
            .build()?;
        let mut synth = SubtractiveSynthCoreBuilder::default()
            .oscillator_1(oscillator_1)
            .oscillator_2(oscillator_2)
            .oscillator_2_sync(patch.oscillator_2_sync)
            .oscillator_mix(oscillator_mix.into())
            .amp_envelope(amp_envelope)
            .dca(Dca::default())
            .lfo(lfo_oscillator)
            .lfo_routing(patch.lfo.routing.into())
            .lfo_depth(patch.lfo.depth.into())
            .filter(filter)
            .filter_cutoff_start(patch.filter_type_24db.cutoff_hz.into())
            .filter_cutoff_end(patch.filter_envelope_weight.into())
            .filter_envelope(filter_envelope)
            .preset_name(Some(preset_name))
            .build()?;

        if let Some(patch_name) = path.file_stem().as_ref() {
            let patch_name = patch_name.to_ascii_lowercase();
            let mut path = PathBuf::from("assets/patches/subtractive/");
            if let Some(patch_name) = patch_name.to_str() {
                path.push(format!("{}.json", patch_name));
                synth.save_patch(&path)?;
            } else {
                eprintln!("skipped {:?}", patch_name);
            }
        }
    }
    Ok(())
}
