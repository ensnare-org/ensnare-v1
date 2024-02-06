// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Cores are basic musical devices without the overhead that the rest of the
//! system needs to use them. A core plus that overhead is an
//! [Entity][crate::traits::Entity]. Cores exist separately from entities so
//! that it's easier to focus on business logic when developing a new device.

pub mod controllers;
pub mod effects;
pub mod instruments;
