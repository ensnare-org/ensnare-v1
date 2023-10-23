// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    control::ControlRouter,
    controllers::ControlAtlas,
    drag_drop::{DragDropEvent, DragDropManager, DragDropSource},
    entities::{controllers::sequencers::LivePatternEvent, prelude::*},
    humidifier::Humidifier,
    midi::prelude::*,
    midi_router::MidiRouter,
    piano_roll::{PatternUid, PianoRoll},
    prelude::*,
    traits::prelude::*,
    uid::IsUid,
    widgets::{
        prelude::*,
        timeline::{cursor, grid},
        track::{make_title_bar_galley, title_bar},
    },
};
use anyhow::anyhow;
use crossbeam_channel::Sender;
use eframe::{
    egui::{Frame, Margin},
    emath::RectTransform,
    epaint::{vec2, Color32, Rect, Stroke, Vec2},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::Range,
    option::Option,
    sync::{Arc, RwLock},
};
use strum_macros::Display;

/// Identifies a [Track].
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TrackUid(pub usize);
impl Default for TrackUid {
    fn default() -> Self {
        Self(1)
    }
}
impl IsUid for TrackUid {
    fn increment(&mut self) -> &Self {
        self.0 += 1;
        self
    }
}
impl Display for TrackUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

/// A [TrackAction] represents any UI operation that happens to a [Track] but
/// that the [Track] can't perform itself.  
#[derive(Clone, Debug, Display)]
pub enum TrackAction {
    /// Using the [EntityFactory], create a new entity of type [EntityKey] and
    /// add it to the track. [Track]s can't do this themselves because they
    /// don't have access to [EntityFactory] (or at least we've decided they
    /// shouldn't).
    NewDevice(EntityKey),

    /// Establish a control link between the source and target uids for the
    /// given parameter.
    LinkControl(Uid, Uid, ControlIndex),

    /// An entity has been selected, and we should show its detail view.
    EntitySelected(Uid),
}
impl IsAction for TrackAction {}

#[derive(Debug)]
pub struct TrackBuffer(pub [StereoSample; Self::LEN]);
impl TrackBuffer {
    pub const LEN: usize = 64;
}
impl Default for TrackBuffer {
    fn default() -> Self {
        Self([StereoSample::default(); Self::LEN])
    }
}

/// Newtype for track title string.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackTitle(pub String);
impl Default for TrackTitle {
    fn default() -> Self {
        Self("Untitled".to_string())
    }
}
impl From<&str> for TrackTitle {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Default)]
pub struct TrackEphemerals {
    buffer: TrackBuffer,
    pub(crate) piano_roll: Arc<RwLock<PianoRoll>>,
    pub(crate) action: Option<TrackAction>,
    view_range: std::ops::Range<MusicalTime>,
    pub(crate) title_font_galley: Option<Arc<eframe::epaint::Galley>>,

    // TODO: we need a story for how this gets restored on deserialization. We
    // have given up the type info by this point, so how do we recognize it?
    pattern_event_sender: Option<Sender<LivePatternEvent>>,
}

/// A collection of instruments, effects, and controllers that combine to
/// produce a single source of audio.
#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    pub title: TrackTitle,

    pub(crate) entity_store: EntityStore,

    /// The entities in the [EntityStore] that are capable of displaying in the timeline.
    pub(crate) timeline_entities: Vec<Uid>,

    /// If present, the one timeline entity that should be foreground during rendering.
    pub(crate) foreground_timeline_entity: Option<Uid>,

    midi_router: MidiRouter,

    /// [ControlRouter] manages the destinations of Control events. It does not
    /// generate events, but when events are generated, it knows where to route
    /// them. [ControlAtlas] manages the sources of Control events. It generates
    /// events but does not handle their routing.
    pub(crate) control_router: ControlRouter,
    pub(crate) control_atlas: ControlAtlas,

    pub(crate) controllers: Vec<Uid>,
    pub(crate) instruments: Vec<Uid>,
    pub(crate) effects: Vec<Uid>,

    humidifier: Humidifier,

    #[serde(skip)]
    pub(crate) e: TrackEphemerals,
}
impl Default for Track {
    fn default() -> Self {
        let mut r = Self {
            title: Default::default(),
            entity_store: Default::default(),
            timeline_entities: Default::default(),
            foreground_timeline_entity: Default::default(),
            midi_router: Default::default(),
            control_router: Default::default(),
            control_atlas: Default::default(),
            controllers: Default::default(),
            instruments: Default::default(),
            effects: Default::default(),
            humidifier: Default::default(),
            e: Default::default(),
        };

        r.timeline_entities.push(Self::CONTROL_ATLAS_UID);
        r.select_next_foreground_timeline_entity();
        r
    }
}
impl Track {
    /// A reserved Uid for the built-in ControlAtlas.
    const CONTROL_ATLAS_UID: Uid = Uid(1);

