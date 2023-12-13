// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    control::{ControlStepBuilder, ControlTripBuilder, ControlTripParams, ControlTripPath},
    cores::LfoControllerParams,
    entities::{
        controllers::{ControlTrip, LfoController, PatternSequencer},
        toys::ToySynth,
    },
    generators::Waveform,
    prelude::*,
};
use ensnare_entity::traits::EntityBounds;
use std::{path::PathBuf, sync::Arc};

// Demonstrates the control (automation) system.
#[test]
fn demo_automation() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut orchestrator = Orchestrator::<dyn EntityBounds>::new();

    // We scope this block so that we can work with Orchestrator only as
    // something implementing the [Orchestrates] trait. This makes sure we're
    // testing the generic trait behavior as much as possible.
    {
        let orchestrator: &mut dyn Orchestrates<dyn EntityBounds> = &mut orchestrator;

        orchestrator.update_tempo(Tempo(128.0));

        let mut piano_roll = PianoRoll::default();

        // Add the lead pattern to the PianoRoll.
        let scale_pattern_uid = {
            piano_roll
                .insert(
                    PatternBuilder::default()
                        .note_sequence(
                            vec![
                                60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72,
                                255,
                            ],
                            None,
                        )
                        .build()
                        .unwrap(),
                )
                .unwrap()
        };

        // Arrange the lead pattern in the sequencer.
        let track_uid = orchestrator.create_track(None).unwrap();
        let mut sequencer = PatternSequencer::default();
        let pattern = piano_roll.get_pattern(&scale_pattern_uid).unwrap().clone();
        assert!(sequencer
            .record(MidiChannel::default(), &pattern.clone(), MusicalTime::START)
            .is_ok());

        assert!(orchestrator
            .assign_uid_and_add_entity(&track_uid, Box::new(sequencer))
            .is_ok());

        // Add a synth to play the pattern.
        let synth_uid = orchestrator
            .assign_uid_and_add_entity(
                &track_uid,
                factory
                    .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .unwrap();
        assert!(orchestrator
            .connect_midi_receiver(synth_uid, MidiChannel::default())
            .is_ok());

        // Add an LFO that will control a synth parameter.
        let lfo_uid = {
            orchestrator
                .assign_uid_and_add_entity(
                    &track_uid,
                    Box::new(LfoController::new_with(
                        Uid::default(),
                        &LfoControllerParams {
                            frequency: FrequencyHz(2.0),
                            waveform: Waveform::Sine,
                        },
                    )),
                )
                .unwrap()
        };

        let pan_param_index = {
            // This would have been a little easier if Orchestrator or Track had a
            // way to query param names, but I'm not sure how often that will
            // happen.
            factory
                .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
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
    let mut orchestrator_helper = OrchestratorHelper::new_with(&mut orchestrator);
    assert!(orchestrator_helper.write_to_file(&output_path).is_ok());
}

#[test]
fn demo_control_trips() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut orchestrator = Orchestrator::<dyn EntityBounds>::new();
    let control_router_clone = Arc::clone(&orchestrator.control_router);

    // Per my epiphany from a few days ago, Orchestrates (the trait) defines
    // arrangement of Entities, and doesn't get into the actual information that
    // the Entities contain. That is why Orchestrates doesn't know anything
    // about patterns, which are implementation details of a certain kind of
    // controller called a sequencer. Likewise, a ControlAtlas is just another
    // controller. Orchestrates doesn't know about ControlAtlas, and thus
    // doesn't know about ControlTrip.
    //
    // Addendum: this comment turned out to be superfluous, but it's still
    // valid. I'm still letting this idea soak in, so I'm keeping the comment
    // here rather than deleting it without checking it in.

    // We scope this block so that we can work with Orchestrator only as
    // something implementing the [Orchestrates] trait. This makes sure we're
    // testing the generic trait behavior as much as possible.
    {
        let orchestrator: &mut dyn Orchestrates<dyn EntityBounds> = &mut orchestrator;

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
        let track_uid = orchestrator.create_track(None).unwrap();
        let mut sequencer = PatternSequencer::default();
        assert!(sequencer
            .record(
                MidiChannel::default(),
                &scale_pattern.clone(),
                MusicalTime::START
            )
            .is_ok());
        assert!(orchestrator
            .assign_uid_and_add_entity(&track_uid, Box::new(sequencer))
            .is_ok());

        // Add a synth to play the pattern. Figure how out to identify the
        // parameter we want to control.
        let entity = factory
            .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
            .unwrap();
        let pan_param_index = entity
            .as_controllable()
            .unwrap()
            .control_index_for_name("dca-pan")
            .unwrap();
        let synth_uid = orchestrator
            .assign_uid_and_add_entity(&track_uid, entity)
            .unwrap();
        assert!(orchestrator
            .connect_midi_receiver(synth_uid, MidiChannel::default())
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
        let trip_params = ControlTripParams::default();
        let outer_trip = Box::new(ControlTrip::new_with(
            Uid::default(),
            &trip_params,
            &control_router_clone,
        ));
        let trip_uid = orchestrator
            .assign_uid_and_add_entity(&track_uid, outer_trip)
            .unwrap();

        // Hook up that ControlTrip to the pan parameter.
        assert!(orchestrator
            .link_control(trip_uid, synth_uid, pan_param_index)
            .is_ok());
    } // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: PathBuf = [env!("CARGO_TARGET_TMPDIR"), "control-trips.wav"]
        .iter()
        .collect();
    let mut orchestrator_helper = OrchestratorHelper::new_with(&mut orchestrator);
    assert!(orchestrator_helper.write_to_file(&output_path).is_ok());
}
