// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `hello-world` example shows how to use basic crate functionality.

use clap::Parser;
use ensnare::{
    entities::{effects::ToyEffect, instruments::ToyInstrument},
    prelude::*,
};

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

    // ToyInstrument is a MIDI instrument that makes simple sounds.
    let synth = ToyInstrument::default();

    // An effect takes the edge off the synth.
    let effect = ToyEffect::default();

    // Orchestrator understands the relationships among the instruments,
    // controllers, and effects, and uses them to produce a song.
    let mut orchestrator = Orchestrator::default();

    // Orchestrator owns the sample rate and propagates it to the devices
    // that it controls.
    orchestrator.update_sample_rate(SampleRate::DEFAULT);

    // An Orchestrator manages a set of Tracks, which are what actually contains
    // musical devices.
    let track_uid = orchestrator.new_midi_track().unwrap();

    // The sequencer sends MIDI commands to the synth. Each MIDI track
    // automatically includes one. There are lots of different ways to populate
    // the sequencer with notes.
    // TODO - not working yet
    // let mut sequencer = track.sequencer_mut();
    // sequencer.append_note(&Note::new_with_midi_note(
    //     MidiNote::A4,
    //     MusicalTime::START,
    //     MusicalTime::DURATION_QUARTER,
    // ));

    // Adding an entity to a track forms a chain that sends MIDI, control, and
    // audio data appropriately.
    orchestrator
        .add_entity(&track_uid, Box::new(synth))
        .unwrap();
    orchestrator
        .add_entity(&track_uid, Box::new(effect))
        .unwrap();

    // Once everything is set up, the orchestrator renders an audio stream.
    let mut orchestrator_helper = OrchestratorHelper::new_with(&mut orchestrator);
    let _ = orchestrator_helper.write_to_file(&std::path::PathBuf::from("output.wav"));

    Ok(())
}
