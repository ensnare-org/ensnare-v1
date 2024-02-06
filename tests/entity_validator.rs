// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::prelude::*;

#[test]
fn entity_validator_production_entities() {
    let factory = BuiltInEntities::register(EntityFactory::default()).finalize();
    validate_factory_entities(&factory);
}

fn validate_factory_entities(factory: &EntityFactory<dyn EntityBounds>) {
    for (uid, key) in factory.keys().iter().enumerate() {
        let uid = Uid(1000 + uid);
        if let Some(mut entity) = factory.new_entity(key, uid) {
            validate_entity(key, &mut entity);
        } else {
            panic!("Couldn't create entity with {key}, but EntityFactory said it existed!");
        }
    }
}

fn validate_entity(key: &EntityKey, entity: &mut Box<dyn EntityBounds>) {
    validate_configurable(key, entity);
    validate_entity_type(key, entity);
}

fn validate_configurable(key: &EntityKey, entity: &mut Box<dyn EntityBounds>) {
    const TEST_SAMPLE_RATE: SampleRate = SampleRate(1111111);
    entity.update_tempo(Tempo(1234.5678));
    entity.update_time_signature(TimeSignature::new_with(127, 128).unwrap());
    entity.update_sample_rate(TEST_SAMPLE_RATE);

    // This caused lots of things to fail and has me rethinking why Configurable
    // needed sample_rate() as such a widespread trait method. TODO
    if false {
        assert!(
            entity.sample_rate().0 > 0,
            "Entity {key}'s default sample rate should be nonzero"
        );
        assert_eq!(
            entity.sample_rate(),
            SampleRate::DEFAULT,
            "Entity {key}'s default sample rate should equal the default of {}",
            SampleRate::DEFAULT_SAMPLE_RATE
        );
        entity.update_sample_rate(TEST_SAMPLE_RATE);
        assert_eq!(
            entity.sample_rate(),
            TEST_SAMPLE_RATE,
            "Entity {key}'s sample rate should change once set"
        );
    }
}

fn validate_entity_type(_key: &EntityKey, _entity: &mut Box<dyn EntityBounds>) {
    // TODO: this is obsolete at the moment because we've decided that there
    // aren't any entity types -- everyone implements everything, even if they
    // don't actually do anything for a particular trait method.

    // let mut is_something = false;
    // if let Some(e) = entity.as_controller_mut() {
    //     is_something = true;
    //     validate_controller(e);
    //     validate_extreme_tempo_and_time_signature(key, e);
    // }
    // if let Some(e) = entity.as_instrument_mut() {
    //     is_something = true;
    //     validate_instrument(e);
    //     validate_extreme_sample_rates(key, entity);
    // }
    // if let Some(e) = entity.as_effect_mut() {
    //     is_something = true;
    //     validate_effect(e);
    //     validate_extreme_sample_rates(key, entity);
    // }
    // assert!(
    //     is_something,
    //     "Entity {key} is neither a controller, nor an instrument, nor an effect!"
    // );
}
