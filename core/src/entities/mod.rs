// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Built-in musical instruments and supporting components.

pub mod controllers;

// Instruments play sounds. They implement the
// [IsInstrument](ensnare_core::traits::IsInstrument) trait, which means that
// they respond to MIDI and produce [StereoSamples](ensnare_core::StereoSample).
// Examples of instruments are [Sampler](crate::instruments::Sampler) and
// [WelshSynth](crate::instruments::WelshSynth).
//pub mod instruments;
