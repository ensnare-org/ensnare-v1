// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `hello-world` example shows how to use basic crate functionality.

use clap::Parser;
use ensnare::prelude::*;
use ensnare_entities_toy::prelude::*;

/// The program's command-line arguments.
#[derive(clap::Parser, Debug, Default)]
#[clap(author, about, long_about = None)]
struct Args {
    /// Print version and exit
    #[clap(short = 'v', long, value_parser)]
    version: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.version {
        eprintln!("{}", ensnare::app_version());
        return Ok(());
    }

    // The system needs a working buffer for audio.
    let _buffer = [StereoSample::SILENCE; 64];

    // Project contains all the the instruments, controllers, and effects, and
    // their relationships, and uses them to produce a song.
    let mut project = Project::default();

    // It also owns the sample rate and propagates it to the devices that it
    // controls.
    project.update_sample_rate(SampleRate::DEFAULT);

    // It manages a set of Tracks, which are what actually contains musical
    // devices.
    let track_uid = project.create_track(None).unwrap();

    // TODO: add musical content to be played on the default MIDI channel.

    // ToyInstrument is a MIDI instrument that makes simple sounds. Adding an
    // entity to a track forms a chain that sends MIDI, control, and audio data
    // appropriately.
    let synth = ToyInstrument::default();
    let synth_uid = project
        .add_entity(track_uid, Box::new(synth), None)
        .unwrap();
    let _ = project.set_midi_receiver_channel(synth_uid, Some(MidiChannel::default()));

    // An effect takes the edge off the synth.
    let effect = ToyEffect::default();
    let _effect_uid = project
        .add_entity(track_uid, Box::new(effect), None)
        .unwrap();

    // Once everything is set up, export_to_wav() renders an audio stream to disk.
    let _ = project.export_to_wav(std::path::PathBuf::from("output.wav"));

    Ok(())
}
