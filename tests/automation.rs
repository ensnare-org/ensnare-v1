// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    automation::{ControlStepBuilder, ControlTripBuilder, ControlTripPath},
    entities_future::{BuiltInEntities, LfoController},
    prelude::*,
};
use ensnare_entities_toy::prelude::*;
use ensnare_entity::factory::EntityFactory;
use std::path::PathBuf;

// Demonstrates the control (automation) system.
#[test]
fn demo_automation() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut project = Project::default();

    project.update_tempo(Tempo(128.0));

    // Add the lead pattern.
    let scale_pattern_uid = {
        project
            .add_pattern(
                PatternBuilder::default()
                    .note_sequence(
                        vec![
                            60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
                        ],
                        None,
                    )
                    .build()
                    .unwrap(),
                None,
            )
            .unwrap()
    };

    // Arrange the lead pattern in the sequencer.
    let track_uid = project.create_track(None).unwrap();
    assert!(project
        .arrange_pattern(track_uid, scale_pattern_uid, MusicalTime::START)
        .is_ok());

    // Add a synth to play the pattern.
    let synth_uid = project
        .add_entity(
            track_uid,
            factory
                .new_entity(EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .set_midi_receiver_channel(synth_uid, Some(MidiChannel::default()))
        .is_ok());

    // Add an LFO that will control a synth parameter.
    let lfo_uid = {
        project
            .add_entity(
                track_uid,
                Box::new(LfoController::new_with(
                    Uid::default(),
                    Oscillator::new_with_waveform_and_frequency(Waveform::Sine, FrequencyHz(2.0)),
                )),
                None,
            )
            .unwrap()
    };

    let pan_param_index = {
        // This would have been a little easier if Orchestrator or Track had a
        // way to query param names, but I'm not sure how often that will
        // happen.
        factory
            .new_entity(EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
            .unwrap()
            .control_index_for_name("dca-pan")
            .unwrap()
    };

    // Link the LFO to the synth's pan.
    assert!(project.link(lfo_uid, synth_uid, pan_param_index).is_ok());

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: PathBuf = [env!("CARGO_TARGET_TMPDIR"), "automation.wav"]
        .iter()
        .collect();
    assert!(project.export_to_wav(output_path).is_ok());
}

#[test]
fn demo_control_trips() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut project = Project::default();

    project.update_tempo(Tempo(128.0));

    // Create the lead pattern.
    let scale_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
                    ],
                    None,
                )
                .build()
                .unwrap(),
            None,
        )
        .unwrap();

    // Arrange the lead pattern.
    let track_uid = project.create_track(None).unwrap();
    assert!(project
        .arrange_pattern(track_uid, scale_pattern_uid, MusicalTime::START)
        .is_ok());

    // Add a synth to play the pattern. Figure how out to identify the
    // parameter we want to control.
    let entity = factory
        .new_entity(EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
        .unwrap();
    let _pan_param_index = entity.control_index_for_name("dca-pan").unwrap();
    let synth_uid = project.add_entity(track_uid, entity, None).unwrap();
    assert!(project
        .set_midi_receiver_channel(synth_uid, Some(MidiChannel::default()))
        .is_ok());

    // Create a ControlTrip that ramps from zero to max over the desired
    // amount of time.

    // TODO: To get settings work to build, I'm substituting a default
    // ControlTripParams instead of all this.
    let _trip = ControlTripBuilder::default()
        .step(
            ControlStepBuilder::default()
                .value(ControlValue::MIN)
                .time(MusicalTime::START)
                .path(ControlTripPath::Linear)
                .build()
                .unwrap(),
        )
        .step(
            ControlStepBuilder::default()
                .value(ControlValue::MAX)
                .time(MusicalTime::new_with_beats(4))
                .path(ControlTripPath::Flat)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    #[cfg(fixme)]
    {
        let outer_trip = Box::new(ControlTrip::new_with(
            Uid::default(),
            &trip_params,
            &control_router_clone,
        ));
        let trip_uid = project.add_entity(track_uid, outer_trip, None).unwrap();

        // Hook up that ControlTrip to the pan parameter.
        assert!(project.link(trip_uid, synth_uid, pan_param_index).is_ok());
    }
    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: PathBuf = [env!("CARGO_TARGET_TMPDIR"), "control-trips.wav"]
        .iter()
        .collect();
    assert!(project.export_to_wav(output_path).is_ok());
}
