// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    entities::{BiQuadFilterLowPass24db, Drumkit, Reverb},
    prelude::*,
};
use ensnare_toys::prelude::*;

fn set_up_drum_track(project: &mut Project, factory: &EntityFactory<dyn EntityBounds>) {
    // Create the track and set it to 50% gain, because we'll have two tracks total.
    let track_uid = project.create_track(None).unwrap();
    project.set_track_output(track_uid, Normal::from(0.5));

    // Rest
    const RR: u8 = 255;

    // Add the drum pattern to the Composer.
    let drum_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        35, RR, RR, RR, 35, RR, RR, RR, 35, RR, RR, RR, 35, RR, RR, RR, //
                        35, RR, RR, RR, 35, RR, RR, RR, 35, RR, RR, RR, 35, RR, RR, RR, //
                    ],
                    None,
                )
                .note_sequence(
                    vec![
                        RR, RR, RR, RR, 39, RR, RR, RR, RR, RR, RR, RR, 39, RR, RR, RR, //
                        RR, RR, RR, RR, 39, RR, RR, RR, RR, RR, RR, RR, 39, RR, RR, RR, //
                    ],
                    None,
                )
                .note_sequence(
                    vec![
                        // Bug: if we do note on every 16th, we get only the first one
                        42, RR, 42, RR, 42, RR, 42, RR, 42, RR, 42, RR, 42, RR, 42, RR, //
                        42, RR, 42, RR, 42, RR, 42, RR, 42, RR, 42, RR, 42, RR, 42, RR, //
                    ],
                    None,
                )
                .build()
                .unwrap(),
            None,
        )
        .unwrap();

    // Arrange the drum pattern in the MIDI track.
    let _ = project.arrange_pattern(track_uid, drum_pattern_uid, MusicalTime::START);

    // Add the drumkit instrument to the track.
    let drumkit_uid = project
        .add_entity(
            track_uid,
            factory
                .new_entity(&EntityKey::from(Drumkit::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .set_midi_receiver_channel(drumkit_uid, Some(MidiChannel(10)))
        .is_ok());

    // Add an effect to the track's effect chain.
    let filter_uid = project
        .add_entity(
            track_uid,
            factory
                .new_entity(
                    &EntityKey::from(BiQuadFilterLowPass24db::ENTITY_KEY),
                    Uid::default(),
                )
                .unwrap(),
            None,
        )
        .unwrap();
    project.set_humidity(filter_uid, Normal::from(0.0));
}

fn set_up_lead_track(project: &mut Project, factory: &EntityFactory<dyn EntityBounds>) {
    // Create the track and set it to 50% gain, because we'll have two tracks total.
    let track_uid = project.create_track(None).unwrap();
    project.set_track_output(track_uid, Normal::from(0.5));

    // Rest
    const RR: u8 = 255;

    // Add the lead pattern to the Composer.
    let scale_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        60, RR, 62, RR, 64, RR, 65, RR, 67, RR, 69, RR, 71, RR, 72, RR, //
                        72, RR, 71, RR, 69, RR, 67, RR, 65, RR, 64, RR, 62, RR, 60, RR, //
                    ],
                    None,
                )
                .build()
                .unwrap(),
            None,
        )
        .unwrap();

    // Arrange the lead pattern.
    assert!(project
        .arrange_pattern(track_uid, scale_pattern_uid, MusicalTime::START)
        .is_ok());

    // Add a synth to play the pattern.
    let synth_uid = project
        .add_entity(
            track_uid,
            factory
                .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .set_midi_receiver_channel(synth_uid, Some(MidiChannel::default()))
        .is_ok());

    // Make the synth sound grittier.
    let reverb_uid = project
        .add_entity(
            track_uid,
            factory
                .new_entity(&EntityKey::from(Reverb::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    project.set_humidity(reverb_uid, Normal::from(0.2));
}

// Demonstrates making a song in Rust. We assume that we knew what the song is
// from the start, so there is no editing -- just programming. Compare the
// edit_song() test, which demonstrates adding elements, changing them, and
// removing them, as you'd expect a GUI DAW to do.
#[test]
fn program_song() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut project = Project::default();

    project.update_tempo(Tempo(128.0));

    set_up_drum_track(&mut project, &factory);
    set_up_lead_track(&mut project, &factory);

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "simple-song.wav"]
        .iter()
        .collect();
    assert!(project.export_to_wav(output_path).is_ok());
}
