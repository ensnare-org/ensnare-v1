// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    entities::controllers::PatternSequencer, entities::effects::Gain, entities::toys::ToySynth,
    prelude::*,
};
use ensnare_entities::BuiltInEntities;
use ensnare_entity::traits::EntityBounds;

// Demonstrates sidechaining (which could be considered a kind of automation,
// but it's important enough to put top-level and make sure it's a good
// experience and not merely possible).
#[test]
fn demo_sidechaining() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut orchestrator = Orchestrator::<dyn EntityBounds>::new();

    {
        let orchestrator: &mut dyn Orchestrates<dyn EntityBounds> = &mut orchestrator;
        // Add the sidechain source track.
        let sidechain_pattern = PatternBuilder::default()
            .note_sequence(
                vec![
                    35, 255, 255, 255, 35, 255, 255, 255, 35, 255, 255, 255, 35, 255, 255, 255,
                ],
                None,
            )
            .build()
            .unwrap();
        let sidechain_track_uid = orchestrator.create_track(None).unwrap();
        let mut sequencer = PatternSequencer::default();
        assert!(sequencer
            .record(
                MidiChannel::default(),
                &sidechain_pattern,
                MusicalTime::START
            )
            .is_ok());
        assert!(orchestrator
            .assign_uid_and_add_entity(&sidechain_track_uid, Box::new(sequencer))
            .is_ok());
        let sidechain_synth_uid = orchestrator
            .assign_uid_and_add_entity(
                &sidechain_track_uid,
                factory
                    .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .unwrap();
        assert!(orchestrator
            .connect_midi_receiver(sidechain_synth_uid, MidiChannel::default())
            .is_ok());

        // This turns the chain's audio output into Control events.
        let signal_passthrough_uid = orchestrator
            .assign_uid_and_add_entity(
                &sidechain_track_uid,
                factory
                    .new_entity(
                        &EntityKey::from("signal-amplitude-inverted-passthrough"),
                        Uid::default(),
                    )
                    .unwrap(),
            )
            .unwrap();
        // In this demo, we don't want to hear the kick track.
        assert!(orchestrator
            .assign_uid_and_add_entity(
                &sidechain_track_uid,
                factory
                    .new_entity(&EntityKey::from("mute"), Uid::default())
                    .unwrap()
            )
            .is_ok());

        // Add the lead track that we want to duck.
        let lead_pattern = PatternBuilder::default()
            .note(Note {
                key: MidiNote::C4 as u8,
                range: TimeRange(MusicalTime::START..MusicalTime::new_with_beats(4)),
            })
            .build()
            .unwrap();
        let lead_track_uid = orchestrator.create_track(None).unwrap();
        let mut sequencer = PatternSequencer::default();
        assert!(sequencer
            .record(MidiChannel(1), &lead_pattern, MusicalTime::START)
            .is_ok());
        assert!(orchestrator
            .assign_uid_and_add_entity(&lead_track_uid, Box::new(sequencer))
            .is_ok());
        let lead_synth_uid = orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .unwrap();
        assert!(orchestrator
            .connect_midi_receiver(lead_synth_uid, MidiChannel(1))
            .is_ok());

        let entity = factory
            .new_entity(&EntityKey::from(Gain::ENTITY_KEY), Uid::default())
            .unwrap();
        let gain_ceiling_param_index = entity.control_index_for_name("ceiling").unwrap();
        let gain_uid = orchestrator
            .assign_uid_and_add_entity(&lead_track_uid, entity)
            .unwrap();

        // Link the sidechain control to the synth's gain.
        assert!(orchestrator
            .link_control(signal_passthrough_uid, gain_uid, gain_ceiling_param_index)
            .is_ok());
    }
    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "sidechaining.wav"]
        .iter()
        .collect();
    let mut orchestrator_helper = OrchestratorHelper::new_with(&mut orchestrator);
    assert!(orchestrator_helper.write_to_file(&output_path).is_ok());
}
