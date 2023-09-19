// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Shows how to use basic crate functionality. To see it print its version, try
//! `cargo run --example hello_world -- -v`.

use anyhow;
use clap::Parser;
use ensnare::core::Sample;

/// The program's command-line arguments.
#[derive(Parser, Debug, Default)]
#[clap(author, about, long_about = None)]
struct Args {
    /// Print version and exit
    #[clap(short = 'v', long, value_parser)]
    version: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.version {
        eprintln!("0.0");
        return Ok(());
    }

    let _ = Sample::default();

    Ok(())
}
