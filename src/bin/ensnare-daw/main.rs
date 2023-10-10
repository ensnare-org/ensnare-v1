// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A digital audio workstation.

use anyhow::anyhow;
use crossbeam_channel::Select;
use eframe::{
    egui::{
        CentralPanel, Context, FontData, FontDefinitions, Layout, ScrollArea, SidePanel, TextStyle,
        TopBottomPanel, Ui,
    },
    emath::Align,
    epaint::{Color32, FontFamily, FontId},
    App, CreationContext,
};
use egui_toast::{Toast, ToastOptions, Toasts};
use ensnare::{
    arrangement::transport, panels::prelude::*, prelude::*, ui::DragDropEvent, version::app_version,
};
use env_logger;
use settings::{Settings, SettingsPanel};
use std::sync::{Arc, Mutex};

mod settings;

enum EnsnareMessage {
    MidiPanelEvent(MidiPanelEvent),
    AudioPanelEvent(AudioPanelEvent),
    OrchestratorEvent(OrchestratorEvent),
}

struct Ensnare {
    event_channel: ChannelPair<EnsnareMessage>,

    orchestrator: Arc<Mutex<Orchestrator>>,

    control_panel: ControlPanel,
    orchestrator_panel: OrchestratorPanel,
    settings_panel: SettingsPanel,
    palette_panel: PalettePanel,

    toasts: Toasts,

    exit_requested: bool,
}
impl Ensnare {
    /// The user-visible name of the application.
    const NAME: &'static str = "Ensnare";

    /// The default name of a new project.
    const DEFAULT_PROJECT_NAME: &'static str = "Untitled";

    /// internal-only key for regular font.
    const FONT_REGULAR: &'static str = "font-regular";
    /// internal-only key for bold font.
    const FONT_BOLD: &'static str = "font-bold";
    /// internal-only key for monospaced font.
    const FONT_MONO: &'static str = "font-mono";