    #[allow(dead_code)]
    fn timeline_entity(&self, uid: &Uid) -> Option<&dyn DisplaysInTimeline> {
        match *uid {
            Self::CONTROL_ATLAS_UID => self.control_atlas.as_displays_in_timeline(),
            _ => {
                if let Some(e) = self.entity_store.get(&uid) {
                    e.as_displays_in_timeline()
                } else {
                    None
                }
            }
        }
    }

    fn timeline_entity_mut(&mut self, uid: &Uid) -> Option<&mut dyn DisplaysInTimeline> {
        match *uid {
            Self::CONTROL_ATLAS_UID => self.control_atlas.as_displays_in_timeline_mut(),
            _ => {
                if let Some(e) = self.entity_store.get_mut(&uid) {
                    e.as_displays_in_timeline_mut()
                } else {
                    None
                }
            }
        }
    }

    // TODO: for now the only way to add something new to a Track is to append it.
    #[allow(missing_docs)]
    pub fn append_entity(&mut self, entity: Box<dyn Entity>, uid: Uid) -> anyhow::Result<()> {
        // Some entities are hybrids, so they can appear in multiple lists.
        // That's why we don't have if-else here.
        if entity.as_controller().is_some() {
            self.controllers.push(uid);
        }
        if entity.as_effect().is_some() {
            self.effects.push(uid);
        }
        if entity.as_instrument().is_some() {
            self.instruments.push(uid);
        }
        if entity.as_handles_midi().is_some() {
            // TODO: for now, everyone's on channel 0
            self.midi_router.connect(uid, MidiChannel(0));
        }
        if entity.as_displays_in_timeline().is_some() {
            self.record_timeline_entity(uid);
        }

        self.entity_store.add(entity, uid)
    }

    fn record_timeline_entity(&mut self, uid: Uid) {
        self.timeline_entities.push(uid);
        if self.foreground_timeline_entity.is_none() {
            self.select_next_foreground_timeline_entity();
        }
    }

    fn remove_timeline_entity(&mut self, uid: &Uid) {
        if *uid == Self::CONTROL_ATLAS_UID {
            panic!("Tried to remove permanent entity ControlAtlas");
        }

        if self.foreground_timeline_entity == Some(*uid) {
            self.select_next_foreground_timeline_entity();
        }
        // It's important to remove this one after picking the next one, because
        // we need its position to determine the next one.
        self.timeline_entities.retain(|e| e != uid);

        if self.timeline_entities.is_empty() {
            self.foreground_timeline_entity = None;
        }
    }

    pub fn select_next_foreground_timeline_entity(&mut self) {
        if let Some(foreground_uid) = self.foreground_timeline_entity {
            if let Some(position) = self
                .timeline_entities
                .iter()
                .position(|uid| *uid == foreground_uid)
            {
                self.foreground_timeline_entity =
                    Some(self.timeline_entities[(position + 1) % self.timeline_entities.len()]);
            } else {
                self.foreground_timeline_entity = None;
            }
        } else {
            self.foreground_timeline_entity = self.timeline_entities.first().copied();
        }
    }

    #[allow(missing_docs)]
    pub fn remove_entity(&mut self, uid: &Uid) -> Option<Box<dyn Entity>> {
        if let Some(entity) = self.entity_store.remove(uid) {
            if entity.as_controller().is_some() {
                self.controllers.retain(|e| e != uid)
            }
            if entity.as_effect().is_some() {
                self.effects.retain(|e| e != uid);
            }
            if entity.as_instrument().is_some() {
                self.instruments.retain(|e| e != uid);
            }
            if entity.as_displays_in_timeline().is_some() {
                self.remove_timeline_entity(uid);
            }
            Some(entity)
        } else {
            None
        }
    }

    /// Returns the [Entity] having the given [Uid], if it exists.
    pub fn entity(&self, uid: &Uid) -> Option<&Box<dyn Entity>> {
        self.entity_store.get(uid)
    }

    /// Returns the mutable [Entity] having the given [Uid], if it exists.
    pub fn entity_mut(&mut self, uid: &Uid) -> Option<&mut Box<dyn Entity>> {
        self.entity_store.get_mut(uid)
    }

    #[allow(missing_docs)]
    pub fn remove_selected_patterns(&mut self) {
        todo!()
        //        self.sequencer.remove_selected_arranged_patterns();
    }

