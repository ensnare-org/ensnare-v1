// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    cores::controllers::LfoControllerCoreBuilder,
    entities::{BuiltInEntities, LfoController},
    prelude::*,
    util::library::init_sample_libraries,
};
use ensnare_toys::prelude::*;

// Demonstrates the control (automation) system.
#[test]
fn demo_automation() {
    init_sample_libraries();
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
        .arrange_pattern(track_uid, scale_pattern_uid, None, MusicalTime::START)
        .is_ok());

    // Add a synth to play the pattern.
    let synth_uid = project
        .add_entity(
            track_uid,
            factory
                .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
        )
        .unwrap();

    // Add an LFO that will control a synth parameter.
    let lfo_uid = {
        project
            .add_entity(
                track_uid,
                Box::new(LfoController::new_with(
                    Uid::default(),
                    LfoControllerCoreBuilder::default().build().unwrap(),
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
            .control_index_for_name("dca-pan")
            .unwrap()
    };

    // Link the LFO to the synth's pan.
    assert!(project.link(lfo_uid, synth_uid, pan_param_index).is_ok());

    let output_prefix: std::path::PathBuf =
        [env!("CARGO_TARGET_TMPDIR"), "automation"].iter().collect();
    assert!(project.save_and_export(output_prefix).is_ok());
}

#[test]
fn demo_signal_path_automation() {
    init_sample_libraries();
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
        .arrange_pattern(track_uid, scale_pattern_uid, None, MusicalTime::START)
        .is_ok());

    // Add a synth to play the pattern. Figure how out to identify the
    // parameter we want to control.
    let entity = factory
        .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
        .unwrap();
    let pan_param_index = entity.control_index_for_name("dca-pan").unwrap();
    let synth_uid = project.add_entity(track_uid, entity).unwrap();

    // Create a SignalPath that ramps from zero to max over the desired
    // amount of time.
    let path = SignalPathBuilder::default()
        .point(
            SignalPointBuilder::default()
                .when(MusicalTime::START)
                .value(BipolarNormal::minimum())
                .build()
                .unwrap(),
        )
        .point(
            SignalPointBuilder::default()
                .when(MusicalTime::new_with_beats(4))
                .value(BipolarNormal::maximum())
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    let path_uid = project.add_path(track_uid, path).unwrap();

    // Hook it up to the pan parameter.
    assert!(project
        .link_path(path_uid, synth_uid, pan_param_index)
        .is_ok());

    let output_prefix: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "signal-path-automation"]
        .iter()
        .collect();
    assert!(project.save_and_export(output_prefix).is_ok());
}
