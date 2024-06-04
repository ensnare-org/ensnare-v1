// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::prelude::*;
use ensnare_toys::prelude::*;
use ensnare_v1::{
    entities::{Drumkit, Gain},
    prelude::*,
};

// Demonstrates sidechaining (which could be considered a kind of automation,
// but it's important enough to put top-level and make sure it's a good
// experience and not merely possible).
//
// There are two tracks: lead and rhythm. The rhythm track's output should
// inversely affect the lead track's gain, so that the lead track makes acoustic
// "room" for the rhythm. (Also called "ducking," as in the lead should "duck"
// out of the way of the rhythm.) Turning off the rhythm track's output, rather
// than mixing it into the final track, should leave empty spaces in the lead
// output and make it easier to see the effect in Audacity.
//
// TODO: this isn't a useful form of sidechaining, because it uses a
// sample-by-sample correction, which basically means that the lead track gets
// all sorts of transients and awfulness, rather than being nicely ducked. The
// missing piece is an envelope that adjusts relatively slowly to the amplitude
// of the source signal.
#[test]
fn demo_sidechaining() {
    Paths::set_instance(Paths::default());
    init_sample_libraries();
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut project = Project::default();

    // Add the sidechain source track.
    let sidechain_track_uid = project.create_track().unwrap();
    project.set_track_midi_channel(sidechain_track_uid, MidiChannel::DRUM);
    project.set_track_output(sidechain_track_uid, Normal::from(0.5));

    // Drumkit notes
    const RR: u8 = 255; // Rest
    const KICK: u8 = 36;

    let drum_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        KICK, RR, RR, RR, KICK, RR, RR, RR, KICK, RR, RR, RR, KICK, RR, RR,
                        RR, //
                        KICK, RR, RR, RR, KICK, RR, RR, RR, KICK, RR, RR, RR, KICK, RR, RR,
                        RR, //
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
            drum_pattern_uid,
            None,
            MusicalTime::START
        )
        .is_ok());
    let _drum_instrument_uid = project
        .add_entity(
            sidechain_track_uid,
            factory
                .new_entity(&EntityKey::from(Drumkit::ENTITY_KEY), Uid::default())
                .unwrap(),
        )
        .unwrap();

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
        )
        .unwrap();

    // In this demo, we don't want to hear the kick track.
    project.set_track_output(sidechain_track_uid, Normal::zero());

    // Add the lead track that we want to duck.
    let lead_track_uid = project.create_track().unwrap();
    let lead_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note(Note::new_with_midi_note(
                    MidiNote::C4,
                    MusicalTime::START,
                    MusicalTime::new_with_beats(4), // Long duration
                ))
                .build()
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .arrange_pattern(lead_track_uid, lead_pattern_uid, None, MusicalTime::START)
        .is_ok());
    let _lead_synth_uid = project
        .add_entity(
            lead_track_uid,
            factory
                .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
        )
        .unwrap();

    let entity = factory
        .new_entity(&EntityKey::from(Gain::ENTITY_KEY), Uid::default())
        .unwrap();
    let gain_ceiling_param_index = entity.control_index_for_name("ceiling").unwrap();
    let gain_uid = project.add_entity(lead_track_uid, entity).unwrap();

    // Link the sidechain control to the synth's gain.
    assert!(project
        .link(signal_passthrough_uid, gain_uid, gain_ceiling_param_index)
        .is_ok());

    let output_prefix: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "sidechaining"]
        .iter()
        .collect();
    assert!(project.save_and_export(output_prefix).is_ok());
}
