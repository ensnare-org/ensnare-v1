// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Widgets that are part of a DAW's typical chrome.

pub use control_bar::{ControlBar, ControlBarAction, ControlBarWidget};
pub use transport::TransportWidget;

mod control_bar;
mod transport;