    #[allow(missing_docs)]
    pub fn route_midi_message(&mut self, channel: MidiChannel, message: MidiMessage) {
        if let Err(e) = self
            .midi_router
            .route(&mut self.entity_store, channel, message)
        {
            eprintln!("While routing: {e}");
        }
    }

    #[allow(missing_docs)]
    pub fn route_control_change(&mut self, uid: Uid, value: ControlValue) {
        if let Err(e) = self.control_router.route(
            &mut |target_uid, index, value| {
                if let Some(e) = self.entity_store.get_mut(target_uid) {
                    if let Some(e) = e.as_controllable_mut() {
                        e.control_set_param_by_index(index, value);
                    }
                }
            },
            uid,
            value,
        ) {
            eprintln!("While routing control change: {e}")
        }
    }

    pub(crate) fn set_title(&mut self, title: TrackTitle) {
        self.title = title;
    }

    /// Sets the wet/dry of an effect in the chain.
    pub fn set_humidity(&mut self, effect_uid: Uid, humidity: Normal) -> anyhow::Result<()> {
        if let Some(entity) = self.entity(&effect_uid) {
            if entity.as_effect().is_some() {
                self.humidifier.set_humidity_by_uid(effect_uid, humidity);
                Ok(())
            } else {
                Err(anyhow!("{effect_uid} is not an effect"))
            }
        } else {
            Err(anyhow!("{effect_uid} not found"))
        }
    }

    /// Moves the indicated effect to a new position within the effects chain.
    /// Zero is the first position.
    pub fn move_effect(&mut self, uid: Uid, new_index: usize) -> anyhow::Result<()> {
        if new_index >= self.effects.len() {
            Err(anyhow!(
                "Can't move {uid} to {new_index} when we have only {} items",
                self.effects.len()
            ))
        } else if self.effects.contains(&uid) {
            self.effects.retain(|e| e != &uid);
            self.effects.insert(new_index, uid);
            Ok(())
        } else {
            Err(anyhow!("Effect {uid} not found"))
        }
    }

    /// Returns the [ControlRouter].
    pub fn control_router_mut(&mut self) -> &mut ControlRouter {
        &mut self.control_router
    }

    /// Returns an immutable reference to the internal buffer.
    pub fn buffer(&self) -> &TrackBuffer {
        &self.e.buffer
    }

    /// Returns a writable version of the internal buffer.
    pub fn buffer_mut(&mut self) -> &mut TrackBuffer {
        &mut self.e.buffer
    }

    pub fn title_mut(&mut self) -> &mut TrackTitle {
        &mut self.title
    }

    pub fn view_range(&self) -> &std::ops::Range<MusicalTime> {
        &self.e.view_range
    }

    /// The [TitleBar] widget needs a Galley so that it can display the title
    /// sideways. But widgets live for only a frame, so it can't cache anything.
    /// Caller to the rescue! We generate the Galley and save it.
    ///
    /// TODO: when we allow title editing, we should set the galley to None so
    /// it can be rebuilt on the next frame.
    pub(crate) fn update_font_galley(&mut self, ui: &mut eframe::egui::Ui) {
        if self.e.title_font_galley.is_none() && !self.title.0.is_empty() {
            self.e.title_font_galley = Some(make_title_bar_galley(ui, &self.title));
        }
    }

    pub(crate) fn add_pattern(
        &mut self,
        pattern_uid: &PatternUid,
        position: MusicalTime,
    ) -> Result<(), anyhow::Error> {
        if let Some(sender) = &self.e.pattern_event_sender {
            let _ = sender.send(LivePatternEvent::Add(*pattern_uid, position));
            Ok(())
        } else {
            Err(anyhow!("No pattern event sender"))
        }
    }

    pub(crate) fn set_sequencer_channel(&mut self, sender: &Sender<LivePatternEvent>) {
        self.e.pattern_event_sender = Some(sender.clone());
    }

    /// Draws the given owned [Entity].
    pub(crate) fn entity_ui(
        &mut self,
        ui: &mut eframe::egui::Ui,
        uid: &Uid,
    ) -> eframe::egui::Response {
        if let Some(entity) = self.entity_store.get_mut(uid) {
            entity.ui(ui)
        } else {
            ui.label("missing!")
        }
    }
}
impl Acts for Track {
    type Action = TrackAction;

    fn set_action(&mut self, action: Self::Action) {
        debug_assert!(
            self.e.action.is_none(),
            "Uh-oh, tried to set to {action} but it was already set to {:?}",
            self.e.action
        );
        self.e.action = Some(action);
    }

