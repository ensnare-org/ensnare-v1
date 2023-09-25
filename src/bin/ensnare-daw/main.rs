// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A digital audio workstation.

use ensnare::prelude::*;

fn main() -> anyhow::Result<()> {
    let _ = Sample::default();
    eprintln!("I make sound, therefore I exist.");
    anyhow::Ok(())
}
