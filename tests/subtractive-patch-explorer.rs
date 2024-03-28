// Copyright (c) 2024 Mike Tsao. All rights reserved.

use ensnare::{
    cores::instruments::SubtractiveSynthCore, entities::SubtractiveSynth, prelude::*, util::Rng,
};
use std::{io, path::PathBuf};

fn render_subtractive_patches() -> anyhow::Result<()> {
    let paths = std::fs::read_dir(PathBuf::from("assets/patches/subtractive/"))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let output_prefix: std::path::PathBuf =
        [env!("CARGO_TARGET_TMPDIR"), "patch-explorer", "subtractive"]
            .iter()
            .collect();
    std::fs::create_dir(&output_prefix)?;
    for path in paths {
        let mut project = Project::default();
        let track_uid = project.create_track(None)?;

        let synth =
            SubtractiveSynth::new_with(Uid::default(), SubtractiveSynthCore::load_patch(&path)?);
        let _synth_uid = project.add_entity(track_uid, Box::new(synth), None)?;

        let mut rng = Rng::default();
        let pattern = PatternBuilder::default()
            .note_sequence(
                vec![
                    60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
                ],
                None,
            )
            .random(&mut rng)
            .build()
            .unwrap();
        let pattern_uid = project.add_pattern(pattern, None)?;
        let _arrangement_uid =
            project.arrange_pattern(track_uid, pattern_uid, None, MusicalTime::START)?;

        let mut output_path = output_prefix.clone();
        if let Some(filename) = path.file_name() {
            output_path.push(filename);
            output_path.set_extension("wav");
            project.export_to_wav(output_path.into())?;
        }
    }
    Ok(())
}

#[test]
fn do_renders() {
    let _ = render_subtractive_patches();
}