    fn new(cc: &CreationContext) -> Self {
        Self::initialize_fonts(&cc.egui_ctx);
        Self::initialize_visuals(&cc.egui_ctx);
        Self::initialize_style(&cc.egui_ctx);

        let settings = Settings::load().unwrap_or_default();
        let orchestrator_panel = OrchestratorPanel::default();
        let orchestrator = Arc::clone(orchestrator_panel.orchestrator());
        let orchestrator_for_settings_panel = Arc::clone(&orchestrator);

        let mut r = Self {
            event_channel: Default::default(),
            orchestrator,
            control_panel: Default::default(),
            orchestrator_panel,
            settings_panel: SettingsPanel::new_with(settings, orchestrator_for_settings_panel),
            palette_panel: Default::default(),
            toasts: Toasts::new()
                .anchor(eframe::emath::Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(eframe::egui::Direction::BottomUp),
            exit_requested: Default::default(),
        };
        r.spawn_app_channel_watcher(cc.egui_ctx.clone());
        r.spawn_channel_aggregator(cc.egui_ctx.clone());
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
                "../../../res/fonts/cousine/Cousine-Regular.ttf"
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
                FontId::new(14.0, FontFamily::Monospace),
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
    fn spawn_channel_aggregator(&mut self, ctx: Context) {
        let r1 = self.settings_panel.midi_panel().receiver().clone();
        let r2 = self.settings_panel.audio_panel().receiver().clone();
        let r3 = self.orchestrator_panel.receiver().clone();
        let r4 = DragDropManager::global().lock().unwrap().receiver().clone();

        let app_sender = self.event_channel.sender.clone();
        let orchestrator_sender = self.orchestrator_panel.sender().clone();

        let _ = std::thread::spawn(move || -> ! {
            let mut sel = Select::new();
            let _ = sel.recv(&r1);
            let _ = sel.recv(&r2);
            let _ = sel.recv(&r3);
            let _ = sel.recv(&r4);

            loop {
                let operation = sel.select();
                let index = operation.index();
                match index {
                    0 => {
                        if let Ok(event) = operation.recv(&r1) {
                            match event {
                                MidiPanelEvent::Midi(channel, message) => {
                                    let _ = orchestrator_sender
                                        .send(OrchestratorInput::Midi(channel, message));
                                }
                                _ => {
                                    let _ = app_sender.send(EnsnareMessage::MidiPanelEvent(event));
                                }
                            }
                        }
                    }
                    1 => {
                        if let Ok(event) = operation.recv(&r2) {
                            let _ = app_sender.send(EnsnareMessage::AudioPanelEvent(event));
                        }
                    }
                    2 => {
                        if let Ok(event) = operation.recv(&r3) {
                            let _ = app_sender.send(EnsnareMessage::OrchestratorEvent(event));
                        }
                    }
                    3 => {
                        if let Ok(event) = operation.recv(&r4) {
                            match event {
                                DragDropEvent::TrackAddDevice(track_uid, key) => {
                                    let _ = orchestrator_sender
                                        .send(OrchestratorInput::TrackAddEntity(track_uid, key));
                                }
                                DragDropEvent::TrackAddPattern(
                                    track_uid,
                                    pattern_uid,
                                    position,
                                ) => {
                                    let _ = orchestrator_sender.send(
                                        OrchestratorInput::TrackPatternAdd(
                                            track_uid,
                                            pattern_uid,
                                            position,
                                        ),
                                    );
                                }
                                DragDropEvent::LinkControl(
                                    source_uid,
                                    target_uid,
                                    control_index,
                                ) => {
                                    let _ =
                                        orchestrator_sender.send(OrchestratorInput::LinkControl(
                                            source_uid,
                                            target_uid,
                                            control_index,
                                        ));
                                }
                            }
                            ctx.request_repaint();
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
                                panic!("this should have been short-circuited in spawn_channel_aggregator");
                            }
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
                            self.orchestrator_panel.update_entity_factory_uid();
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

    fn show_top(&mut self, ui: &mut Ui) {
        // if let Some(action) = self
        //     .menu_bar
        //     .show_with_action(ui, self.orchestrator_panel.is_any_track_selected())
        // {
        //     self.handle_menu_bar_action(action);
        // }
        // ui.separator();
        ui.horizontal_centered(|ui| {
            if let Ok(mut o) = self.orchestrator.lock() {
                ui.add(transport(o.transport_mut()));
            }
            self.control_panel.ui(ui);
        });
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
                self.settings_panel.toggle();
                None
            }
        };
        if let Some(input) = input {
            self.orchestrator_panel.send_to_service(input);
        }
    }

    fn show_bottom(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            eframe::egui::warn_if_debug_build(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(app_version())
            });
        });
    }

    fn show_left(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            self.palette_panel.ui(ui);
        });
    }

    fn show_right(&mut self, ui: &mut Ui) {
        ScrollArea::horizontal().show(ui, |ui| ui.label("Under Construction"));
    }

    fn show_center(&mut self, ui: &mut Ui) {
        self.orchestrator_panel.ui(ui);
        self.toasts.show(ui.ctx());
    }

    fn show_settings_panel(&mut self, ctx: &Context) {
        let mut is_settings_open = self.settings_panel.is_open();
        eframe::egui::Window::new("Settings")
            .open(&mut is_settings_open)
            .show(ctx, |ui| self.settings_panel.ui(ui));
        if self.settings_panel.is_open() && !is_settings_open {
            self.settings_panel.toggle();
        }
    }
}
impl App for Ensnare {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.handle_app_event_channel();

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

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(1366.0, 768.0)),
        ..Default::default()
    };

    if EntityFactory::initialize(register_factory_entities(EntityFactory::default())).is_err() {
        return Err(anyhow!("Couldn't set EntityFactory once_cell"));
    }
    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        return Err(anyhow!("Couldn't set DragDropManager once_cell"));
    }

    if let Err(e) = eframe::run_native(
        Ensnare::NAME,
        options,
        Box::new(|cc| Box::new(Ensnare::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
