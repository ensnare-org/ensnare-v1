// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Main struct for Ensnare DAW application.

use crate::{
    menu::{MenuBar, MenuBarAction},
    settings::{Settings, SettingsPanel},
};
use crossbeam_channel::{unbounded, Select, Sender};
use eframe::{
    egui::{
        CentralPanel, Context, Direction, Event, FontData, FontDefinitions, Layout, ScrollArea,
        SidePanel, TextStyle, TopBottomPanel,
    },
    emath::{Align, Align2},
    epaint::{Color32, FontFamily, FontId},
    App, CreationContext,
};
use egui_toast::{Toast, ToastOptions, Toasts};
use ensnare::{app_version, prelude::*};
use ensnare_egui_widgets::{oblique_strategies, ObliqueStrategiesManager};
use ensnare_orchestration::orchestration::orchestrator;
use std::sync::{Arc, Mutex};

enum EnsnareMessage {
    MidiPanelEvent(MidiPanelEvent),
    AudioPanelEvent(AudioPanelEvent),
    OrchestratorEvent(OrchestratorEvent),
}

pub(super) struct Ensnare {
    event_channel: ChannelPair<EnsnareMessage>,

    orchestrator: Arc<Mutex<OldOrchestrator>>,

    menu_bar: MenuBar,
    control_panel: ControlPanel,
    orchestrator_panel: OrchestratorPanel,
    settings_panel: SettingsPanel,
    palette_panel: PalettePanel,

    toasts: Toasts,

    oblique_strategies_mgr: ObliqueStrategiesManager,

    exit_requested: bool,

    keyboard_events_sender: Sender<Event>,

    pub is_settings_panel_open: bool,

    new_orchestrator: Orchestrator,
}
impl Ensnare {
    /// The user-visible name of the application.
    pub(super) const NAME: &'static str = "Ensnare";

    /// The default name of a new project.
    const DEFAULT_PROJECT_NAME: &'static str = "Untitled";

    /// internal-only key for regular font.
    const FONT_REGULAR: &'static str = "font-regular";
    /// internal-only key for bold font.
    const FONT_BOLD: &'static str = "font-bold";
    /// internal-only key for monospaced font.
    const FONT_MONO: &'static str = "font-mono";

    pub(super) fn new(cc: &CreationContext) -> Self {
        let mut factory = EntityFactory::default();
        register_factory_entities(&mut factory);
        factory.complete_registration();
        if EntityFactory::initialize(factory).is_err() {
            panic!("Couldn't set EntityFactory once_cell");
        }
        if DragDropManager::initialize(DragDropManager::default()).is_err() {
            panic!("Couldn't set DragDropManager once_cell");
        }

        Self::initialize_fonts(&cc.egui_ctx);
        Self::initialize_visuals(&cc.egui_ctx);
        Self::initialize_style(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let settings = Settings::load().unwrap_or_default();
        let orchestrator_panel = OrchestratorPanel::default();
        let orchestrator = Arc::clone(&orchestrator_panel.orchestrator);
        let orchestrator_for_settings_panel = Arc::clone(&orchestrator);
        let control_panel = ControlPanel::default();
        // orchestrator.lock().unwrap().e.sample_buffer_channel_sender =
        //     Some(control_panel.sample_channel.sender.clone());
        // let keyboard_events_sender = orchestrator
        //     .lock()
        //     .unwrap()
        //     .e
        //     .keyboard_controller
        //     .sender()
        //     .clone();
        let (keyboard_events_sender, _receiver) = unbounded();
        let mut r = Self {
            event_channel: Default::default(),
            orchestrator,
            menu_bar: Default::default(),
            control_panel,

            orchestrator_panel,
            settings_panel: SettingsPanel::new_with(settings, orchestrator_for_settings_panel),
            palette_panel: Default::default(),
            toasts: Toasts::new()
                .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(Direction::BottomUp),
            oblique_strategies_mgr: Default::default(),
            exit_requested: Default::default(),
            keyboard_events_sender,
            is_settings_panel_open: Default::default(),
            new_orchestrator: Default::default(),
        };
        r.spawn_app_channel_watcher(cc.egui_ctx.clone());
        r.spawn_channel_aggregator();
        r
    }

    fn initialize_fonts(ctx: &Context) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            Self::FONT_REGULAR.to_owned(),
            FontData::from_static(include_bytes!(
                "../../../res/fonts/jost/static/Jost-Regular.ttf"
            )),
        );
        fonts.font_data.insert(
            Self::FONT_BOLD.to_owned(),
            FontData::from_static(include_bytes!(
                "../../../res/fonts/jost/static/Jost-Bold.ttf"
            )),
        );
        fonts.font_data.insert(
            Self::FONT_MONO.to_owned(),
            FontData::from_static(include_bytes!(
                "../../../res/fonts/roboto-mono/RobotoMono-VariableFont_wght.ttf"
            )),
        );