    fn take_action(&mut self) -> Option<Self::Action> {
        self.e.action.take()
    }
}
impl GeneratesToInternalBuffer<StereoSample> for Track {
    fn generate_batch_values(&mut self, len: usize) -> usize {
        if len > self.e.buffer.0.len() {
            eprintln!(
                "requested {} samples but buffer is only len {}",
                len,
                self.e.buffer.0.len()
            );
            return 0;
        }

        // TODO: When I ditched TrackType, I lost the ability to be explicit
        // about the track type (wow). I'm still developing the heuristics for
        // aux vs. non-aux tracks. This is a good thing, though, because I'm
        // starting to think that the distinction was artificial (for example, I
        // just realized that aux tracks of course should be automatable, which
        // means that they have a timeline view).
        if self.instruments.is_empty() {
            // This track won't be generating any signal internally; rather,
            // it'll only process what that caller has already put there. That's fine.
        } else {
            // This track has instruments, each of which will be adding signal
            // to the buffer. Let's start with a clean one.
            self.e.buffer.0.fill(StereoSample::SILENCE);
        }

        for uid in self.instruments.iter() {
            if let Some(e) = self.entity_store.get_mut(uid) {
                if let Some(e) = e.as_instrument_mut() {
                    // Note that we're expecting everyone to ADD to the buffer,
                    // not to overwrite! TODO: convert all instruments to have
                    // internal buffers
                    e.generate_batch_values(&mut self.e.buffer.0);
                }
            }
        }

        for uid in self.effects.iter() {
            if let Some(e) = self.entity_store.get_mut(uid) {
                if let Some(e) = e.as_effect_mut() {
                    let humidity = self.humidifier.get_humidity_by_uid(uid);
                    if humidity == Normal::zero() {
                        continue;
                    }
                    self.humidifier
                        .transform_batch(humidity, e, &mut self.e.buffer.0);
                }
            }
        }

        // See #146 TODO - at this point we might want to gather any events
        // produced during the effects stage.

        self.e.buffer.0.len()
    }

    fn values(&self) -> &[StereoSample] {
        &self.e.buffer.0
    }
}
impl Ticks for Track {
    fn tick(&mut self, tick_count: usize) {
        self.entity_store.tick(tick_count);
    }
}
impl Configurable for Track {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.entity_store.update_sample_rate(sample_rate);
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.entity_store.update_tempo(tempo);
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.entity_store.update_time_signature(time_signature);
    }
}

// TODO: I think this is wrong and misguided. If MIDI messages are handled by
// Track, then each Track needs to record who's receiving on which channel, and
// messages can't be sent from a device on one track to one on a different
// track. While that could make parallelism easier, it doesn't seem intuitively
// correct, because in a real studio you'd be able to hook up MIDI cables
// independently of audio cables.
#[cfg(never)]
impl HandlesMidi for Track {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        messages_fn: &mut dyn FnMut(Uid, MidiChannel, MidiMessage),
    ) {
        for e in self.controllers.iter_mut() {
            e.handle_midi_message(channel, &message, messages_fn);
        }
        for e in self.instruments.iter_mut() {
            e.handle_midi_message(channel, &message, messages_fn);
        }
    }
}
impl Controls for Track {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.entity_store.update_time(range);
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        // Create a place to hold MIDI messages that we need to route.
        let mut midi_events = Vec::default();

        // Peek at incoming events before handing them to control_events_fn.
        let mut handler = |uid, event| {
            match event {
                // We need to route MIDI messages to all eligible Entities in
                // this Track, so we save them up.
                EntityEvent::Midi(channel, message) => {
                    midi_events.push((channel, message));
                }
                EntityEvent::Control(_) => {}
            }
            control_events_fn(uid, event);
        };

        // Let everyone work and possibly generate messages.
        self.entity_store.work(&mut handler);

        // We've accumulated all the MIDI messages. Route them to our own
        // MidiRouter. They've already been forwarded to the caller via
        // control_events_fn.
        midi_events.into_iter().for_each(|(channel, message)| {
            let _ = self
                .midi_router
                .route(&mut self.entity_store, channel, message);
        });
    }

    fn is_finished(&self) -> bool {
        self.entity_store.is_finished()
    }

    fn play(&mut self) {
        self.entity_store.play();
    }

    fn stop(&mut self) {
        self.entity_store.stop();
    }

    fn skip_to_start(&mut self) {
        self.entity_store.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.entity_store.is_performing()
    }
}
impl Serializable for Track {
    fn after_deser(&mut self) {
        // TODO: I think here is where we'd tell the sequencer about the piano
        // roll again (which will be hard).
        self.entity_store.after_deser();
    }
}
impl DisplaysInTimeline for Track {
    fn set_view_range(&mut self, view_range: &Range<MusicalTime>) {
        self.entity_store.iter_mut().for_each(|e| {
            if let Some(e) = e.as_displays_in_timeline_mut() {
                e.set_view_range(view_range);
            }
        });
        self.e.view_range = view_range.clone();
    }
}
impl Displays for Track {}

