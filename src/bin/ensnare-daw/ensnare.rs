// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Main struct for Ensnare DAW application.

use crate::{
    menu::{MenuBar, MenuBarAction},
    project::DawProject,
    settings::{Settings, SettingsPanel},
};
use crossbeam_channel::{Select, Sender};
use eframe::{
    egui::{
        CentralPanel, Context, Event, FontData, FontDefinitions, Layout, Modifiers, ScrollArea,
        SidePanel, TextStyle, TopBottomPanel,
    },
    emath::Align,
    epaint::{Color32, FontFamily, FontId},
    App, CreationContext,
};
use egui_toast::{Toast, ToastOptions, Toasts};
use ensnare::{all_entities::EntityWrapper, app_version, prelude::*, project::ProjectTitle};
use std::{ops::DerefMut, path::PathBuf, sync::Arc};

// TODO: clean these up. An app should need to use only the top ensnare crate,
// and ideally it can get by with just importing prelude::*.
use ensnare_cores_egui::widgets::timeline::{timeline_icon_strip, TimelineIconStripAction};
use ensnare_egui_widgets::{oblique_strategies, ObliqueStrategiesManager};
use ensnare_orchestration::{egui::entity_palette, ProjectAction};
use ensnare_services::{control_bar_widget, ControlBarAction};

#[allow(dead_code)]
#[derive(Debug, derive_more::Display)]
enum LoadError {
    Todo,
}

#[allow(dead_code)]
#[derive(Debug, derive_more::Display)]
enum SaveError {
    Todo,
}

enum EnsnareMessage {
    MidiPanelEvent(MidiPanelEvent),
    AudioPanelEvent(AudioPanelEvent),
    OrchestratorEvent(OrchestratorEvent),
    ProjectLoaded(Result<DawProject, LoadError>),
    ProjectSaved(Result<PathBuf, SaveError>),
}

pub(super) struct Ensnare {
    factory: Arc<EntityFactory<dyn EntityWrapper>>,

    event_channel: ChannelPair<EnsnareMessage>,

    project: DawProject,

    menu_bar: MenuBar,
    control_bar: ControlBar,
    orchestrator_service: OrchestratorService<dyn EntityWrapper>,
    settings_panel: SettingsPanel,

    toasts: Toasts,

    oblique_strategies_mgr: ObliqueStrategiesManager,

    exit_requested: bool,

    keyboard_events_sender: Sender<Event>,

    pub is_settings_panel_open: bool,

    // Copy of keyboard modifier state at top of frame
    modifiers: Modifiers,
}
impl Ensnare {
    /// The user-visible name of the application.
    pub(super) const NAME: &'static str = "Ensnare";

    /// internal-only key for regular font.
    const FONT_REGULAR: &'static str = "font-regular";
    /// internal-only key for bold font.
    const FONT_BOLD: &'static str = "font-bold";
    /// internal-only key for monospaced font.
    const FONT_MONO: &'static str = "font-mono";

