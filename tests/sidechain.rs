// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{entities::Gain, prelude::*};
use ensnare_toys::prelude::*;

// Demonstrates sidechaining (which could be considered a kind of automation,
// but it's important enough to put top-level and make sure it's a good
// experience and not merely possible).
#[test]
fn demo_sidechaining() {
    SampleLibrary::set_instance(SampleLibrary::default());
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut project = Project::default();

    // Add the sidechain source track.
    let sidechain_track_uid = project.create_track(None).unwrap();
    let sidechain_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        35, 255, 255, 255, 35, 255, 255, 255, 35, 255, 255, 255, 35, 255, 255, 255,
                    ],
                    None,
                )
                .build()
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .arrange_pattern(
            sidechain_track_uid,
            sidechain_pattern_uid,
            MusicalTime::START
        )
        .is_ok());
    let sidechain_synth_uid = project
        .add_entity(
            sidechain_track_uid,
            factory
                .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .set_midi_receiver_channel(sidechain_synth_uid, Some(MidiChannel::default()))
        .is_ok());

    // This turns the chain's audio output into Control events.
    let signal_passthrough_uid = project
        .add_entity(
            sidechain_track_uid,
            factory
                .new_entity(
                    &EntityKey::from("signal-amplitude-inverted-passthrough"),
                    Uid::default(),
                )
                .unwrap(),
            None,
        )
        .unwrap();

    // In this demo, we don't want to hear the kick track.
    project.set_track_output(sidechain_track_uid, Normal::zero());

    // Add the lead track that we want to duck.
    let lead_track_uid = project.create_track(None).unwrap();
    let lead_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note(Note::new_with_midi_note(
                    MidiNote::C4,
                    MusicalTime::START,
                    MusicalTime::new_with_beats(4),
                ))
                .build()
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .arrange_pattern(lead_track_uid, lead_pattern_uid, MusicalTime::START)
        .is_ok());
    let lead_synth_uid = project
        .add_entity(
            lead_track_uid,
            factory
                .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .set_midi_receiver_channel(lead_synth_uid, Some(MidiChannel(1)))
        .is_ok());

    let entity = factory
        .new_entity(&EntityKey::from(Gain::ENTITY_KEY), Uid::default())
        .unwrap();
    let gain_ceiling_param_index = entity.control_index_for_name("ceiling").unwrap();
    let gain_uid = project.add_entity(lead_track_uid, entity, None).unwrap();

    // Link the sidechain control to the synth's gain.
    assert!(project
        .link(signal_passthrough_uid, gain_uid, gain_ceiling_param_index)
        .is_ok());

    let output_prefix: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "sidechaining"]
        .iter()
        .collect();
    assert!(project.save_and_export(output_prefix).is_ok());
}