/// Wraps a [TrackWidget] as a [Widget](eframe::egui::Widget).
pub fn track_widget<'a>(
    track_uid: TrackUid,
    track: &'a mut Track,
    is_selected: bool,
    ui_state: TrackUiState,
    cursor: Option<MusicalTime>,
    action: &'a mut Option<TrackWidgetAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        TrackWidget::new(track_uid, track, cursor, action)
            .is_selected(is_selected)
            .ui_state(ui_state)
            .ui(ui)
    }
}

// pub(crate) fn track_view_height(&self) -> f32 {
//     if matches!(track_type, TrackType::Aux) {
//         device_view_height(ui_state)
//     } else {
//         timeline_view_height(ui_state) + device_view_height(ui_state)
//     }
// }

// pub(crate) const fn timeline_view_height(_ui_state: TrackUiState) -> f32 {
//     64.0
// }

pub(crate) const fn device_view_height(ui_state: TrackUiState) -> f32 {
    match ui_state {
        TrackUiState::Collapsed => 32.0,
        TrackUiState::Expanded => 96.0,
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub enum TrackUiState {
    #[default]
    Collapsed,
    Expanded,
}

#[derive(Debug, Display)]
pub enum TrackWidgetAction {
    EntitySelected(Uid),
}

/// An egui widget that draws a [Track].
#[derive(Debug)]
struct TrackWidget<'a> {
    track_uid: TrackUid,
    track: &'a mut Track,
    is_selected: bool,
    ui_state: TrackUiState,
    cursor: Option<MusicalTime>,
    action: &'a mut Option<TrackWidgetAction>,
}
impl<'a> Displays for TrackWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // inner_margin() should be half of the Frame stroke width to leave room
        // for it. Thanks vikrinox on the egui Discord.
        Frame::default()
            .inner_margin(Margin::same(0.5))
            .stroke(Stroke {
                width: 1.0,
                color: {
                    if self.is_selected {
                        Color32::YELLOW
                    } else {
                        Color32::DARK_GRAY
                    }
                },
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    //                    ui.set_min_height(track_view_height(self.track.ty(), self.ui_state));

                    // The `Response` is based on the title bar, so
                    // clicking/dragging on the title bar affects the `Track` as a
                    // whole.
                    let font_galley = self
                        .track
                        .e
                        .title_font_galley
                        .as_ref()
                        .map(|fg| Arc::clone(&fg));
                    let response = ui.add(title_bar(font_galley));

                    // Take up all the space we're given, even if we can't fill
                    // it with widget content.
                    ui.set_min_size(ui.available_size());

                    // The frames shouldn't have space between them.
                    ui.style_mut().spacing.item_spacing = Vec2::ZERO;

                    // Build the track content with the device view beneath it.
                    ui.vertical(|ui| {
                        // Do we need to draw the timeline view?
                        if !self.track.timeline_entities.is_empty() {
                            // This is declared here because we need it to keep
                            // existing outside of the drop_target() block, so
                            // that we can use it to calculate the position of
                            // mouse clicks within the timeline rect.
                            let mut from_screen = RectTransform::identity(Rect::NOTHING);
                            let can_accept = self.check_drag_source_for_timeline();
                            let response = DragDropManager::drop_target(ui, can_accept, |ui| {
                                // Reserve space for the device view.
                                // ui.set_max_height(track_view_height(
                                //     self.track.ty(),
                                //     self.ui_state,
                                // ));

                                // Determine the rectangle that all the composited layers will use.
                                let desired_size = vec2(ui.available_width(), 64.0);
                                let (_id, rect) = ui.allocate_space(desired_size);

                                let temp_range = MusicalTime::START..MusicalTime::DURATION_WHOLE;
                                let view_range = self.track.view_range().clone();

                                from_screen = RectTransform::from_to(
                                    rect,
                                    Rect::from_x_y_ranges(
                                        view_range.start.total_units() as f32
                                            ..=view_range.end.total_units() as f32,
                                        rect.top()..=rect.bottom(),
                                    ),
                                );

                                // The Grid is always disabled and drawn first.
                                let _ = ui
                                    .allocate_ui_at_rect(rect, |ui| {
                                        ui.add(grid(
                                            temp_range.clone(),
                                            self.track.view_range().clone(),
                                        ))
                                    })
                                    .inner;

                                // Draw the disabled timeline views.
                                let enabled_uid = self.track.foreground_timeline_entity.clone();
                                let entities: Vec<Uid> = self
                                    .track
                                    .timeline_entities
                                    .iter()
                                    .filter(|uid| enabled_uid != Some(**uid))
                                    .cloned()
                                    .collect();
                                entities.iter().for_each(|uid| {
                                    if let Some(e) = self.track.timeline_entity_mut(uid) {
                                        ui.add_enabled_ui(false, |ui| {
                                            ui.allocate_ui_at_rect(rect, |ui| e.ui(ui)).inner
                                        })
                                        .inner;
                                    }
                                });

                                // Draw the one enabled timeline view.
                                if let Some(uid) = enabled_uid {
                                    if let Some(e) = self.track.timeline_entity_mut(&uid) {
                                        ui.add_enabled_ui(true, |ui| {
                                            ui.allocate_ui_at_rect(rect, |ui| e.ui(ui)).inner
                                        })
                                        .inner;
                                    }
                                }

                                // Finally, if it's present, draw the cursor.
                                if let Some(position) = self.cursor {
                                    if view_range.contains(&position) {
                                        let _ = ui
                                            .allocate_ui_at_rect(rect, |ui| {
                                                ui.add(cursor(position, view_range.clone()))
                                            })
                                            .inner;
                                    }
                                }
                            })
                            .response;
                            if DragDropManager::is_dropped(ui, &response) {
                                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                    let time_pos = from_screen * pointer_pos;
                                    let time = MusicalTime::new_with_units(time_pos.x as usize);
                                    if let Some(source) = DragDropManager::source() {
                                        let event = match source {
                                            DragDropSource::NewDevice(key) => Some(
                                                DragDropEvent::TrackAddDevice(self.track_uid, key),
                                            ),
                                            DragDropSource::Pattern(pattern_uid) => {
                                                Some(DragDropEvent::TrackAddPattern(
                                                    self.track_uid,
                                                    pattern_uid,
                                                    time,
                                                ))
                                            }
                                            _ => None,
                                        };
                                        if let Some(event) = event {
                                            DragDropManager::enqueue_event(event);
                                        }
                                    }
                                } else {
                                    eprintln!("Dropped on timeline at unknown position");
                                }
                            }
                        }

                        // Draw the signal chain view for every kind of track.
                        ui.scope(|ui| {
                            ui.set_max_height(device_view_height(self.ui_state));
                            let mut action = None;
                            ui.add(signal_chain(&mut self.track, &mut action));
                            if let Some(action) = action {
                                match action {
                                    SignalChainWidgetAction::EntitySelected(uid) => {
                                        *self.action = Some(TrackWidgetAction::EntitySelected(uid));
                                    }
                                }
                            }
                        });

                        response
                    })
                    .inner
                })
                .inner
            })
            .inner
    }
}
impl<'a> TrackWidget<'a> {
    fn new(
        track_uid: TrackUid,
        track: &'a mut Track,
        cursor: Option<MusicalTime>,
        action: &'a mut Option<TrackWidgetAction>,
    ) -> Self {
        Self {
            track_uid,
            track,
            is_selected: false,
            ui_state: TrackUiState::Collapsed,
            cursor,
            action,
        }
    }

    fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    fn ui_state(mut self, ui_state: TrackUiState) -> Self {
        self.ui_state = ui_state;
        self
    }

    // Looks at what's being dragged, if anything, and updates any state needed
    // to handle it. Returns whether we are interested in this drag source.
    fn check_drag_source_for_timeline(&mut self) -> bool {
        if let Some(source) = DragDropManager::source() {
            if matches!(source, DragDropSource::Pattern(..)) {
                return true;
            }
        }
        false
    }
}

/// Wraps a [SignalChainWidget] as a [Widget](eframe::egui::Widget). Mutates many things.
pub fn signal_chain<'a>(
    track: &'a mut Track,
    action: &'a mut Option<SignalChainWidgetAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| SignalChainWidget::new(track, action).ui(ui)
}

#[derive(Debug, Display)]
pub enum SignalChainWidgetAction {
    EntitySelected(Uid),
}
impl IsAction for SignalChainWidgetAction {}

#[derive(Debug)]
struct SignalChainWidget<'a> {
    track: &'a mut Track,
    ui_size: UiSize,
    action: &'a mut Option<SignalChainWidgetAction>,
}
impl<'a> SignalChainWidget<'a> {
    pub fn new(track: &'a mut Track, action: &'a mut Option<SignalChainWidgetAction>) -> Self {
        Self {
            track,
            ui_size: Default::default(),
            action,
        }
    }

