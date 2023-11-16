// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    entities::{
        controllers::PatternSequencer, effects::Reverb, instruments::Drumkit, toys::ToySynth,
    },
    prelude::*,
};

fn set_up_drum_track(o: &mut dyn Orchestrates, factory: &EntityFactory) {
    // Create the track and set it to 50% gain, because we'll have two tracks total.
    let track_uid = o.create_track().unwrap();
    o.set_track_output(track_uid, Normal::from(0.5));

    // Rest
    const RR: u8 = 255;

    // Add the drum pattern to the PianoRoll.
    // We need to scope piano_roll to satisfy the borrow checker.
    let drum_pattern = PatternBuilder::default()
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
        .unwrap();

    // Arrange the drum pattern in a new MIDI track's Sequencer.
    let mut sequencer = PatternSequencer::default();
    assert!(sequencer
        .record(MidiChannel(10), &drum_pattern, MusicalTime::START)
        .is_ok());
    assert!(o
        .assign_uid_and_add_entity(&track_uid, Box::new(sequencer))
        .is_ok());

    // Add the drumkit instrument to the track.
    let drumkit_uid = o
        .assign_uid_and_add_entity(
            &track_uid,
            factory
                .new_entity(&EntityKey::from(Drumkit::ENTITY_KEY), Uid::default())
                .unwrap(),
        )
        .unwrap();
    assert!(o
        .connect_midi_receiver(drumkit_uid, MidiChannel(10))
        .is_ok());

    // Add an effect to the track's effect chain.
    let filter_uid = o
        .assign_uid_and_add_entity(
            &track_uid,
            factory
                .new_entity(&EntityKey::from("filter-low-pass-24db"), Uid::default())
                .unwrap(),
        )
        .unwrap();
    assert!(o.set_effect_humidity(filter_uid, Normal::from(0.0)).is_ok());
}

fn set_up_lead_track(o: &mut dyn Orchestrates, factory: &EntityFactory) {
    // Create the track and set it to 50% gain, because we'll have two tracks total.
    let track_uid = o.create_track().unwrap();
    o.set_track_output(track_uid, Normal::from(0.5));

    // Rest
    const RR: u8 = 255;

    // Add the lead pattern to the PianoRoll.
    let scale_pattern = PatternBuilder::default()
        .note_sequence(
            vec![
                60, RR, 62, RR, 64, RR, 65, RR, 67, RR, 69, RR, 71, RR, 72, RR, //
                72, RR, 71, RR, 69, RR, 67, RR, 65, RR, 64, RR, 62, RR, 60, RR, //
            ],
            None,
        )
        .build()
        .unwrap();

    // Arrange the lead pattern in a new MIDI track's Sequencer.
    let mut sequencer = PatternSequencer::default();
    assert!(sequencer
        .record(MidiChannel::default(), &scale_pattern, MusicalTime::START)
        .is_ok());

    assert!(o
        .assign_uid_and_add_entity(&track_uid, Box::new(sequencer))
        .is_ok());

    // Add a synth to play the pattern.
    let synth_uid = o
        .assign_uid_and_add_entity(
            &track_uid,
            factory
                .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
        )
        .unwrap();
    assert!(o
        .connect_midi_receiver(synth_uid, MidiChannel::default())
        .is_ok());

    // Make the synth sound grittier.
    let reverb_uid = o
        .assign_uid_and_add_entity(
            &track_uid,
            factory
                .new_entity(&EntityKey::from(Reverb::ENTITY_KEY), Uid::default())
                .unwrap(),
        )
        .unwrap();
    assert!(o.set_effect_humidity(reverb_uid, Normal::from(0.2)).is_ok());
}

// Demonstrates making a song in Rust. We assume that we knew what the song is
// from the start, so there is no editing -- just programming. Compare the
// edit_song() test, which demonstrates adding elements, changing them, and
// removing them, as you'd expect a GUI DAW to do.
#[test]
fn program_song() {
    let mut factory = EntityFactory::default();
    register_factory_entities(&mut factory);
    register_toy_entities(&mut factory);
    let _ = EntityFactory::initialize(factory);
    let factory = EntityFactory::global();

    let mut orchestrator = Orchestrator::default();

    // Work with just the Orchestrates trait for a while.
    {
        let orchestrator: &mut dyn Orchestrates = &mut orchestrator;

        orchestrator.update_tempo(Tempo(128.0));

        set_up_drum_track(orchestrator, factory);
        set_up_lead_track(orchestrator, factory);
    }

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "simple-song.wav"]
        .iter()
        .collect();
    let mut orchestrator_helper = OrchestratorHelper::new_with(&mut orchestrator);
    assert!(orchestrator_helper.write_to_file(&output_path).is_ok());
}
