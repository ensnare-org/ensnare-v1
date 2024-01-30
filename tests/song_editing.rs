// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::{
    entities::{
        effects::{Gain, Reverb},
        toys::{ToyInstrument, ToySynth},
    },
    prelude::*,
};
use ensnare_entities::BuiltInEntities;

#[test]
fn edit_song() {
    let factory =
        ToyEntities::register(BuiltInEntities::register(EntityFactory::default())).finalize();

    let mut project = Project::default();

    // Create two MIDI tracks.
    let rhythm_track_uid = project.create_track(None).unwrap();
    let lead_track_uid = project.create_track(None).unwrap();

    // Prepare the rhythm track first. Create a rhythm pattern, add it to the
    // Composer, and then manipulate it. If we were really doing this in Rust
    // code, it would be simpler to create, manipulate, and then add, rather
    // than create, add, and manipulate, because Composer takes ownership. But
    // in a DAW, we expect that Composer's GUI will do the pattern
    // manipulation, so we're modeling that flow. This requires a bit of scoping
    // to satisfy the borrow checker.
    let drum_pattern = PatternBuilder::default().build().unwrap();
    let drum_pattern_uid = project.add_pattern(drum_pattern, None).unwrap();
    {
        let drum_pattern = project.pattern_mut(drum_pattern_uid).unwrap();

        let mut note = Note::new_with(60, MusicalTime::START, MusicalTime::DURATION_HALF);
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
                note.range.0.start + MusicalTime::DURATION_BREVE,
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

    // Pattern is good; add an instrument to the track. (This should be
    // Drumkit, but there are TODO reasons why it isn't.)
    let drumkit_uid = project
        .add_entity(
            rhythm_track_uid,
            factory
                .new_entity(EntityKey::from(ToyInstrument::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .set_midi_receiver_channel(drumkit_uid, Some(MidiChannel(10)))
        .is_ok());

    // Arrange the drum pattern.
    assert!(project
        .arrange_pattern(rhythm_track_uid, drum_pattern_uid, MusicalTime::START)
        .is_ok());

    // Rest
    const RR: u8 = 255;

    // Now set up the lead track. We need a pattern; we'll whip up something
    // quickly because we already showed the editing process while making the
    // drum pattern.
    let lead_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        60, RR, 62, RR, 64, RR, 65, RR, 67, RR, 69, RR, 71, RR, 72, RR,
                    ],
                    None,
                )
                .build()
                .unwrap(),
            None,
        )
        .unwrap();

    let welsh_synth_uid = project
        .add_entity(
            lead_track_uid,
            factory
                .new_entity(EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    assert!(project
        .set_midi_receiver_channel(welsh_synth_uid, Some(MidiChannel::default()))
        .is_ok());

    // Hmmm, we don't like the sound of that synth; let's replace it with another.
    let _ = project.remove_entity(welsh_synth_uid);
    assert!(project
        .add_entity(
            lead_track_uid,
            factory
                .new_entity(EntityKey::from(ToySynth::ENTITY_KEY), Uid::default())
                .unwrap(),
            None
        )
        .is_ok());

    // That's better, but it needs an effect.
    assert!(project
        .add_entity(
            lead_track_uid,
            factory
                .new_entity(EntityKey::from(Reverb::ENTITY_KEY), Uid::default())
                .unwrap(),
            None
        )
        .is_ok());
    // And another.
    let lead_gain_uid = project
        .add_entity(
            lead_track_uid,
            factory
                .new_entity(EntityKey::from(Gain::ENTITY_KEY), Uid::default())
                .unwrap(),
            None,
        )
        .unwrap();
    // Sounds better if gain is second in chain (index 1, after the synth).
    let _ = project.move_entity(lead_gain_uid, None, Some(1));

    // Arrange the lead pattern.
    assert!(project
        .arrange_pattern(lead_track_uid, lead_pattern_uid, MusicalTime::START)
        .is_ok());

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf = [
        env!("CARGO_TARGET_TMPDIR"),
        "simple-song-with-edits-new.wav",
    ]
    .iter()
    .collect();
    assert!(project.export_to_wav(output_path).is_ok());
}