    fn can_accept(&self) -> bool {
        if let Some(source) = DragDropManager::source() {
            matches!(source, DragDropSource::NewDevice(_))
        } else {
            false
        }
    }

    fn check_drop(&mut self) {
        if let Some(source) = DragDropManager::source() {
            if let DragDropSource::NewDevice(key) = source {
                self.track.e.action = Some(TrackAction::NewDevice(key));
            }
        }
    }
}
impl<'a> Displays for SignalChainWidget<'a> {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        self.ui_size = UiSize::from_height(ui.available_height());
        let desired_size = ui.available_size();

        ui.allocate_ui(desired_size, |ui| {
            let stroke = ui.ctx().style().visuals.noninteractive().bg_stroke;
            eframe::egui::Frame::default()
                .stroke(stroke)
                .inner_margin(eframe::egui::Margin::same(stroke.width / 2.0))
                .show(ui, |ui| {
                    ui.set_min_size(desired_size);
                    ui.horizontal_top(|ui| {
                        self.track
                            .controllers
                            .iter()
                            .chain(
                                self.track
                                    .instruments
                                    .iter()
                                    .chain(self.track.effects.iter()),
                            )
                            .filter(|e| !self.track.timeline_entities.contains(e))
                            .for_each(|uid| {
                                if let Some(entity) = self.track.entity_store.get_mut(uid) {
                                    if entity.as_controller().is_some() {
                                        DragDropManager::drag_source(
                                            ui,
                                            eframe::egui::Id::new(entity.name()),
                                            DragDropSource::ControlSource(*uid),
                                            |ui| {
                                                ui.label(entity.name());
                                            },
                                        )
                                    } else {
                                        ui.label(entity.name());
                                    }
                                    if ui.button("show").clicked() {
                                        *self.action =
                                            Some(SignalChainWidgetAction::EntitySelected(*uid));
                                    }
                                }
                            });
                        let response = DragDropManager::drop_target(ui, self.can_accept(), |ui| {
                            ui.label("[+]")
                        })
                        .response;
                        if DragDropManager::is_dropped(ui, &response) {
                            self.check_drop();
                        }
                    })
                    .inner
                });
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::factory::test_entities::{
        TestEffect, TestInstrument, TestInstrumentCountsMidiMessages,
    };
    use ensnare_proc_macros::{Control, IsControllerWithTimelineDisplay, Uid};

    #[test]
    fn basic_track_operations() {
        let mut t = Track::default();
        assert!(t.controllers.is_empty());
        assert!(t.effects.is_empty());
        assert!(t.instruments.is_empty());

        // Create an instrument and add it to a track.
        let id1 = Uid(1);
        assert!(t
            .append_entity(Box::new(TestInstrument::default()), id1)
            .is_ok());

        // Add a second instrument to the track.
        let id2 = Uid(2);
        assert!(t
            .append_entity(Box::new(TestInstrument::default()), id2)
            .is_ok());

        assert_ne!(id1, id2, "Don't forget to assign unique IDs!");

        assert_eq!(
            t.instruments[0], id1,
            "first appended entity should be at index 0"
        );
        assert_eq!(
            t.instruments[1], id2,
            "second appended entity should be at index 1"
        );
        assert_eq!(
            t.instruments.len(),
            2,
            "there should be exactly as many entities as added"
        );

        let _ = t.remove_entity(&id1).unwrap();
        assert_eq!(t.instruments.len(), 1, "removed exactly one instrument");
        assert_eq!(
            t.instruments[0], id2,
            "the remaining instrument should be the one we left"
        );
        assert!(
            t.entity_store.get(&id1).is_none(),
            "it should be gone from the store"
        );

        let effect_id1 = Uid(3);
        assert!(t
            .append_entity(Box::new(TestEffect::default()), effect_id1)
            .is_ok());
        let effect_id2 = Uid(4);
        assert!(t
            .append_entity(Box::new(TestEffect::default()), effect_id2,)
            .is_ok());

        assert_eq!(t.effects[0], effect_id1);
        assert_eq!(t.effects[1], effect_id2);
        assert!(t.move_effect(effect_id1, 1).is_ok());
        assert_eq!(
            t.effects[0], effect_id2,
            "After moving effects, id2 should be first"
        );
        assert_eq!(t.effects[1], effect_id1);
    }

    // We expect that a MIDI message will be routed to the eligible Entities in
    // the same Track, and forwarded to the work() caller, presumably to decide
    // whether to send it to other destination(s) such as external MIDI
    // interfaces.
    #[test]
    fn midi_messages_sent_to_caller_and_sending_track_instruments() {
        let mut t = Track::default();

        assert!(t
            .append_entity(
                Box::new(ToyControllerAlwaysSendsMidiMessage::default()),
                Uid(2001),
            )
            .is_ok());

        let receiver = TestInstrumentCountsMidiMessages::default();
        let counter = Arc::clone(receiver.received_midi_message_count_mutex());
        assert!(t.append_entity(Box::new(receiver), Uid(2002)).is_ok());

        let mut external_midi_messages = 0;
        t.play();
        t.work(&mut |_uid, _event| {
            external_midi_messages += 1;
        });

        if let Ok(c) = counter.lock() {
            assert_eq!(
                *c, 1,
                "The receiving instrument in the track should have received the message"
            );
        };

        assert_eq!(
            external_midi_messages, 1,
            "After one work(), one MIDI message should have emerged for external processing"
        );
    }

    #[derive(
        Default, Debug, Control, IsControllerWithTimelineDisplay, Uid, Serialize, Deserialize,
    )]
    struct TimelineDisplayer {
        uid: Uid,
    }
    impl Serializable for TimelineDisplayer {}
    impl Controls for TimelineDisplayer {}
    impl Configurable for TimelineDisplayer {}
    impl HandlesMidi for TimelineDisplayer {}
    impl Displays for TimelineDisplayer {}
    impl DisplaysInTimeline for TimelineDisplayer {
        fn set_view_range(&mut self, _view_range: &std::ops::Range<MusicalTime>) {}
    }