    pub(super) fn new(cc: &CreationContext, factory: EntityFactory<dyn EntityWrapper>) -> Self {
        Self::initialize_fonts(&cc.egui_ctx);
        Self::initialize_visuals(&cc.egui_ctx);
        Self::initialize_style(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let project = DawProject::new_project();
        let orchestrator = Arc::clone(&project.orchestrator);
        let settings = Settings::load().unwrap_or_default();
        let control_bar = ControlBar::default();
        let factory = Arc::new(factory);

        let mut r = Self {
            menu_bar: MenuBar::new_with(&factory),
            orchestrator_service: OrchestratorService::<dyn EntityWrapper>::new_with(
                &orchestrator,
                &factory,
            ),
            factory,
            project,
            settings_panel: SettingsPanel::new_with(
                settings,
                &orchestrator,
                Some(control_bar.sample_channel.sender.clone()),
            ),
            control_bar,
            keyboard_events_sender: orchestrator
                .lock()
                .unwrap()
                .keyboard_controller
                .sender()
                .clone(),
            toasts: Toasts::new()
                .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(Direction::BottomUp),
            event_channel: Default::default(),
            oblique_strategies_mgr: Default::default(),
            exit_requested: Default::default(),
            is_settings_panel_open: Default::default(),
            modifiers: Modifiers::default(),
        };
        r.spawn_app_channel_watcher(cc.egui_ctx.clone());
        r.spawn_channel_aggregator();
        r
    }

    fn set_project(&mut self, project: DawProject) {
        self.orchestrator_service
            .send_to_service(OrchestratorInput::SetOrchestrator(Arc::clone(
                &project.orchestrator,
            )));
        self.settings_panel.exit();
        self.settings_panel = SettingsPanel::new_with(
            Settings::default(), // TODO
            &project.orchestrator,
            Some(self.control_bar.sample_channel.sender.clone()),
        );
        self.keyboard_events_sender = project
            .orchestrator
            .lock()
            .unwrap()
            .keyboard_controller
            .sender()
            .clone();
        self.project = project;
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
        let r0 = self.settings_panel.midi_service.receiver().clone();
        let r1 = self.settings_panel.audio_service.receiver().clone();
        let r2 = self.orchestrator_service.receiver().clone();

        let app_sender = self.event_channel.sender.clone();
        let orchestrator_sender = self.orchestrator_service.sender().clone();

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
                                self.control_bar.tickle_midi_in();
                            }
                            MidiPanelEvent::MidiOut => self.control_bar.tickle_midi_out(),
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
                            let title = title.unwrap_or(ProjectTitle::default().into());
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
                    EnsnareMessage::ProjectLoaded(result) => match result {
                        Ok(project) => {
                            self.set_project(project);
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Success,
                                text: format!(
                                    "Loaded {} from {}",
                                    self.project.title,
                                    self.project.load_path.as_ref().unwrap().display()
                                )
                                .into(),
                                options: ToastOptions::default()
                                    .duration_in_seconds(2.0)
                                    .show_progress(false),
                            });
                        }
                        Err(err) => {
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Error,
                                text: format!("Error loading {}", err).into(),
                                options: ToastOptions::default().duration_in_seconds(5.0),
                            });
                        }
                    },
                    EnsnareMessage::ProjectSaved(result) => match result {
                        Ok(save_path) => {
                            // TODO: this should happen only if the save operation was
                            // explicit. Autosaves should be invisible.
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Success,
                                text: format!("Saved to {}", save_path.display()).into(),
                                options: ToastOptions::default()
                                    .duration_in_seconds(1.0)
                                    .show_progress(false),
                            });
                        }
                        Err(err) => {
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Error,
                                text: format!("Error saving {}", err).into(),
                                options: ToastOptions::default().duration_in_seconds(5.0),
                            });
                        }
                    },
                }
            } else {
                break;
            }
        }
    }

    fn update_orchestrator_audio_interface_config(&mut self) {
        let sample_rate = self.settings_panel.audio_service.sample_rate();
        if let Ok(mut o) = self.project.orchestrator.lock() {
            o.update_sample_rate(sample_rate);
        }
    }

    fn show_top(&mut self, ui: &mut eframe::egui::Ui) {
        self.menu_bar
            .set_is_any_track_selected(self.orchestrator_service.is_any_track_selected());
        self.menu_bar.ui(ui);
        let menu_action = self.menu_bar.take_action();
        self.handle_menu_bar_action(menu_action);
        ui.separator();

        let mut control_bar_action = None;
        ui.horizontal_centered(|ui| {
            if let Ok(mut o) = self.project.orchestrator.lock() {
                ui.add(transport(&mut o.transport));
            }
            ui.add(control_bar_widget(
                &mut self.control_bar,
                &mut control_bar_action,
            ));
        });
        ui.add_space(2.0);
        if let Some(action) = control_bar_action {
            self.handle_control_panel_action(action);
        }
    }

    fn handle_control_panel_action(&mut self, action: ControlBarAction) {
        let input = match action {
            ControlBarAction::Play => Some(OrchestratorInput::ProjectPlay),
            ControlBarAction::Stop => Some(OrchestratorInput::ProjectStop),
            ControlBarAction::New => {
                self.handle_project_new();
                None
            }
            ControlBarAction::Open(path) => {
                self.handle_project_load(Some(path));
                None
            }
            ControlBarAction::Save(path) => {
                self.handle_project_save(Some(path));
                None
            }
            ControlBarAction::ToggleSettings => {
                self.is_settings_panel_open = !self.is_settings_panel_open;
                None
            }
        };
        if let Some(input) = input {
            self.orchestrator_service.send_to_service(input);
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
        ScrollArea::vertical().show(ui, |ui| ui.add(entity_palette(self.factory.sorted_keys())));
    }

    fn show_right(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::horizontal().show(ui, |ui| ui.label("Under Construction"));
    }

    fn show_center(&mut self, ui: &mut eframe::egui::Ui) {
        let mut action = None;
        ui.add(timeline_icon_strip(&mut action));
        if let Some(action) = action {
            match action {
                TimelineIconStripAction::NextTimelineView => {
                    self.project.switch_to_next_frontmost_timeline_displayer();
                }
                TimelineIconStripAction::ShowPianoRoll => {
                    self.project.is_piano_roll_visible = !self.project.is_piano_roll_visible
                }
            }
        }
        self.project.show_piano_roll(ui);
        self.project.show_detail(ui);
        let mut view_range = self.project.view_range.clone();
        let mut action = None;
        if let Ok(mut o) = self.project.orchestrator.lock() {
            let _ = ui.add(project_widget::<dyn EntityWrapper>(
                &self.project,
                o.deref_mut(),
                &mut view_range,
                &mut action,
            ));
        }
        self.project.view_range = view_range;
        if let Some(action) = action {
            self.handle_action(action);
        }
        // If we're performing, then we know the screen is updating, so we
        // should draw it..
        if let Ok(o) = self.project.orchestrator.lock() {
            if o.is_performing() {
                ui.ctx().request_repaint();
            }
        }

        self.toasts.show(ui.ctx());
    }

    fn show_settings_panel(&mut self, ctx: &Context) {
        eframe::egui::Window::new("Settings")
            .open(&mut self.is_settings_panel_open)
            .show(ctx, |ui| self.settings_panel.ui(ui));
    }

    fn handle_input_events(&mut self, ctx: &eframe::egui::Context) {
        ctx.input(|i| {
            self.modifiers = i.modifiers.clone();

            for e in i.events.iter() {
                match e {
                    eframe::egui::Event::Key {
                        repeat, modifiers, ..
                    } => {
                        if !repeat && !modifiers.any() {
                            let _ = self.keyboard_events_sender.send(e.clone());
                        }
                    }
                    Event::MouseWheel {
                        delta, modifiers, ..
                    } => {
                        if modifiers.command_only() {
                            if delta.y > 0.0 {
                                eprintln!("zoom timeline in")
                            } else {
                                eprintln!("zoom timeline out")
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn handle_action(&mut self, action: ProjectAction) {
        match action {
            ProjectAction::ClickTrack(_track_uid) => {
                // TODO: this was in orchestrator_panel, and I'm not too fond of
                // its design.
                //
                // self.track_selection_set.lock().unwrap().click(&track_uid,
                //     self.is_control_only_down);
            }
            ProjectAction::DoubleClickTrack(_track_uid) => {
                // This used to expand/collapse, but that's gone.
            }
            ProjectAction::NewDeviceForTrack(track_uid, key) => {
                if let Ok(mut o) = self.project.orchestrator.lock() {
                    let uid = o.mint_entity_uid();
                    if let Some(entity) = self.factory.new_entity(&key, uid) {
                        let _ = o.add_entity(&track_uid, entity);
                    }
                }
            }
            ProjectAction::EntitySelected(uid, name) => {
                self.project.select_detail(uid, name);
            }
        }
    }

    fn handle_menu_bar_action(&mut self, action: Option<MenuBarAction>) {
        if let Some(action) = action {
            match action {
                MenuBarAction::Quit => self.exit_requested = true,
                MenuBarAction::ProjectNew => {
                    self.handle_project_new();
                }
                MenuBarAction::ProjectOpen => {
                    self.handle_project_load(Some(PathBuf::from("ensnare-project.json")));
                }
                MenuBarAction::ProjectSave => {
                    self.handle_project_save(None);
                }
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

    fn handle_project_new(&mut self) {
        self.set_project(DawProject::new_project());
    }

    fn handle_project_save(&mut self, path: Option<PathBuf>) {
        if let Ok(save_path) = self.project.save(path) {
            let _ = self
                .event_channel
                .sender
                .send(EnsnareMessage::ProjectSaved(Ok(save_path)));
        }
    }

    fn handle_project_load(&mut self, path: Option<PathBuf>) {
        // TODO: pop up chooser if needed
        if let Ok(project) = DawProject::load(path.unwrap(), &self.factory) {
            let _ = self
                .event_channel
                .sender
                .send(EnsnareMessage::ProjectLoaded(Ok(project)));
        }
    }

    fn check_drag_and_drop(&mut self) {
        if let Some((source, target)) = DragDropManager::check_and_clear_drop_event() {
            let mut handled = false;
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
                    DropTarget::TrackPosition(track_uid, position) => {
                        self.project
                            .request_pattern_add(track_uid, pattern_uid, position);
                        handled = true;
                        None
                    }
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
                let _ = self.orchestrator_service.send_to_service(input);
            } else {
                if !handled {
                    eprintln!("WARNING: unhandled DnD pair: {source:?} {target:?}");
                }
            }
        }
    }
}
impl App for Ensnare {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.handle_app_event_channel();
        self.handle_input_events(ctx);
        self.orchestrator_service
            .set_control_only_down(self.modifiers.command_only());

        // TODO - too much work
        let project_title_str: String = self.project.title.clone().into();
        //        frame.set_window_title(project_title_str.as_str());

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
            //   frame.close();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.settings_panel.settings.has_been_saved() {
            let _ = self.settings_panel.settings.save();
        }
        self.settings_panel.exit();
        self.orchestrator_service.exit();
    }
}