        // Make these fonts the highest priority.
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, Self::FONT_REGULAR.to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, Self::FONT_MONO.to_owned());
        fonts
            .families
            .entry(FontFamily::Name(Self::FONT_BOLD.into()))
            .or_default()
            .insert(0, Self::FONT_BOLD.to_owned());

        ctx.set_fonts(fonts);
    }

    /// Sets the default visuals.
    fn initialize_visuals(ctx: &Context) {
        let mut visuals = ctx.style().visuals.clone();

        // It's better to set text color this way than to change
        // Visuals::override_text_color because override_text_color overrides
        // dynamic highlighting when hovering over interactive text.
        visuals.widgets.noninteractive.fg_stroke.color = Color32::LIGHT_GRAY;
        ctx.set_visuals(visuals);
    }

    fn initialize_style(ctx: &Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(16.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
            (
                TextStyle::Monospace,
                FontId::new(16.0, FontFamily::Monospace),
            ),
            (
                TextStyle::Button,
                FontId::new(16.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(14.0, FontFamily::Proportional),
            ),
        ]
        .into();

        ctx.set_style(style);
    }

    /// Watches certain channels and asks for a repaint, which triggers the
    /// actual channel receiver logic, when any of them has something
    /// receivable.
    ///
    /// https://docs.rs/crossbeam-channel/latest/crossbeam_channel/struct.Select.html#method.ready
    ///
    /// We call ready() rather than select() because select() requires us to
    /// complete the operation that is ready, while ready() just tells us that a
    /// recv() would not block.
    fn spawn_app_channel_watcher(&mut self, ctx: Context) {
        let receiver = self.event_channel.receiver.clone();
        let _ = std::thread::spawn(move || -> ! {
            let mut sel = Select::new();
            let _ = sel.recv(&receiver);
            loop {
                let _ = sel.ready();
                ctx.request_repaint();
            }
        });
    }

    /// Watches all the channel receivers we know about, and either handles them
    /// immediately off the UI thread or forwards them to the app's event
    /// channel.
    fn spawn_channel_aggregator(&mut self) {
        let r0 = self.settings_panel.midi_panel().receiver().clone();
        let r1 = self.settings_panel.audio_panel().receiver().clone();
        let r2 = self.orchestrator_panel.receiver().clone();

        let app_sender = self.event_channel.sender.clone();
        let orchestrator_sender = self.orchestrator_panel.sender().clone();

        let _ = std::thread::spawn(move || -> ! {
            let mut sel = Select::new();
            let _ = sel.recv(&r0);
            let _ = sel.recv(&r1);
            let _ = sel.recv(&r2);

            loop {
                let operation = sel.select();
                let index = operation.index();
                match index {
                    0 => {
                        if let Ok(event) = operation.recv(&r0) {
                            match event {
                                MidiPanelEvent::Midi(channel, message) => {
                                    let _ = orchestrator_sender
                                        .send(OrchestratorInput::Midi(channel, message));
                                    // We still send this through to the app so
                                    // that it can update the UI activity
                                    // indicators, but not as urgently as this
                                    // block.
                                }
                                _ => {}
                            }
                            let _ = app_sender.send(EnsnareMessage::MidiPanelEvent(event));
                        }
                    }
                    1 => {
                        if let Ok(event) = operation.recv(&r1) {
                            let _ = app_sender.send(EnsnareMessage::AudioPanelEvent(event));
                        }
                    }
                    2 => {
                        if let Ok(event) = operation.recv(&r2) {
                            let _ = app_sender.send(EnsnareMessage::OrchestratorEvent(event));
                        }
                    }
                    _ => {
                        panic!("missing case for a new receiver")
                    }
                }
            }
        });
    }

    fn handle_app_event_channel(&mut self) {
        // As long the channel has messages in it, we'll keep handling them. We
        // don't expect a giant number of messages; otherwise we'd worry about
        // blocking the UI.
        loop {
            if let Ok(m) = self.event_channel.receiver.try_recv() {
                match m {
                    EnsnareMessage::MidiPanelEvent(event) => {
                        match event {
                            MidiPanelEvent::Midi(..) => {
                                // This was already forwarded to Orchestrator. Here we update the UI.
                                self.control_panel.tickle_midi_in();
                            }
                            MidiPanelEvent::MidiOut => self.control_panel.tickle_midi_out(),
                            MidiPanelEvent::SelectInput(_) => {
                                // TODO: save selection in prefs
                            }
                            MidiPanelEvent::SelectOutput(_) => {
                                // TODO: save selection in prefs
                            }
                            MidiPanelEvent::PortsRefreshed => {
                                // TODO: remap any saved preferences to ports that we've found
                                self.settings_panel.handle_midi_port_refresh();
                            }
                        }
                    }
                    EnsnareMessage::AudioPanelEvent(event) => match event {
                        AudioPanelEvent::InterfaceChanged => {
                            self.update_orchestrator_audio_interface_config();
                        }
                    },
                    EnsnareMessage::OrchestratorEvent(event) => match event {
                        OrchestratorEvent::Tempo(_tempo) => {
                            // This is (usually) an acknowledgement that Orchestrator
                            // got our request to change, so we don't need to do
                            // anything.
                        }
                        OrchestratorEvent::Quit => {
                            eprintln!("OrchestratorEvent::Quit")
                        }
                        OrchestratorEvent::Loaded(path, title) => {
                            let title = title.unwrap_or(String::from(Self::DEFAULT_PROJECT_NAME));
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Success,
                                text: format!("Loaded {} from {}", title, path.display()).into(),
                                options: ToastOptions::default()
                                    .duration_in_seconds(2.0)
                                    .show_progress(false),
                            });
                        }
                        OrchestratorEvent::LoadError(path, error) => {
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Error,
                                text: format!("Error loading {}: {}", path.display(), error).into(),
                                options: ToastOptions::default().duration_in_seconds(5.0),
                            });
                        }
                        OrchestratorEvent::Saved(path) => {
                            // TODO: this should happen only if the save operation was
                            // explicit. Autosaves should be invisible.
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Success,
                                text: format!("Saved to {}", path.display()).into(),
                                options: ToastOptions::default()
                                    .duration_in_seconds(1.0)
                                    .show_progress(false),
                            });
                        }
                        OrchestratorEvent::SaveError(path, error) => {
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Error,
                                text: format!("Error saving {}: {}", path.display(), error).into(),
                                options: ToastOptions::default().duration_in_seconds(5.0),
                            });
                        }
                        OrchestratorEvent::New => {
                            // No special UI needed for this.
                        }
                    },
                }
            } else {
                break;
            }
        }
    }

    fn update_orchestrator_audio_interface_config(&mut self) {
        let sample_rate = self.settings_panel.audio_panel().sample_rate();
        if let Ok(mut o) = self.orchestrator.lock() {
            o.update_sample_rate(sample_rate);
        }
    }

    fn show_top(&mut self, ui: &mut eframe::egui::Ui) {
        self.menu_bar
            .set_is_any_track_selected(self.orchestrator_panel.is_any_track_selected());
        self.menu_bar.ui(ui);
        let menu_action = self.menu_bar.take_action();
        self.handle_menu_bar_action(menu_action);
        ui.separator();
        ui.horizontal_centered(|ui| {
            if let Ok(mut o) = self.orchestrator.lock() {
                ui.add(transport(&mut o.transport));
            }
            self.control_panel.ui(ui);
        });
        ui.add_space(2.0);
        if let Some(action) = self.control_panel.take_action() {
            self.handle_control_panel_action(action);
        }
    }

    fn handle_control_panel_action(&mut self, action: ControlPanelAction) {
        let input = match action {
            ControlPanelAction::Play => Some(OrchestratorInput::ProjectPlay),
            ControlPanelAction::Stop => Some(OrchestratorInput::ProjectStop),
            ControlPanelAction::New => Some(OrchestratorInput::ProjectNew),
            ControlPanelAction::Open(path) => Some(OrchestratorInput::ProjectOpen(path)),
            ControlPanelAction::Save(path) => Some(OrchestratorInput::ProjectSave(path)),
            ControlPanelAction::ToggleSettings => {
                self.is_settings_panel_open = !self.is_settings_panel_open;
                None
            }
        };
        if let Some(input) = input {
            self.orchestrator_panel.send_to_service(input);
        }
    }

    fn show_bottom(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            eframe::egui::warn_if_debug_build(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(app_version());
                if let Some(seed) = self.oblique_strategies_mgr.check_seed() {
                    ui.add(oblique_strategies(seed));
                }
            });
        });
    }

    fn show_left(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            self.palette_panel.ui(ui);
        });
    }

    fn show_right(&mut self, ui: &mut eframe::egui::Ui) {
        ui.add(orchestrator(&mut self.new_orchestrator));
        //        ScrollArea::horizontal().show(ui, |ui| ui.label("Under Construction"));
    }

    fn show_center(&mut self, ui: &mut eframe::egui::Ui) {
        self.orchestrator_panel.ui(ui);
        self.toasts.show(ui.ctx());
    }

    fn show_settings_panel(&mut self, ctx: &Context) {
        eframe::egui::Window::new("Settings")
            .open(&mut self.is_settings_panel_open)
            .show(ctx, |ui| self.settings_panel.ui(ui));
    }

    fn copy_keyboard_events(&mut self, ctx: &eframe::egui::Context) {
        ctx.input(|i| {
            for e in i.events.iter() {
                match e {
                    eframe::egui::Event::Key {
                        repeat, modifiers, ..
                    } => {
                        if !repeat && !modifiers.any() {
                            let _ = self.keyboard_events_sender.send(e.clone());
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn handle_menu_bar_action(&mut self, action: Option<MenuBarAction>) {
        if let Some(action) = action {
            match action {
                MenuBarAction::Quit => self.exit_requested = true,
                MenuBarAction::ProjectNew => todo!(),
                MenuBarAction::ProjectOpen => todo!(),
                MenuBarAction::ProjectSave => todo!(),
                MenuBarAction::TrackNewMidi => todo!(),
                MenuBarAction::TrackNewAudio => todo!(),
                MenuBarAction::TrackNewAux => todo!(),
                MenuBarAction::TrackDuplicate => todo!(),
                MenuBarAction::TrackDelete => todo!(),
                MenuBarAction::TrackRemoveSelectedPatterns => todo!(),
                MenuBarAction::TrackAddThing(_) => todo!(),
                MenuBarAction::ComingSoon => todo!(),
            }
        }
    }

    fn check_drag_and_drop(&mut self) {
        if let Some((source, target)) = DragDropManager::check_and_clear_drop_event() {
            let input = match source {
                DragSource::NewDevice(ref key) => match target {
                    DropTarget::Controllable(_, _) => todo!(),
                    DropTarget::Track(track_uid) => Some(OrchestratorInput::TrackAddEntity(
                        track_uid,
                        EntityKey::from(key),
                    )),
                    DropTarget::TrackPosition(_, _) => {
                        eprintln!("DropTarget::TrackPosition not implemented - ignoring");
                        None
                    }
                },
                DragSource::Pattern(pattern_uid) => match target {
                    DropTarget::Controllable(_, _) => todo!(),
                    DropTarget::Track(_) => todo!(),
                    DropTarget::TrackPosition(track_uid, position) => Some(
                        OrchestratorInput::TrackPatternAdd(track_uid, pattern_uid, position),
                    ),
                },
                DragSource::ControlSource(source_uid) => match target {
                    DropTarget::Controllable(target_uid, index) => Some(
                        OrchestratorInput::LinkControl(source_uid, target_uid, index),
                    ),
                    DropTarget::Track(_) => todo!(),
                    DropTarget::TrackPosition(_, _) => todo!(),
                },
            };
            if let Some(input) = input {
                let _ = self.orchestrator_panel.send_to_service(input);
            } else {
                eprintln!("WARNING: unhandled DnD pair: {source:?} {target:?}");
            }
        }
    }
}
impl App for Ensnare {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.handle_app_event_channel();
        self.copy_keyboard_events(ctx);

        let is_control_only_down = ctx.input(|i| i.modifiers.command_only());
        self.orchestrator_panel
            .set_control_only_down(is_control_only_down);

        let top = TopBottomPanel::top("top-panel")
            .resizable(false)
            .exact_height(64.0);
        let bottom = TopBottomPanel::bottom("bottom-panel")
            .resizable(false)
            .exact_height(24.0);
        let left = SidePanel::left("left-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0);
        let right = SidePanel::right("right-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0);
        let center = CentralPanel::default();

        top.show(ctx, |ui| {
            self.show_top(ui);
        });
        bottom.show(ctx, |ui| {
            self.show_bottom(ui);
        });
        left.show(ctx, |ui| {
            self.show_left(ui);
        });
        right.show(ctx, |ui| {
            self.show_right(ui);
        });
        center.show(ctx, |ui| {
            self.show_center(ui);
        });

        self.show_settings_panel(ctx);

        self.check_drag_and_drop();

        if self.exit_requested {
            frame.close();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.settings_panel.settings().has_been_saved() {
            let _ = self.settings_panel.settings_mut().save();
        }
        self.settings_panel.exit();
        self.orchestrator_panel.exit();
    }
}
