// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    entities_future::{Gain, Reverb},
    prelude::*,
};
use ensnare_entities_toy::prelude::*;

// Demonstrates use of aux buses.
#[test]
fn aux_bus() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut project = Project::default();

    let track_uid_1 = project.create_track(None).unwrap();
    let track_uid_2 = project.create_track(None).unwrap();
    let aux_track_uid = project.create_track(None).unwrap();

    let synth_pattern_uid_1 = project
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

    let synth_pattern_uid_2 = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        84, 255, 83, 255, 81, 255, 79, 255, 77, 255, 76, 255, 74, 255, 72, 255,
                    ],
                    None,
                )
                .build()
                .unwrap(),
            None,
        )
        .unwrap();

    let synth_uid_1 = {
        assert!(project
            .arrange_pattern(track_uid_1, synth_pattern_uid_1, MusicalTime::START)
            .is_ok());

        // Even though we want the effect to be placed after the instrument in
        // the audio chain, we can add the effect before we add the instrument.
        // This is because the processing order is always controllers,
        // instruments, effects.
        assert!(project
            .add_entity(
                track_uid_1,
                factory
                    .new_entity(EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                    .unwrap(),
                None
            )
            .is_ok());
        project
            .add_entity(
                track_uid_1,
                factory
                    .new_entity(EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap(),
                None,
            )
            .unwrap()
    };
    assert!(project
        .set_midi_receiver_channel(synth_uid_1, Some(MidiChannel::default()))
        .is_ok());

    let synth_uid_2 = {
        assert!(project
            .arrange_pattern(track_uid_2, synth_pattern_uid_2, MusicalTime::START)
            .is_ok());
        assert!(project
            .add_entity(
                track_uid_2,
                factory
                    .new_entity(EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                    .unwrap(),
                None
            )
            .is_ok());
        project
            .add_entity(
                track_uid_2,
                factory
                    .new_entity(EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap(),
                None,
            )
            .unwrap()
    };
    assert!(project
        .set_midi_receiver_channel(synth_uid_2, Some(MidiChannel(1)))
        .is_ok());

    let _effect_uid_1 = {
        project
            .add_entity(
                aux_track_uid,
                factory
                    .new_entity(EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                    .unwrap(),
                None,
            )
            .unwrap();
        project
            .add_entity(
                aux_track_uid,
                factory
                    .new_entity(EntityKey::from(Reverb::ENTITY_KEY), Uid::default())
                    .unwrap(),
                None,
            )
            .unwrap()
    };

    let _ = project.add_send(track_uid_1, aux_track_uid, Normal::from(1.0));
    let _ = project.add_send(track_uid_2, aux_track_uid, Normal::from(1.0));

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "aux-bus.wav"]
        .iter()
        .collect();
    assert!(project.export_to_wav(output_path).is_ok());
}
