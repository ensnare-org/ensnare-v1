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

pub use ensnare_entities_factory::BuiltInEntities;
#[cfg(feature = "test")]
pub use ensnare_entities_test::register_test_entities;

pub mod prelude {
    #[cfg(feature = "test")]
    pub use super::register_test_entities;
    pub use super::BuiltInEntities;
}
