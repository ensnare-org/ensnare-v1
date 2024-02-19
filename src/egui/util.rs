// Copyright (c) 2024 Mike Tsao. All rights reserved.

use eframe::egui::{Frame, Response};
use std::sync::Arc;

// See https://github.com/emilk/egui/issues/4059 for why this
// code is a bit cumbersome
pub fn dnd_drop_zone_with_inner_response<Payload>(
    ui: &mut eframe::egui::Ui,
    add_contents: impl FnOnce(&mut eframe::egui::Ui) -> Response,
) -> (Option<Response>, Response, Option<Arc<Payload>>)
where
    Payload: core::any::Any + Send + Sync,
{
    let mut inner_response = None;
    let (mut response, payload) = ui.dnd_drop_zone::<Payload>(Frame::default(), |ui| {
        inner_response = Some(add_contents(ui));
    });
    if let Some(inner_response) = inner_response.as_ref() {
        response |= inner_response.clone();
    }
    (inner_response, response, payload)
}
