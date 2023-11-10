// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    entities::effects::{Gain, Reverb},
    prelude::*,
};
use ensnare_factory_entities::controllers::PatternSequencer;
use ensnare_toy_entities::prelude::*;
use std::sync::RwLock;

#[test]
fn edit_song() {
    let mut factory = EntityFactory::default();
    register_factory_entities(&mut factory);
    register_toy_entities(&mut factory);
    factory.complete_registration();
    let _ = EntityFactory::initialize(factory);
    let factory = EntityFactory::global();

    let mut orchestrator = Orchestrator::default();
    let mut piano_roll = PianoRoll::default();

    {
        let orchestrator: &mut dyn Orchestrates = &mut orchestrator;

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

        // Pattern is good; add an instrument to the track.
        assert!(orchestrator
            .assign_uid_and_add_entity(
                &rhythm_track_uid,
                factory
                    .new_entity(&EntityKey::from(ToyInstrument::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .is_ok());

        // Arrange the drum pattern.
        let mut sequencer = PatternSequencer::default();
        assert!(sequencer
            .record(MidiChannel(10), &drum_pattern, MusicalTime::START)
            .is_ok());
        assert!(orchestrator
            .assign_uid_and_add_entity(&rhythm_track_uid, Box::new(sequencer))
            .is_ok());

        // Rest
        const RR: u8 = 255;

        // Now set up the lead track. We need a pattern; we'll whip up something
        // quickly because we already showed the editing process while making the
        // drum pattern.
        let lead_pattern = PatternBuilder::default()
            .note_sequence(
                vec![
                    60, RR, 62, RR, 64, RR, 65, RR, 67, RR, 69, RR, 71, RR, 72, RR,
                ],
                None,
            )
            .build()
            .unwrap();
        let _ = piano_roll.insert(lead_pattern.clone());

        let welsh_synth_uid = orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .unwrap();

        // Hmmm, we don't like the sound of that synth; let's replace it with another.
        let _ = orchestrator.remove_entity(&welsh_synth_uid);
        assert!(orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap()
            )
            .is_ok());

        // That's better, but it needs an effect.
        assert!(orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(Reverb::ENTITY_KEY), Uid::default())
                    .unwrap()
            )
            .is_ok());
        // And another.
        let lead_gain_uid = orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .unwrap();
        // Sounds better if gain is first in chain.
        let _ = orchestrator.set_effect_position(lead_gain_uid, 0);

        // Arrange the lead pattern.
        let mut sequencer = PatternSequencer::default();
        assert!(sequencer
            .record(MidiChannel::default(), &lead_pattern, MusicalTime::START)
            .is_ok());

        assert!(orchestrator
            .assign_uid_and_add_entity(&lead_track_uid, Box::new(sequencer))
            .is_ok());
    }

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [
        env!("CARGO_TARGET_TMPDIR"),
        "simple-song-with-edits-new.wav",
    ]
    .iter()
    .collect();
    let mut orchestrator_helper = OrchestratorHelper::new_with(&mut orchestrator);
    assert!(orchestrator_helper.write_to_file(&output_path).is_ok());
}

#[ignore = "reason"]
#[test]
fn old_edit_song() {
    let mut factory = EntityFactory::default();
    register_factory_entities(&mut factory);
    register_toy_entities(&mut factory);
    factory.complete_registration();
    let _ = EntityFactory::initialize(factory);
    let factory = EntityFactory::global();

    let mut orchestrator = Orchestrator::default();
    // let piano_roll = std::sync::Arc::clone(&orchestrator.piano_roll);
    let piano_roll = std::sync::Arc::new(RwLock::new(PianoRoll::default()));

    // Rest
    const RR: u8 = 255;

    // Create drum and lead patterns.
    let drum_pattern_uid = piano_roll
        .write()
        .unwrap()
        .insert(PatternBuilder::default().build().unwrap());
    let lead_pattern_uid = piano_roll.write().unwrap().insert(
        PatternBuilder::default()
            .note_sequence(
                vec![
                    60, RR, 62, RR, 64, RR, 65, RR, 67, RR, 69, RR, 71, RR, 72, RR,
                ],
                None,
            )
            .build()
            .unwrap(),
    );

    // Create two MIDI tracks.
    let drum_track_uid = orchestrator.new_midi_track().unwrap();
    let lead_track_uid = orchestrator.new_midi_track().unwrap();

    // Now switch to working with just the Orchestrates trait to see how
    // powerful it is.
    let (rhythm_track_uid, lead_track_uid) = {
        let orchestrator: &mut dyn Orchestrates = &mut orchestrator;
        let mut piano_roll_writeable = piano_roll.write().unwrap();

        // If we were really doing this in Rust code, it would be simpler to
        // create, manipulate, and then add, rather than create, add, and
        // manipulate, because PianoRoll takes ownership. But in a DAW, we
        // expect that PianoRoll's GUI will do the pattern manipulation, so
        // we're modeling that flow. This requires a bit of scoping to satisfy
        // the borrow checker.
        {
            let drum_pattern = piano_roll_writeable
                .get_pattern_mut(&drum_pattern_uid)
                .unwrap();

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
        }

        // Pattern is good; add an instrument to the track.
        assert!(orchestrator
            .assign_uid_and_add_entity(
                &drum_track_uid,
                factory
                    .new_entity(&EntityKey::from(ToyInstrument::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .is_ok());

        // Now set up the lead track. We need a pattern; we'll whip up something
        // quickly because we already showed the editing process while making the
        // drum pattern.

        let welsh_synth_uid = orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .unwrap();

        // Hmmm, we don't like the sound of that synth; let's replace it with another.
        let _ = orchestrator.remove_entity(&welsh_synth_uid);
        assert!(orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                    .unwrap()
            )
            .is_ok());

        // That's better, but it needs an effect.
        assert!(orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(Reverb::ENTITY_KEY), Uid::default())
                    .unwrap()
            )
            .is_ok());
        // And another.
        let lead_gain_uid = orchestrator
            .assign_uid_and_add_entity(
                &lead_track_uid,
                factory
                    .new_entity(&EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                    .unwrap(),
            )
            .unwrap();
        // Sounds better if gain is first in chain.
        let _ = orchestrator.set_effect_position(lead_gain_uid, 0);

        (drum_track_uid, lead_track_uid)
    };

    // Back to concrete Orchestrator, which has a LivePatternSequencer. Arrange
    // the two patterns.
    assert!(orchestrator
        .add_pattern_to_track(&rhythm_track_uid, &drum_pattern_uid, MusicalTime::START)
        .is_ok());
    assert!(orchestrator
        .add_pattern_to_track(&lead_track_uid, &lead_pattern_uid, MusicalTime::START)
        .is_ok());

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [
        env!("CARGO_TARGET_TMPDIR"),
        "simple-song-with-edits-old.wav",
    ]
    .iter()
    .collect();
    let mut orchestrator_helper = OrchestratorHelper::new_with(&mut orchestrator);
    assert!(orchestrator_helper.write_to_file(&output_path).is_ok());
}
