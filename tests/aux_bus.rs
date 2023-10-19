// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{entities::controllers::PatternSequencerBuilder, prelude::*};

// Demonstrates use of aux buses.
#[test]
fn aux_bus() {
    let _ = EntityFactory::initialize(register_factory_entities(EntityFactory::default()));
    let factory = EntityFactory::global();

    let mut orchestrator = OrchestratorBuilder::default()
        .title(Some("Auxiliary Buses".to_string()))
        .build()
        .unwrap();

    {
        let orchestrator: &mut dyn Orchestrates = &mut orchestrator;
        let track_uid_1 = orchestrator.create_track().unwrap();
        let track_uid_2 = orchestrator.create_track().unwrap();
        let aux_track_uid = orchestrator.create_track().unwrap();

        let synth_pattern_1 = PatternBuilder::default()
            .note_sequence(
                vec![
                    60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
                ],
                None,
            )
            .build()
            .unwrap();

        let synth_pattern_2 = PatternBuilder::default()
            .note_sequence(
                vec![
                    84, 255, 83, 255, 81, 255, 79, 255, 77, 255, 76, 255, 74, 255, 72, 255,
                ],
                None,
            )
            .build()
            .unwrap();

        {
            assert!(orchestrator
                .add_entity(
                    &track_uid_1,
                    Box::new(
                        PatternSequencerBuilder::default()
                            .pattern(synth_pattern_1.clone())
                            .build()
                            .unwrap(),
                    )
                )
                .is_ok());

            // Even though we want the effect to be placed after the instrument in
            // the audio chain, we can add the effect before we add the instrument.
            // This is because the processing order is always controllers,
            // instruments, effects.
            assert!(orchestrator
                .add_entity(
                    &track_uid_1,
                    factory.new_entity(&EntityKey::from("gain")).unwrap()
                )
                .is_ok());
            orchestrator
                .add_entity(
                    &track_uid_1,
                    factory.new_entity(&EntityKey::from("toy-synth")).unwrap(),
                )
                .unwrap();
        };
        let _synth_uid_2 = {
            assert!(orchestrator
                .add_entity(
                    &track_uid_2,
                    Box::new(
                        PatternSequencerBuilder::default()
                            .pattern(synth_pattern_2.clone())
                            .build()
                            .unwrap(),
                    )
                )
                .is_ok());
            assert!(orchestrator
                .add_entity(
                    &track_uid_2,
                    factory.new_entity(&EntityKey::from("gain")).unwrap()
                )
                .is_ok());
            orchestrator
                .add_entity(
                    &track_uid_2,
                    factory.new_entity(&EntityKey::from("toy-synth")).unwrap(),
                )
                .unwrap()
        };
        let _effect_uid_1 = {
            orchestrator
                .add_entity(
                    &aux_track_uid,
                    factory.new_entity(&EntityKey::from("gain")).unwrap(),
                )
                .unwrap();
            orchestrator
                .add_entity(
                    &aux_track_uid,
                    factory.new_entity(&EntityKey::from("reverb")).unwrap(),
                )
                .unwrap()
        };

        let _ = orchestrator.send_to_aux(track_uid_1, aux_track_uid, Normal::from(1.0));
        let _ = orchestrator.send_to_aux(track_uid_2, aux_track_uid, Normal::from(1.0));
    }
    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "aux-bus.wav"]
        .iter()
        .collect();
    assert!(orchestrator.write_to_file(&output_path).is_ok());
}
