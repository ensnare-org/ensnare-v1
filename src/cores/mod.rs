// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Cores are basic musical devices.

pub use controllers::*;
pub use effects::*;
pub use instruments::*;

pub mod controllers;
pub mod effects;
pub mod instruments;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{control::ControlIndex, traits::Controllable};
    use std::collections::HashSet;

    // This could be a test specific to the Control proc macro, but we'd like to
    // run it over all the entities we know about in case someone implements the
    // Controls trait manually.
    fn validate_entity_control(entity: &mut dyn Controllable) {
        let mut param_names: HashSet<String> = HashSet::default();

        for index in 0..entity.control_index_count() {
            let index = ControlIndex(index);
            let param_name = entity.control_name_for_index(index).unwrap();
            assert!(
                param_names.insert(param_name.clone()),
                "Duplicate param name {} at index {index}",
                &param_name
            );
            assert_eq!(
                entity.control_index_for_name(&param_name).unwrap(),
                index,
                "Couldn't recover expected index {index} from control_index_for_name({})",
                &param_name
            );
        }
        assert_eq!(
            param_names.len(),
            entity.control_index_count(),
            "control_index_count() agrees with number of params"
        );

        // The Controls trait doesn't support getting values, only setting them.
        // So we can't actually verify that our sets are doing anything. If this
        // becomes an issue, then we have two options: (1) extend the Controls
        // trait to allow getting, and then worry that any errors are tested by
        // the same generated code that has the error, or (2) come up with a
        // wacky proc macro that converts param_name into a getter invocation. I
        // don't think regular macros can do that because of hygiene rules.
        for index in 0..entity.control_index_count() {
            let index = ControlIndex(index);
            let param_name = entity.control_name_for_index(index).unwrap();
            entity.control_set_param_by_index(index, 0.0.into());
            entity.control_set_param_by_index(index, 1.0.into());
            entity.control_set_param_by_name(&param_name, 0.0.into());
            entity.control_set_param_by_name(&param_name, 1.0.into());
        }
    }

    #[test]
    fn control_works() {
        let mut entity = Bitcrusher::default();
        validate_entity_control(&mut entity);

        // TODO: move this somewhere that does testing for all entities.
    }
}
