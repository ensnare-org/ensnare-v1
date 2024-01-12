// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    control::{ControlStepBuilder, ControlTripBuilder, ControlTripParams, ControlTripPath},
    cores::LfoControllerParams,
    entities::{
        controllers::{LfoController, PatternSequencer},
        toys::ToySynth,
    },
    generators::Waveform,
    prelude::*,
};
use std::path::PathBuf;

// Demonstrates the control (automation) system.
#[test]
fn demo_automation() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut project = Project::default();

    project.update_tempo(Tempo(128.0));

    let mut piano_roll = PianoRoll::default();

    // Add the lead pattern to the PianoRoll.
    let scale_pattern_uid = {
        piano_roll
            .insert(
                PatternBuilder::default()
                    .note_sequence(
                        vec![
                            60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
                        ],
                        None,
                    )
                    .build()
                    .unwrap(),
            )
            .unwrap()
    };

    // Arrange the lead pattern in the sequencer.
    let track_uid = project.create_track(None).unwrap();
    let mut sequencer = PatternSequencer::default();
    let pattern = piano_roll.get_pattern(&scale_pattern_uid).unwrap().clone();
    assert!(sequencer
        .record(MidiChannel::default(), &pattern.clone(), MusicalTime::START)
        .is_ok());

    assert!(project
        .add_entity(track_uid, Box::new(sequencer), None)
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
                    &LfoControllerParams {
                        frequency: FrequencyHz(2.0),
                        waveform: Waveform::Sine,
                    },
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
    let scale_pattern = PatternBuilder::default()
        .note_sequence(
            vec![
                60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
            ],
            None,
        )
        .build()
        .unwrap();

    // Arrange the lead pattern in a sequencer.

    // Add the sequencer to a new track.
    let track_uid = project.create_track(None).unwrap();
    let mut sequencer = PatternSequencer::default();
    assert!(sequencer
        .record(
            MidiChannel::default(),
            &scale_pattern.clone(),
            MusicalTime::START
        )
        .is_ok());
    assert!(project
        .add_entity(track_uid, Box::new(sequencer), None)
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
    let _trip_params = ControlTripParams::default();

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
