// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{entities::controllers::PatternSequencerBuilder, prelude::*};

#[test]
fn edit_song() {
    let _ = EntityFactory::initialize(register_factory_entities(EntityFactory::default()));
    let factory = EntityFactory::global();

    let mut orchestrator = OrchestratorBuilder::default()
        .title(Some("Simple Song (Edits)".to_string()))
        .build()
        .unwrap();

    {
        let orchestrator: &mut dyn Orchestrates = &mut orchestrator;
        let mut piano_roll = PianoRoll::default();

        // Create two MIDI tracks.
        let rhythm_track_uid = orchestrator.create_track().unwrap();
        let lead_track_uid = orchestrator.create_track().unwrap();

        // Prepare the rhythm track first. Create a rhythm pattern, add it to the
        // PianoRoll, and then manipulate it. If we were really doing this in Rust
        // code, it would be simpler to create, manipulate, and then add, rather
        // than create, add, and manipulate, because PianoRoll takes ownership. But
        // in a DAW, we expect that PianoRoll's GUI will do the pattern
        // manipulation, so we're modeling that flow. This requires a bit of scoping
        // to satisfy the borrow checker.
        let drum_pattern = PatternBuilder::default().build().unwrap();
        let drum_pattern_uid = piano_roll.insert(drum_pattern);
        let drum_pattern = {
            let drum_pattern = piano_roll.get_pattern_mut(&drum_pattern_uid).unwrap();

            let mut note = Note {
                key: 60,
                range: MusicalTime::START..(MusicalTime::START + MusicalTime::DURATION_HALF),
            };
            // Add to the pattern.
            drum_pattern.add_note(note.clone());
            // Wait, no, didn't want to do that.
            drum_pattern.remove_note(&note);
            // It should be a kick. Change and then re-add.
            note.key = 35;
            drum_pattern.add_note(note.clone());

            // We don't have to keep removing/re-adding to edit notes. If we can
            // describe them, then we can edit them within the pattern.
            let note = drum_pattern.change_note_key(&note.clone(), 39).unwrap();
            let note = drum_pattern
                .move_note(
                    &note.clone(),
                    note.range.start + MusicalTime::DURATION_BREVE,
                )
                .unwrap();
            let _ = drum_pattern
                .move_and_resize_note(
                    &note.clone(),
                    MusicalTime::START,
                    MusicalTime::DURATION_SIXTEENTH,
                )
                .unwrap();
            drum_pattern.clone()
        };

        // TEMP while we decide if patterns or notes are the basic sequencer unit

        // Pattern is good; add an instrument to the track.
        assert!(orchestrator
            .append_entity(
                &rhythm_track_uid,
                factory
                    .new_entity(&EntityKey::from("toy-instrument"))
                    .unwrap(),
            )
            .is_ok());

        // Arrange the drum pattern.
        assert!(orchestrator
            .append_entity(
                &rhythm_track_uid,
                Box::new(
                    PatternSequencerBuilder::default()
                        .pattern(drum_pattern.clone())
                        .build()
                        .unwrap(),
                )
            )
            .is_ok());

        // Now set up the lead track. We need a pattern; we'll whip up something
        // quickly because we already showed the editing process while making the
        // drum pattern.
        let lead_pattern = PatternBuilder::default()
            .note_sequence(
                vec![
                    60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
                ],
                None,
            )
            .build()
            .unwrap();
        let _ = piano_roll.insert(lead_pattern.clone());

        let welsh_synth_uid = orchestrator
            .append_entity(
                &lead_track_uid,
                factory.new_entity(&EntityKey::from("toy-synth")).unwrap(),
            )
            .unwrap();

        // Hmmm, we don't like the sound of that synth; let's replace it with another.
        let _ = orchestrator.remove_entity(&welsh_synth_uid);
        assert!(orchestrator
            .append_entity(
                &lead_track_uid,
                factory.new_entity(&EntityKey::from("toy-synth")).unwrap()
            )
            .is_ok());

        // That's better, but it needs an effect.
        assert!(orchestrator
            .append_entity(
                &lead_track_uid,
                factory.new_entity(&EntityKey::from("reverb")).unwrap()
            )
            .is_ok());
        // And another.
        let lead_gain_uid = orchestrator
            .append_entity(
                &lead_track_uid,
                factory.new_entity(&EntityKey::from("gain")).unwrap(),
            )
            .unwrap();
        // Sounds better if gain is first in chain.
        let _ = orchestrator.move_effect(lead_gain_uid, 0);

        // Arrange the lead pattern.
        assert!(orchestrator
            .append_entity(
                &lead_track_uid,
                Box::new(
                    PatternSequencerBuilder::default()
                        .pattern(lead_pattern.clone())
                        .build()
                        .unwrap(),
                )
            )
            .is_ok());
    }

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf =
        [env!("CARGO_TARGET_TMPDIR"), "simple-song-with-edits.wav"]
            .iter()
            .collect();
    assert!(orchestrator.write_to_file(&output_path).is_ok());
}
