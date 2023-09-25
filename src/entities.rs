// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    control::ControlTrip, controllers::Timer,
    entities::test_entities::TestControllerAlwaysSendsMidiMessage, generators::Waveform,
    midi::prelude::*, mini_sequencer::SequencerBuilder, prelude::*, utils::Paths,
};
use ensnare_entities::{
    effects::{bitcrusher::Bitcrusher, compressor::Compressor, limiter::Limiter, mixer::Mixer},
    instruments::{
        drumkit::{Drumkit, DrumkitParams},
        fm::{FmSynth, FmSynthParams},
        sampler::{Sampler, SamplerParams},
    },
    prelude::*,
};
use ensnare_toys::{ToyController, ToyEffect, ToyInstrument, ToySynth, ToySynthParams};

/// Registers all [EntityFactory]'s entities. Note that the function returns a
/// EntityFactory, rather than operating on an &mut. This encourages
/// one-and-done creation, after which the factory is immutable:
///
/// ```ignore
/// let factory = register_factory_entities(EntityFactory::default());
/// ```
///
/// TODO: maybe a Builder pattern is better, so that people can compose
/// factories out of any entities they want, and still get the benefits of
/// immutability.
#[must_use]
pub fn register_factory_entities(mut factory: EntityFactory) -> EntityFactory {
    // TODO: might be nice to move HasUid::name() to be a function.

    // Controllers
    factory.register_entity(EntityKey::from("arpeggiator"), || {
        Box::new(Arpeggiator::new_with(
            &ArpeggiatorParams::default(),
            MidiChannel(0),
        ))
    });
    factory.register_entity(EntityKey::from("control-trip"), || {
        Box::new(ControlTrip::default())
    });
    factory.register_entity(EntityKey::from("lfo"), || {
        Box::new(LfoController::new_with(&LfoControllerParams {
            frequency: FrequencyHz::from(0.2),
            waveform: Waveform::Sawtooth,
        }))
    });
    factory.register_entity(EntityKey::from("sequencer"), || {
        Box::new(
            SequencerBuilder::default()
                .midi_channel_out(MidiChannel(0))
                .build()
                .unwrap(),
        )
    });
    factory.register_entity(EntityKey::from("signal-passthrough"), || {
        Box::<SignalPassthroughController>::default()
    });
    factory.register_entity(EntityKey::from("signal-amplitude-passthrough"), || {
        Box::new(SignalPassthroughController::new_amplitude_passthrough_type())
    });
    factory.register_entity(
        EntityKey::from("signal-amplitude-inverted-passthrough"),
        || Box::new(SignalPassthroughController::new_amplitude_inverted_passthrough_type()),
    );
    factory.register_entity(EntityKey::from("timer"), || {
        Box::new(Timer::new_with(MusicalTime::DURATION_QUARTER))
    });
    factory.register_entity(EntityKey::from("toy-controller"), || {
        Box::<ToyController>::default()
    });
    factory.register_entity(EntityKey::from("toy-controller-noisy"), || {
        Box::new(TestControllerAlwaysSendsMidiMessage::default())
    });

    // Effects
    factory.register_entity(EntityKey::from("bitcrusher"), || {
        Box::new(Bitcrusher::default())
    });
    factory.register_entity(EntityKey::from("compressor"), || {
        Box::new(Compressor::default())
    });
    factory.register_entity(EntityKey::from("filter-low-pass-24db"), || {
        Box::new(BiQuadFilterLowPass24db::new_with(
            &BiQuadFilterLowPass24dbParams::default(),
        ))
    });
    factory.register_entity(EntityKey::from("gain"), || {
        Box::new(Gain::new_with(&GainParams {
            ceiling: Normal::from(0.5),
        }))
    });
    factory.register_entity(EntityKey::from("limiter"), || Box::new(Limiter::default()));
    factory.register_entity(EntityKey::from("mixer"), || Box::new(Mixer::default()));
    // TODO: this is lazy. It's too hard right now to adjust parameters within
    // code, so I'm creating a special instrument with the parameters I want.
    factory.register_entity(EntityKey::from("mute"), || {
        Box::new(Gain::new_with(&GainParams {
            ceiling: Normal::minimum(),
        }))
    });
    factory.register_entity(EntityKey::from("reverb"), || {
        Box::new(Reverb::new_with(&ReverbParams {
            attenuation: Normal::from(0.8),
            seconds: 1.0,
        }))
    });
    factory.register_entity(EntityKey::from("toy-effect"), || {
        Box::<ToyEffect>::default()
    });

    // Instruments
    factory.register_entity(EntityKey::from("toy-synth"), || {
        Box::new(ToySynth::new_with(&ToySynthParams::default()))
    });
    factory.register_entity(EntityKey::from("toy-instrument"), || {
        Box::<ToyInstrument>::default()
    });
    factory.register_entity(EntityKey::from("drumkit"), || {
        Box::new(Drumkit::new_with(
            &DrumkitParams::default(),
            &Paths::default(),
        ))
    });
    factory.register_entity(EntityKey::from("fm-synth"), || {
        Box::new(FmSynth::new_with(&FmSynthParams::default()))
    });
    factory.register_entity(EntityKey::from("sampler"), || {
        Box::new(Sampler::new_with(
            &SamplerParams {
                filename: "stereo-pluck.wav".to_string(),
                root: 0.0.into(),
            },
            &Paths::default(),
        ))
    });
    factory.register_entity(EntityKey::from("welsh-synth"), || {
        Box::new(WelshSynth::new_with(&WelshSynthParams::default()))
    });

    factory.complete_registration();

    factory
}