    #[test]
    fn track_picks_next_foreground_entity() {
        let e1_uid = Uid(1000);
        let e2_uid = Uid(2000);
        let e3_uid = Uid(3000);
        let e1 = Box::new(TimelineDisplayer::default());
        let e2 = Box::new(TimelineDisplayer::default());
        let e3 = Box::new(TimelineDisplayer::default());

        // TODO: reenable these after we decide what behavior we're actually looking for.

        let mut t = Track::default();
        // assert!(
        //     t.foreground_timeline_entity.is_none(),
        //     "should be none foreground at creation"
        // );
        let _ = t.append_entity(e1, e1_uid);
        // assert!(
        //     t.foreground_timeline_entity.is_some(),
        //     "adding one should make it foreground"
        // );
        let e1 = t.remove_entity(&e1_uid).unwrap();
        // assert!(
        //     t.foreground_timeline_entity.is_none(),
        //     "removing the last one should make none foreground"
        // );
        let _ = t.append_entity(e1, e1_uid);
        // assert_eq!(
        //     t.foreground_timeline_entity,
        //     Some(e1_uid),
        //     "adding first one should make it foreground"
        // );
        let _ = t.append_entity(e2, e2_uid);
        // assert_eq!(
        //     t.foreground_timeline_entity,
        //     Some(e1_uid),
        //     "adding a second shouldn't change which is foreground"
        // );
        let _ = t.append_entity(e3, e3_uid);
        // assert_eq!(
        //     t.foreground_timeline_entity,
        //     Some(e1_uid),
        //     "adding a third shouldn't change which is foreground"
        // );
        let e3 = t.remove_entity(&e3_uid).unwrap();
        // assert_eq!(
        //     t.foreground_timeline_entity,
        //     Some(e1_uid),
        //     "removing the third shouldn't change which is foreground"
        // );
        let _e1 = t.remove_entity(&e1_uid).unwrap();
        // assert_eq!(
        //     t.foreground_timeline_entity,
        //     Some(e2_uid),
        //     "removing the first/foreground should pick the new first one"
        // );

        let _ = t.append_entity(e3, e3_uid);
        t.select_next_foreground_timeline_entity();
        assert_eq!(
            t.foreground_timeline_entity,
            Some(e2_uid),
            "selecting next after built-in should pick the second (first was removed)"
        );

        t.select_next_foreground_timeline_entity();
        assert_eq!(
            t.foreground_timeline_entity,
            Some(e3_uid),
            "selecting next after second should pick the third"
        );

        t.select_next_foreground_timeline_entity();
        assert_eq!(
            t.foreground_timeline_entity,
            Some(Track::CONTROL_ATLAS_UID),
            "selecting next after third should wrap back to built-in"
        );
    }
}
