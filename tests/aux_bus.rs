// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    entities::effects::{Gain, Reverb},
    prelude::*,
};
use ensnare_entities::controllers::PatternSequencer;
use ensnare_entities_toy::prelude::*;

// Demonstrates use of aux buses.
#[test]
fn aux_bus() {
    let mut factory = EntityFactory::default();
    register_factory_entities(&mut factory);
    register_toy_entities(&mut factory);
    let _ = EntityFactory::initialize(factory);
    let factory = EntityFactory::global();

    let mut orchestrator = Orchestrator::default();

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

        let synth_uid_1 = {
            let mut sequencer = PatternSequencer::default();
            assert!(sequencer
                .record(MidiChannel::default(), &synth_pattern_1, MusicalTime::START)
                .is_ok());
            assert!(orchestrator
                .assign_uid_and_add_entity(&track_uid_1, Box::new(sequencer))
                .is_ok());

            // Even though we want the effect to be placed after the instrument in
            // the audio chain, we can add the effect before we add the instrument.
            // This is because the processing order is always controllers,
            // instruments, effects.
            assert!(orchestrator
                .assign_uid_and_add_entity(
                    &track_uid_1,
                    factory
                        .new_entity(&EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                        .unwrap()
                )
                .is_ok());
            orchestrator
                .assign_uid_and_add_entity(
                    &track_uid_1,
                    factory
                        .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                        .unwrap(),
                )
                .unwrap()
        };
        assert!(orchestrator
            .connect_midi_receiver(synth_uid_1, MidiChannel::default())
            .is_ok());

        let synth_uid_2 = {
            let mut sequencer = PatternSequencer::default();
            assert!(sequencer
                .record(MidiChannel(1), &synth_pattern_2, MusicalTime::START)
                .is_ok());
            assert!(orchestrator
                .assign_uid_and_add_entity(&track_uid_2, Box::new(sequencer))
                .is_ok());
            assert!(orchestrator
                .assign_uid_and_add_entity(
                    &track_uid_2,
                    factory
                        .new_entity(&EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                        .unwrap()
                )
                .is_ok());
            orchestrator
                .assign_uid_and_add_entity(
                    &track_uid_2,
                    factory
                        .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                        .unwrap(),
                )
                .unwrap()
        };
        assert!(orchestrator
            .connect_midi_receiver(synth_uid_2, MidiChannel(1))
            .is_ok());

        let _effect_uid_1 = {
            orchestrator
                .assign_uid_and_add_entity(
                    &aux_track_uid,
                    factory
                        .new_entity(&EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                        .unwrap(),
                )
                .unwrap();
            orchestrator
                .assign_uid_and_add_entity(
                    &aux_track_uid,
                    factory
                        .new_entity(&EntityKey::from(Reverb::ENTITY_KEY), Uid::default())
                        .unwrap(),
                )
                .unwrap()
        };

        let _ = orchestrator.send(track_uid_1, aux_track_uid, Normal::from(1.0));
        let _ = orchestrator.send(track_uid_2, aux_track_uid, Normal::from(1.0));
    }
    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "aux-bus.wav"]
        .iter()
        .collect();
    let mut orchestrator_helper = OrchestratorHelper::new_with(&mut orchestrator);
    assert!(orchestrator_helper.write_to_file(&output_path).is_ok());
}
