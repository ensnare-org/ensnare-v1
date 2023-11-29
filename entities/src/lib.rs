// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Ensnare structs that implement the Entity trait.

pub mod controllers {
    pub use ensnare_entities_factory::controllers::*;
    #[cfg(feature = "test")]
    pub use ensnare_entities_test::controllers::*;
}
pub mod effects {
    pub use ensnare_entities_factory::effects::*;
    #[cfg(feature = "test")]
    pub use ensnare_entities_test::effects::*;
}
pub mod instruments {
    pub use ensnare_entities_factory::instruments::*;
    #[cfg(feature = "test")]
    pub use ensnare_entities_test::instruments::*;
}

// TODO: would be helpful to move these into a special group to indicate that
// they generally aren't supposed to be instantiated by end users.
pub mod piano_roll {
    pub use ensnare_entities_factory::piano_roll::PianoRoll;
}

pub use ensnare_entities_factory::{register_factory_entities, FactoryEntity};
#[cfg(feature = "test")]
pub use ensnare_entities_test::register_test_entities;

pub mod prelude {
    pub use super::register_factory_entities;
    #[cfg(feature = "test")]
    pub use super::register_test_entities;
}
