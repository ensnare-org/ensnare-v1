// Copyright (c) 2023 Mike Tsao. All rights reserved.

pub(crate) use controllers::ToyControllerCore;
pub(crate) use effects::ToyEffectCore;
pub(crate) use instruments::ToyInstrumentCore;
pub(crate) use synth::ToySynthCore;

mod controllers;
mod effects;
mod instruments;
mod synth;

#[cfg(test)]
pub mod tests {
    use super::*;
    use ensnare::prelude::*;

    pub trait GeneratesStereoSampleAndHandlesMidi: Generates<StereoSample> + HandlesMidi {}

    /// Checks that the given instrument is silent to start and then makes a
    /// sound within 16 samples after a MIDI note-on.
    pub fn check_instrument(instrument: &mut dyn GeneratesStereoSampleAndHandlesMidi) {
        let mut buffer = [StereoSample::default(); 16];

        instrument.generate(&mut buffer);
        assert!(
            buffer.iter().all(|s| *s == StereoSample::SILENCE),
            "note-off should be silent"
        );

        instrument.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_on(1, 1),
            &mut |_, _| {},
        );
        instrument.generate(&mut buffer);
        assert!(
            buffer.iter().any(|s| *s != StereoSample::SILENCE),
            "note-on should produce sound"
        );
    }

    // TODO: restore tests that test basic trait behavior, then figure out how
    // to run everyone implementing those traits through that behavior. For now,
    // this one just tests that a generic instrument doesn't panic when accessed
    // for non-consecutive time slices.
    #[test]
    fn sources_audio_random_access() {
        let mut instrument = instruments::ToyInstrumentCore::default();
        let mut buffer = [StereoSample::default(); 100];
        instrument.generate(&mut buffer);
    }
}
