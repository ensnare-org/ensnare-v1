// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::prelude::*;

fn set_up_drum_track(o: &mut dyn Orchestrates, factory: &EntityFactory) {
    // Add the drum pattern to the PianoRoll.
    // We need to scope piano_roll to satisfy the borrow checker.
    let drum_pattern = PatternBuilder::default()
        .note_sequence(
            vec![
                35, 255, 255, 255, 35, 255, 255, 255, 35, 255, 255, 255, 35, 255, 255, 255,
            ],
            None,
        )
        .note_sequence(
            vec![
                255, 255, 255, 255, 39, 255, 255, 255, 255, 255, 255, 255, 39, 255, 255, 255,
            ],
            None,
        )
        .note_sequence(
            vec![
                // Bug: if we do note on every 16th, we get only the first one
                42, 255, 42, 255, 42, 255, 42, 255, 42, 255, 42, 255, 42, 255, 42, 255,
            ],
            None,
        )
        .build()
        .unwrap();

    // Arrange the drum pattern in a new MIDI track's Sequencer. By default, the
    // Sequencer emits events on MIDI channel 0.
    let track_uid = o.create_track().unwrap();
    let mut sequencer = Box::new(ESSequencerBuilder::default().build().unwrap());
    factory.assign_entity_uid(sequencer.as_mut());
    sequencer.insert_pattern(&drum_pattern, MusicalTime::START);
    o.append_entity(&track_uid, sequencer);

    // Add the drumkit instrument to the track. By default, it listens on MIDI channel 0.
    assert!(o
        .append_entity(
            &track_uid,
            factory
                .new_entity(&EntityKey::from("toy-instrument"))
                .unwrap(),
        )
        .is_ok());

    // Add an effect to the track's effect chain.
    let filter_uid = o
        .append_entity(
            &track_uid,
            factory
                .new_entity(&EntityKey::from("filter-low-pass-24db"))
                .unwrap(),
        )
        .unwrap();
    o.set_humidity(filter_uid, Normal::from(0.2));
}

fn set_up_lead_track(o: &mut dyn Orchestrates, factory: &EntityFactory) {
    // Add the lead pattern to the PianoRoll.
    let scale_pattern = PatternBuilder::default()
        .note_sequence(
            vec![
                60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
            ],
            None,
        )
        .build()
        .unwrap();

    // Arrange the lead pattern in a new MIDI track's Sequencer.
    let track_uid = o.create_track().unwrap();
    let mut sequencer = Box::new(ESSequencerBuilder::default().build().unwrap());
    factory.assign_entity_uid(sequencer.as_mut());
    let _ = sequencer.insert_pattern(&scale_pattern, MusicalTime::START);
    assert!(o.append_entity(&track_uid, sequencer).is_ok());

    // Add a synth to play the pattern.
    assert!(o
        .append_entity(
            &track_uid,
            factory.new_entity(&EntityKey::from("toy-synth")).unwrap()
        )
        .is_ok());

    // Make the synth sound better.
    let reverb_uid = o
        .append_entity(
            &&track_uid,
            factory.new_entity(&EntityKey::from("reverb")).unwrap(),
        )
        .unwrap();
    assert!(o.set_humidity(reverb_uid, Normal::from(0.2)).is_ok());
}

// Demonstrates making a song in Rust. We assume that we knew what the song is
// from the start, so there is no editing -- just programming. Compare the
// edit_song() test, which demonstrates adding elements, changing them, and
// removing them, as you'd expect a GUI DAW to do.
#[test]
fn program_song() {
    let factory = register_factory_entities(EntityFactory::default());
    let mut orchestrator = OrchestratorBuilder::default()
        .title(Some("Simple Song".to_string()))
        .build()
        .unwrap();

    {
        let orchestrator: &mut dyn Orchestrates = &mut orchestrator;

        orchestrator.update_tempo(Tempo(128.0));

        set_up_drum_track(orchestrator, &factory);
        set_up_lead_track(orchestrator, &factory);
    }

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [env!("CARGO_TARGET_TMPDIR"), "simple-song.wav"]
        .iter()
        .collect();
    assert!(orchestrator.write_to_file(&output_path).is_ok());
}
