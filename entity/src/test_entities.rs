// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{factory::EntityFactory, traits::Displays};
use ensnare_core::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerInstrument, IsController, IsEffect, IsInstrument, Metadata,
};
use std::sync::{Arc, Mutex};

/// Registers all [EntityFactory]'s test entities. Test entities are generally
/// simple, and provide instrumentation rather than useful audio functionality.
#[must_use]
pub fn register_test_entities(mut factory: EntityFactory) -> EntityFactory {
    factory.register_entity_with_str_key(TestInstrument::ENTITY_KEY, |_uid| {
        Box::new(TestInstrument::default())
    });
    factory.register_entity_with_str_key(TestController::ENTITY_KEY, |_uid| {
        Box::new(TestController::default())
    });
    factory.register_entity_with_str_key(TestEffect::ENTITY_KEY, |_uid| {
        Box::new(TestEffect::default())
    });

    factory.complete_registration();

    factory
}

/// The smallest possible [IsController].
#[derive(Debug, Default, IsController, Metadata)]
pub struct TestController {
    uid: Uid,
}
impl Displays for TestController {}
impl HandlesMidi for TestController {}
impl Controls for TestController {}
impl Configurable for TestController {}
impl Serializable for TestController {}

/// The smallest possible [IsInstrument].
#[derive(Debug, Default, IsInstrument, Metadata)]
pub struct TestInstrument {
    pub uid: Uid,
    pub sample_rate: SampleRate,
}
impl Displays for TestInstrument {}
impl HandlesMidi for TestInstrument {}
impl Controllable for TestInstrument {}
impl Configurable for TestInstrument {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }
}
impl Serializable for TestInstrument {}
impl Generates<StereoSample> for TestInstrument {
    fn value(&self) -> StereoSample {
        StereoSample::default()
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        values.fill(StereoSample::default())
    }
}
impl Ticks for TestInstrument {}

/// The smallest possible [IsEffect].
#[derive(Debug, Default, IsEffect, Metadata)]
pub struct TestEffect {
    uid: Uid,
}
impl Displays for TestEffect {}
impl Configurable for TestEffect {}
impl Controllable for TestEffect {}
impl Serializable for TestEffect {}
impl TransformsAudio for TestEffect {}

/// An [IsInstrument](ensnare::traits::IsInstrument) that counts how many
/// MIDI messages it has received.
#[derive(Debug, Default, IsInstrument, Metadata)]
pub struct TestInstrumentCountsMidiMessages {
    uid: Uid,
    pub received_midi_message_count: Arc<Mutex<usize>>,
}
impl Generates<StereoSample> for TestInstrumentCountsMidiMessages {
    fn value(&self) -> StereoSample {
        StereoSample::default()
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        values.fill(StereoSample::default())
    }
}
impl Configurable for TestInstrumentCountsMidiMessages {}
impl Controllable for TestInstrumentCountsMidiMessages {}
impl Ticks for TestInstrumentCountsMidiMessages {}
impl HandlesMidi for TestInstrumentCountsMidiMessages {
    fn handle_midi_message(
        &mut self,
        _action: MidiChannel,
        _: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        if let Ok(mut received_count) = self.received_midi_message_count.lock() {
            *received_count += 1;
        }
    }
}
impl Serializable for TestInstrumentCountsMidiMessages {}
impl TestInstrumentCountsMidiMessages {
    pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
        &self.received_midi_message_count
    }
}
impl Displays for TestInstrumentCountsMidiMessages {}

#[derive(Debug, Default, InnerInstrument, InnerConfigurable, IsController, Metadata)]
pub struct TestAudioSource {
    uid: Uid,
    inner: ensnare_cores::TestAudioSource,
}
impl Displays for TestAudioSource {}
impl HandlesMidi for TestAudioSource {}
impl Controls for TestAudioSource {}
impl Serializable for TestAudioSource {}
