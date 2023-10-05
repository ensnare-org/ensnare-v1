// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::prelude::*;
use std::path::PathBuf;

// Demonstrates the control (automation) system.
#[test]
fn demo_automation() {
    let factory = register_factory_entities(EntityFactory::default());

    let mut orchestrator = OrchestratorBuilder::default()
        .title(Some("Automation".to_string()))
        .build()
        .unwrap();

    // We scope this block so that we can work with Orchestrator only as
    // something implementing the [Orchestrates] trait. This makes sure we're
    // testing the generic trait behavior as much as possible.
    {
        let orchestrator: &mut dyn Orchestrates = &mut orchestrator;

        orchestrator.update_tempo(Tempo(128.0));

        let mut piano_roll = PianoRoll::default();

        // Add the lead pattern to the PianoRoll.
        let scale_pattern_uid = {
            piano_roll.insert(
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
        };

        // Arrange the lead pattern in the sequencer.
        let track_uid = orchestrator.create_track().unwrap();
        assert!(orchestrator
            .append_entity(
                &track_uid,
                factory.create_entity_with_minted_uid(|| {
                    let pattern = piano_roll.get_pattern(&scale_pattern_uid).unwrap().clone();
                    Box::new(
                        ESSequencerBuilder::default()
                            .pattern((MusicalTime::new_with_beats(0), pattern))
                            .build()
                            .unwrap(),
                    )
                })
            )
            .is_ok());

        // Add a synth to play the pattern.
        let synth_uid = orchestrator
            .append_entity(
                &track_uid,
                factory.new_entity(&EntityKey::from("toy-synth")).unwrap(),
            )
            .unwrap();

        // Add an LFO that will control a synth parameter.
        let lfo_uid = {
            let lfo = factory.create_entity_with_minted_uid(|| {
                Box::new(LfoController::new_with(&LfoControllerParams {
                    frequency: FrequencyHz(2.0),
                    waveform: Waveform::Sine,
                }))
            });
            orchestrator.append_entity(&track_uid, lfo).unwrap()
        };

        let pan_param_index = {
            // This would have been a little easier if Orchestrator or Track had a
            // way to query param names, but I'm not sure how often that will
            // happen.
            factory
                .new_entity(&EntityKey::from("toy-synth"))
                .unwrap()
                .as_controllable()
                .unwrap()
                .control_index_for_name("dca-pan")
                .unwrap()
        };

        // Link the LFO to the synth's pan.
        assert!(orchestrator
            .link_control(lfo_uid, synth_uid, pan_param_index)
            .is_ok());
    }
    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: PathBuf = [env!("CARGO_TARGET_TMPDIR"), "automation.wav"]
        .iter()
        .collect();
    assert!(orchestrator.write_to_file(&output_path).is_ok());
}

#[test]
fn demo_control_trips() {
    let factory = register_factory_entities(EntityFactory::default());

    let mut orchestrator = OrchestratorBuilder::default()
        .title(Some("Automation".to_string()))
        .build()
        .unwrap();

    // We scope this block so that we can work with Orchestrator only as
    // something implementing the [Orchestrates] trait. This makes sure we're
    // testing the generic trait behavior as much as possible.
    {
        let orchestrator: &mut dyn Orchestrates = &mut orchestrator;

        orchestrator.update_tempo(Tempo(128.0));

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
        let track_uid = orchestrator.create_track().unwrap();
        assert!(orchestrator
            .append_entity(
                &track_uid,
                factory.create_entity_with_minted_uid(|| Box::new(
                    ESSequencerBuilder::default()
                        .pattern((MusicalTime::START, scale_pattern.clone()))
                        .build()
                        .unwrap()
                ))
            )
            .is_ok());

        // Add a synth to play the pattern. Figure how out to identify the
        // parameter we want to control.
        let entity = factory.new_entity(&EntityKey::from("toy-synth")).unwrap();
        let pan_param_index = entity
            .as_controllable()
            .unwrap()
            .control_index_for_name("dca-pan")
            .unwrap();
        let synth_uid = orchestrator.append_entity(&track_uid, entity).unwrap();

        // Create a [ControlAtlas] that will manage [ControlTrips](ControlTrip).
        // First add a ControlTrip that ramps from zero to max over the desired
        // amount of time.
        let (atlas, trip_uid) = {
            let mut trip = ControlTripBuilder::default()
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
            let trip_uid = factory.assign_entity_uid(&mut trip);
            let atlas = ControlAtlasBuilder::default()
                .trip(trip)
                .uid(factory.mint_uid())
                .build()
                .unwrap();
            (Box::new(atlas), trip_uid)
        };

        let _ = orchestrator.append_entity(&track_uid, atlas).unwrap();

        // Hook up that ControlTrip to the pan parameter.
        assert!(orchestrator
            .link_control(trip_uid, synth_uid, pan_param_index)
            .is_ok());
    } // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: PathBuf = [env!("CARGO_TARGET_TMPDIR"), "control-trips.wav"]
        .iter()
        .collect();
    assert!(orchestrator.write_to_file(&output_path).is_ok());
}
