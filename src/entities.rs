// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{
    entities::{EntityFactory, EntityKey},
    midi::MidiChannel,
    prelude::*,
    temp_impls::{
        controllers::{
            arpeggiator::{Arpeggiator, ArpeggiatorParams},
            mini_sequencer::SequencerBuilder,
            SignalPassthroughController, Timer,
        },
        effects::{
            filter::{BiQuadFilterLowPass24db, BiQuadFilterLowPass24dbParams},
            gain::{Gain, GainParams},
            reverb::{Reverb, ReverbParams},
        },
    },
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

    factory.register_entity(EntityKey::from("arpeggiator"), || {
        Box::new(Arpeggiator::new_with(
            &ArpeggiatorParams::default(),
            MidiChannel(0),
        ))
    });
    factory.register_entity(EntityKey::from("sequencer"), || {
        Box::new(
            SequencerBuilder::default()
                .midi_channel_out(MidiChannel(0))
                .build()
                .unwrap(),
        )
    });
    factory.register_entity(EntityKey::from("reverb"), || {
        Box::new(Reverb::new_with(&ReverbParams {
            attenuation: Normal::from(0.8),
            seconds: 1.0,
        }))
    });
    factory.register_entity(EntityKey::from("gain"), || {
        Box::new(Gain::new_with(&GainParams {
            ceiling: Normal::from(0.5),
        }))
    });
    // TODO: this is lazy. It's too hard right now to adjust parameters within
    // code, so I'm creating a special instrument with the parameters I want.
    factory.register_entity(EntityKey::from("mute"), || {
        Box::new(Gain::new_with(&GainParams {
            ceiling: Normal::minimum(),
        }))
    });
    factory.register_entity(EntityKey::from("filter-low-pass-24db"), || {
        Box::new(BiQuadFilterLowPass24db::new_with(
            &BiQuadFilterLowPass24dbParams::default(),
        ))
    });
    factory.register_entity(EntityKey::from("timer"), || {
        Box::new(Timer::new_with(MusicalTime::DURATION_QUARTER))
    });
    factory.register_entity(EntityKey::from("toy-synth"), || {
        Box::new(ToySynth::new_with(&ToySynthParams::default()))
    });
    factory.register_entity(EntityKey::from("toy-instrument"), || {
        Box::new(ToyInstrument::default())
    });
    factory.register_entity(EntityKey::from("toy-controller"), || {
        Box::new(ToyController::default())
    });
    factory.register_entity(EntityKey::from("toy-effect"), || {
        Box::new(ToyEffect::default())
    });
    // factory.register_entity(Key::from("toy-controller-noisy"), || {
    //     Box::new(ToyControllerAlwaysSendsMidiMessage::default())
    // });
    // factory.register_entity(Key::from("welsh-synth"), || {
    //     Box::new(WelshSynth::new_with(&WelshSynthParams::default()))
    // });
    // factory.register_entity(Key::from("drumkit"), || {
    //     Box::new(Drumkit::new_with(
    //         &DrumkitParams::default(),
    //         &Paths::default(),
    //     ))
    // });
    // factory.register_entity(Key::from("lfo"), || {
    //     Box::new(LfoController::new_with(&LfoControllerParams {
    //         frequency: FrequencyHz::from(0.2),
    //         waveform: Waveform::Sawtooth,
    //     }))
    // });
    // factory.register_entity(Key::from("control-trip"), || {
    //     Box::new(ControlTrip::default())
    // });
    factory.register_entity(EntityKey::from("signal-passthrough"), || {
        Box::new(SignalPassthroughController::default())
    });
    factory.register_entity(EntityKey::from("signal-amplitude-passthrough"), || {
        Box::new(SignalPassthroughController::new_amplitude_passthrough_type())
    });
    factory.register_entity(
        EntityKey::from("signal-amplitude-inverted-passthrough"),
        || Box::new(SignalPassthroughController::new_amplitude_inverted_passthrough_type()),
    );

    factory.complete_registration();

    factory
}
