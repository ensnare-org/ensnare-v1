// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::{
    egui::{CursorIcon, Id as EguiId, InnerResponse, LayerId, Order, Sense, Ui},
    epaint::{self, Color32, Rect, Shape, Stroke, Vec2},
};
use ensnare_core::{piano_roll::PatternUid, prelude::*};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use strum_macros::Display;

/// The one and only DragDropManager. Access it with `DragDropManager::global()`.
static DD_MANAGER: OnceCell<Mutex<DragDropManager>> = OnceCell::new();

#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum DragSource {
    NewDevice(String),
    Pattern(PatternUid),
    ControlSource(Uid),
}

#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum DropTarget {
    Controllable(Uid, ControlIndex),
    Track(TrackUid),
    TrackPosition(TrackUid, MusicalTime),
}

// TODO: a way to express rules about what can and can't be dropped
#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct DragDropManager {
    source: Option<DragSource>,
    target: Option<DropTarget>,
}
#[allow(missing_docs)]
impl DragDropManager {
    /// Provides the one and only [DragDropManager].
    pub fn global() -> &'static Mutex<Self> {
        DD_MANAGER
            .get()
            .expect("DragDropManager has not been initialized")
    }

    pub fn reset() {
        if let Ok(mut dd) = Self::global().lock() {
            dd.source = None;
            dd.target = None;
        }
    }

    /// The main egui update() method should call this method once at the end of
    /// rendering. If it returns something, then the user dragged the source
    /// onto the target, and the app should handle accordingly. No matter what,
    /// this method clears source/target so that it's ready for the next
    /// update().  
    pub fn check_and_clear_drop_event() -> Option<(DragSource, DropTarget)> {
        if let Ok(mut dd) = Self::global().lock() {
            if let Some(source) = dd.source.take() {
                if let Some(target) = dd.target.take() {
                    return Some((source, target));
                }
            }
        }
        None
    }

    // These two functions are based on egui_demo_lib/src/demo/drag_and_drop.rs
    pub fn drag_source(
        ui: &mut eframe::egui::Ui,
        id: EguiId,
        drag_source: DragSource,
        body: impl FnOnce(&mut Ui) -> eframe::egui::Response,
    ) -> eframe::egui::Response {
        // This allows the app to avoid having to call reset() on every event
        // loop iteration, and fixes the bug that a drop target could see only
        // the drag sources that were instantiated earlier in the main event
        // loop.
        if !Self::is_anything_being_dragged(ui) {
            Self::reset();
        }

        if ui.memory(|mem| mem.is_being_dragged(id)) {
            // It is. So let's mark that it's the one.
            Self::global().lock().unwrap().source = Some(drag_source);

            // Indicate in UI that we're dragging.
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);

            // Plan to draw above everything else except debug.
            let layer_id = LayerId::new(Order::Tooltip, id);

            // Draw the body and grab the response.
            let response = ui.with_layer_id(layer_id, body);
            let (response, inner) = (response.response, response.inner);

            // Shift the entire tooltip layer to keep up with mouse movement.
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let delta = pointer_pos - response.rect.center();
                ui.ctx().translate_layer(layer_id, delta);
            }

            inner
        } else {
            // Let the body draw itself, but scope to undo any style changes.
            let response = ui.scope(body);
            let (response, inner) = (response.response, response.inner);

            // If the mouse is still over the item, change cursor to indicate
            // that user could drag.
            let response = ui.interact(response.rect, id, Sense::drag());
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }

            inner
        }
    }

    pub fn drop_target<R>(
        ui: &mut eframe::egui::Ui,
        can_accept_what_is_being_dragged: bool,
        body: impl FnOnce(&mut Ui) -> (R, DropTarget),
    ) -> InnerResponse<R> {
        // Is there any drag source at all?
        let is_anything_dragged = Self::is_anything_being_dragged(ui);

        // Carve out a UI-sized area but leave a bit of margin to draw DnD
        // highlight.
        let margin = Vec2::splat(2.0);
        let outer_rect_bounds = ui.available_rect_before_wrap();
        let inner_rect = outer_rect_bounds.shrink2(margin);

        // We want this to draw behind the body, but we're not sure what it is
        // yet.
        let where_to_put_background = ui.painter().add(Shape::Noop);

        // Draw the potential target.
        let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
        let (ret, drop_target) = body(&mut content_ui);

        // I think but am not sure that this calculates the actual boundaries of
        // what the body drew.
        let outer_rect =
            Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);

        // Figure out what's going on in that rect.
        let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

        // Adjust styling depending on whether this is still a potential target.
        let style = if is_anything_dragged && can_accept_what_is_being_dragged && response.hovered()
        {
            ui.visuals().widgets.active
        } else {
            ui.visuals().widgets.inactive
        };
        let mut fill = style.bg_fill;
        let mut stroke = style.bg_stroke;
        if is_anything_dragged {
            if !can_accept_what_is_being_dragged {
                fill = ui.visuals().gray_out(fill);
                stroke.color = ui.visuals().gray_out(stroke.color);
            }
        } else {
            fill = Color32::TRANSPARENT;
            stroke = Stroke::NONE;
        };

        // Update the background border based on target state.
        ui.painter().set(
            where_to_put_background,
            epaint::RectShape::new(rect, style.rounding, fill, stroke),
        );

        if Self::is_dropped(ui, &response) {
            Self::global().lock().unwrap().target = Some(drop_target);
        }

        InnerResponse::new(ret, response)
    }

    fn is_anything_being_dragged(ui: &mut eframe::egui::Ui) -> bool {
        ui.memory(|mem| mem.is_anything_being_dragged())
    }

    fn is_source_set() -> bool {
        Self::global().lock().unwrap().source.is_some()
    }

    pub fn is_dropped(ui: &mut eframe::egui::Ui, response: &eframe::egui::Response) -> bool {
        Self::is_anything_being_dragged(ui)
            && response.hovered()
            && ui.input(|i| i.pointer.any_released())
            && Self::is_source_set()
    }

    pub fn source() -> Option<DragSource> {
        Self::global().lock().unwrap().source.clone()
    }

    pub fn initialize(drag_drop_manager: Self) -> Result<(), Mutex<DragDropManager>> {
        DD_MANAGER.set(Mutex::new(drag_drop_manager))
    }
}
