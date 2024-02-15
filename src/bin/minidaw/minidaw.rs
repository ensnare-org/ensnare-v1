// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Main struct for MiniDaw application.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
    events::{MiniDawEvent, MiniDawEventAggregationService, MiniDawInput},
    menu::{MenuBar, MenuBarAction},
    settings::Settings,
};
use crossbeam_channel::{Select, Sender};
use eframe::{
    egui::{
        CentralPanel, Context, Event, Layout, Modifiers, ScrollArea, SidePanel, TopBottomPanel,
    },
    emath::{Align, Align2},
    epaint::Vec2,
    App, CreationContext,
};
use egui_notify::Toasts;
use ensnare::{
    app_version,
    egui::{
        ControlBar, ControlBarAction, ControlBarWidget, ObliqueStrategiesWidget,
        TimelineIconStripAction, TimelineIconStripWidget, TransportWidget,
    },
    prelude::*,
    traits::DisplaysAction,
};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

#[derive(Debug, Default)]
pub(super) struct RenderingState {
    pub(super) is_composer_visible: bool,
    pub(super) is_settings_panel_open: bool,
    pub(super) is_detail_open: bool,
    pub(super) detail_uid: Option<Uid>,
    pub(super) detail_title: String,
}

#[derive(Debug, Default)]
pub(super) struct MiniDawEphemeral {
    pub(super) is_project_performing: bool,
}

pub(super) struct MiniDaw {
    // factory creates new entities.
    factory: Arc<EntityFactory<dyn EntityBounds>>,

    // Takes a number of individual services' event channels and aggregates them
    // into a single stream that the app can consume.
    aggregator: MiniDawEventAggregationService,

    // Channels for sending commands to services.
    #[allow(dead_code)]
    audio_sender: Sender<AudioServiceInput>,
    #[allow(dead_code)]
    midi_sender: Sender<MidiServiceInput>,
    project_sender: Sender<ProjectServiceInput>,

    // A non-owning ref to the project. (ProjectService is the owner.)
    project: Option<Arc<RwLock<Project>>>,

    menu_bar: MenuBar,
    control_bar: ControlBar,
    settings: Settings,

    toasts: Toasts,

    oblique_strategies_mgr: ObliqueStrategiesWidget,

    exit_requested: bool,

    rendering_state: RenderingState,

    e: MiniDawEphemeral,

    // Copy of keyboard modifier state at top of frame
    modifiers: Modifiers,
}
impl MiniDaw {
    /// The user-visible name of the application.
    pub(super) const NAME: &'static str = "MiniDAW";

    pub(super) fn new(cc: &CreationContext, factory: EntityFactory<dyn EntityBounds>) -> Self {
        let factory = Arc::new(factory);

        let mut settings = Settings::load().unwrap_or_default();
        let audio_service = AudioService::new();
        let midi_service = MidiService::new_with(&settings.midi_settings);
        settings.set_midi_sender(midi_service.sender());
        let project_service = ProjectService::new_with(&factory);
        let control_bar = ControlBar::default();

        let mut r = Self {
            audio_sender: audio_service.sender().clone(),
            midi_sender: midi_service.input_channels.sender.clone(),
            project_sender: project_service.sender().clone(),
            aggregator: MiniDawEventAggregationService::new_with(
                audio_service,
                midi_service,
                project_service,
                settings.receiver(),
            ),
            project: Default::default(),
            menu_bar: MenuBar::new_with(&factory),
            factory,
            settings,
            control_bar,
            toasts: Toasts::default(),
            oblique_strategies_mgr: Default::default(),
            exit_requested: Default::default(),
            rendering_state: Default::default(),
            e: Default::default(),
            modifiers: Modifiers::default(),
        };

        // TODO: this works, but I'm not sure it's a good design. Is it like
        // EntityFactory and should be provided to the ProjectService
        // constructor?
        r.send_to_project(ProjectServiceInput::VisualizationQueue(
            r.control_bar.visualization_queue.clone(),
        ));

        r.spawn_app_channel_watcher(cc.egui_ctx.clone());
        r
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
        let receiver = self.aggregator.receiver().clone();
        let _ = std::thread::spawn(move || -> ! {
            let mut sel = Select::new();
            let _ = sel.recv(&receiver);
            loop {
                let _ = sel.ready();
                ctx.request_repaint();
            }
        });
    }

    fn set_window_title(ctx: &eframe::egui::Context, title: &ProjectTitle) {
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Title(format!(
            "{} - {}",
            Self::NAME,
            title.as_str()
        )));
    }

    /// Processes all the aggregated events.
    fn handle_events(&mut self, ctx: &eframe::egui::Context) {
        // As long the channel has messages in it, we'll keep handling them. We
        // don't expect a giant number of messages; otherwise we'd worry about
        // blocking the UI.
        while let Ok(event) = self.aggregator.receiver().try_recv() {
            match event {
                MiniDawEvent::MidiPanelEvent(event) => {
                    match event {
                        MidiServiceEvent::Midi(..) => {
                            // This was already forwarded to Orchestrator. Here we update the UI.
                            self.control_bar.tickle_midi_in();
                        }
                        MidiServiceEvent::MidiOut => self.control_bar.tickle_midi_out(),
                        MidiServiceEvent::SelectInput(_) => {
                            // TODO: save selection in prefs
                        }
                        MidiServiceEvent::SelectOutput(_) => {
                            // TODO: save selection in prefs
                        }
                        MidiServiceEvent::InputPortsRefreshed(ports) => {
                            // TODO: remap any saved preferences to ports that we've found
                            self.settings.handle_midi_input_port_refresh(&ports);
                        }
                        MidiServiceEvent::OutputPortsRefreshed(ports) => {
                            // TODO: remap any saved preferences to ports that we've found
                            self.settings.handle_midi_output_port_refresh(&ports);
                        }
                    }
                }
                MiniDawEvent::AudioServiceEvent(event) => match event {
                    AudioServiceEvent::Reset(_sample_rate, _channel_count, _queue) => {
                        // Already forwarded by aggregator to project.
                        self.update_orchestrator_audio_interface_config();
                    }
                    AudioServiceEvent::NeedsAudio(_count) => {
                        // Forward was already handled by aggregator.
                    }
                    AudioServiceEvent::Underrun => {
                        eprintln!("Warning: audio buffer underrun")
                    }
                },
                MiniDawEvent::ProjectServiceEvent(event) => match event {
                    ProjectServiceEvent::TitleChanged(title) => {
                        Self::set_window_title(ctx, &title);
                    }
                    ProjectServiceEvent::IsPerformingChanged(is_performing) => {
                        self.e.is_project_performing = is_performing;
                    }
                    ProjectServiceEvent::Quit => {
                        // Nothing to do
                    }
                    ProjectServiceEvent::Loaded(new_project) => {
                        if let Ok(project) = new_project.read() {
                            let title = project.title.clone().unwrap_or_default();

                            // TODO: this duplicates TitleChanged. Should
                            // the service be in charge of sending that
                            // event after Loaded? Whose responsibility is it?
                            Self::set_window_title(ctx, &title);

                            if let Some(load_path) = project.load_path() {
                                self.toasts
                                    .success(format!(
                                        "Loaded {} from {}",
                                        title,
                                        load_path.display().to_string()
                                    ))
                                    .set_duration(Some(Duration::from_secs(2)));
                            }
                        }
                        self.project = Some(new_project);
                    }
                    ProjectServiceEvent::LoadFailed(path, e) => {
                        self.toasts
                            .error(format!("Error loading from {path:?}: {e:?}").to_string())
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                    ProjectServiceEvent::Saved(save_path) => {
                        // TODO: this should happen only if the save operation was
                        // explicit. Autosaves should be invisible.
                        self.toasts
                            .success(format!("Saved to {}", save_path.display()).to_string())
                            .set_duration(Some(Duration::from_secs(2)));
                    }
                    ProjectServiceEvent::SaveFailed(e) => {
                        self.toasts
                            .error(format!("Error saving {}", e).to_string())
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                    ProjectServiceEvent::Exported(export_path) => {
                        self.toasts
                            .success(format!("Exported to {}", export_path.display()).to_string())
                            .set_duration(Some(Duration::from_secs(2)));
                    }
                    ProjectServiceEvent::ExportFailed(e) => {
                        self.toasts
                            .error(format!("Error exporting {}", e).to_string())
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                    ProjectServiceEvent::Midi(..) => {
                        panic!("ProjectServiceEvent::Midi should be handled by the aggregation service and never forwarded")
                    }
                },
                MiniDawEvent::Quit => {
                    eprintln!("MiniDawEvent::Quit");
                }
            }
        }
    }

    fn update_orchestrator_audio_interface_config(&mut self) {
        let sample_rate = self.settings.audio_settings.sample_rate();
        self.send_to_project(ProjectServiceInput::ProjectSetSampleRate(sample_rate));
    }

    fn show_top(&mut self, ui: &mut eframe::egui::Ui) {
        self.menu_bar.ui(ui);
        let menu_action = self.menu_bar.take_action();
        self.handle_menu_bar_action(menu_action);
        ui.separator();

        let mut control_bar_action = None;
        ui.horizontal_centered(|ui| {
            if let Some(project) = self.project.as_mut() {
                if let Ok(mut project) = project.write() {
                    if ui
                        .add(TransportWidget::widget(&mut project.transport))
                        .changed()
                    {
                        project.notify_transport_tempo_change();
                        project.notify_transport_time_signature_change();
                    }
                }
            } else {
                // there might be some flicker here while we wait for the
                // project to first come into existence
            }
            ui.add(ControlBarWidget::widget(
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
        match action {
            ControlBarAction::Play => self.send_to_project(ProjectServiceInput::ProjectPlay),
            ControlBarAction::Stop => self.send_to_project(ProjectServiceInput::ProjectStop),
            ControlBarAction::New => self.send_to_project(ProjectServiceInput::ProjectNew),

            ControlBarAction::Open(path) => {
                {
                    let this = &mut *self;
                    this.send_to_project(ProjectServiceInput::ProjectLoad(path));
                };
            }
            ControlBarAction::Save(path) => {
                {
                    let this = &mut *self;
                    let path = Some(path);
                    this.send_to_project(ProjectServiceInput::ProjectSave(path));
                };
            }
            ControlBarAction::ToggleSettings => {
                self.rendering_state.is_settings_panel_open =
                    !self.rendering_state.is_settings_panel_open;
            }
            ControlBarAction::ExportToWav(path) => {
                self.send_to_project(ProjectServiceInput::ProjectExportToWav(Some(path)))
            }
        }
    }

    fn show_bottom(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            eframe::egui::warn_if_debug_build(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(app_version());
                if let Some(seed) = self.oblique_strategies_mgr.check_seed() {
                    ui.add(ObliqueStrategiesWidget::widget(seed));
                }
            });
        });
        self.toasts.show(ui.ctx());
    }

    fn show_left(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.add(EntityPaletteWidget::widget(self.factory.sorted_keys()))
        });
    }

    fn show_right(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            if let Some(uid) = self.rendering_state.detail_uid {
                if let Some(project) = self.project.as_mut() {
                    if let Ok(mut project) = project.write() {
                        ui.heading(self.rendering_state.detail_title.as_str());
                        ui.separator();
                        let mut action = None;
                        if let Some(entity) = project.orchestrator.entity_repo.entity_mut(uid) {
                            entity.ui(ui);
                            action = entity.take_action();
                        }
                        if let Some(action) = action {
                            match action {
                                DisplaysAction::Link(source, index) => match source {
                                    ControlLinkSource::Entity(source_uid) => {
                                        let _ = project.link(source_uid, uid, index);
                                    }
                                    ControlLinkSource::Path(path_uid) => {
                                        let _ = project.link_path(path_uid, uid, index);
                                    }
                                    ControlLinkSource::None => panic!(),
                                },
                            }
                        }
                    }
                }
            } else {
                ui.heading("No instrument selected");
            }
        });
    }

    fn show_center(&mut self, ui: &mut eframe::egui::Ui) {
        let mut action = None;
        ui.add(TimelineIconStripWidget::widget(&mut action));
        if let Some(action) = action {
            match action {
                TimelineIconStripAction::NextTimelineView => {
                    self.send_to_project(ProjectServiceInput::NextTimelineDisplayer);
                }
                TimelineIconStripAction::ShowComposer => {
                    self.rendering_state.is_composer_visible =
                        !self.rendering_state.is_composer_visible;
                }
            }
        }
        let mut action = None;
        if let Some(project) = self.project.as_mut() {
            if let Ok(mut project) = project.write() {
                project.view_state.cursor = Some(project.transport.current_time());
                project.view_state.view_range = Self::calculate_project_view_range(
                    &project.time_signature(),
                    project.composer.extent(),
                );
                eframe::egui::Window::new("Composer")
                    .open(&mut self.rendering_state.is_composer_visible)
                    .default_width(ui.available_width())
                    .anchor(
                        eframe::emath::Align2::LEFT_BOTTOM,
                        eframe::epaint::vec2(5.0, 5.0),
                    )
                    .show(ui.ctx(), |ui| {
                        let response = ui.add(ComposerWidget::widget(&mut project.composer));
                        response
                    });

                let _ = ui.add(ProjectWidget::widget(&mut project, &mut action));
            }
        }
        if let Some(action) = action {
            self.handle_project_action(action);
        }

        // If we're performing, then we know the screen is updating, so we
        // should draw it.
        if self.e.is_project_performing {
            ui.ctx().request_repaint();
        }
    }

    fn show_settings_panel(&mut self, ctx: &Context) {
        eframe::egui::Window::new("Settings")
            .open(&mut self.rendering_state.is_settings_panel_open)
            .auto_sized()
            .anchor(Align2::CENTER_CENTER, Vec2::default())
            .show(ctx, |ui| self.settings.ui(ui));
    }

    fn handle_input_events(&mut self, ctx: &eframe::egui::Context) {
        ctx.input(|i| {
            self.modifiers = i.modifiers.clone();

            for e in i.events.iter() {
                match e {
                    eframe::egui::Event::Key {
                        repeat,
                        modifiers,
                        key,
                        pressed,
                        physical_key,
                    } => {
                        if !repeat && !modifiers.any() {
                            self.send_to_project(ProjectServiceInput::KeyEvent(
                                *key,
                                *pressed,
                                *physical_key,
                            ));
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

    fn handle_project_action(&mut self, action: ProjectAction) {
        match action {
            ProjectAction::NewDeviceForTrack(track_uid, key) => {
                self.send_to_project(ProjectServiceInput::TrackAddEntity(
                    track_uid,
                    EntityKey::from(key),
                ));
            }
            ProjectAction::SelectEntity(uid, title) => {
                // This is a view-only thing, so we can add a field in this
                // struct and use it to decide what to display. No need to get
                // Project involved.
                self.rendering_state.detail_uid = Some(uid);
                self.rendering_state.detail_title = title.clone();
                self.rendering_state.is_detail_open = true;
            }
            ProjectAction::RemoveEntity(uid) => {
                self.send_to_project(ProjectServiceInput::ProjectRemoveEntity(uid))
            }
        }
    }

    fn handle_menu_bar_action(&mut self, action: Option<MenuBarAction>) {
        let Some(action) = action else { return };
        match action {
            MenuBarAction::Quit => self.exit_requested = true,
            MenuBarAction::ProjectNew => self.send_to_project(ProjectServiceInput::ProjectNew),
            MenuBarAction::ProjectOpen => {
                self.send_to_project(ProjectServiceInput::ProjectLoad(PathBuf::from(
                    "minidaw-project.json",
                )));
            }
            MenuBarAction::ProjectSave => {
                self.send_to_project(ProjectServiceInput::ProjectSave(None))
            }
            MenuBarAction::ProjectExportToWav => self.send_to_project(
                ProjectServiceInput::ProjectExportToWav(Some(PathBuf::from("minidaw-project.wav"))),
            ),
            MenuBarAction::TrackNewMidi => {
                self.send_to_project(ProjectServiceInput::TrackNewMidi);
            }
            MenuBarAction::TrackNewAudio => {
                self.send_to_project(ProjectServiceInput::TrackNewAudio);
            }
            MenuBarAction::TrackNewAux => {
                self.send_to_project(ProjectServiceInput::TrackNewAux);
            }
            MenuBarAction::TrackDuplicate => todo!(),
            MenuBarAction::TrackDelete => todo!(),
            MenuBarAction::TrackRemoveSelectedPatterns => todo!(),
            MenuBarAction::TrackAddThing(_) => todo!(),
            MenuBarAction::ComingSoon => todo!(),
        }
    }

    fn check_drag_and_drop(&mut self) {
        // if let Some((source, target)) = DragDropManager::check_and_clear_drop_event() {
        //     match source {
        //         DragSource::NewDevice(ref key) => match target {
        //             DropTarget::Controllable(_, _) => todo!(),
        //             DropTarget::Track(track_uid) => {
        //                 self.send_to_project(ProjectServiceInput::TrackAddEntity(
        //                     track_uid,
        //                     EntityKey::from(key),
        //                 ));
        //             }
        //             DropTarget::TrackPosition(_, _) => {
        //                 eprintln!("DropTarget::TrackPosition not implemented - ignoring");
        //             }
        //         },
        //         DragSource::Pattern(pattern_uid) => match target {
        //             DropTarget::Controllable(_, _) => todo!(),
        //             DropTarget::Track(_) => todo!(),
        //             DropTarget::TrackPosition(track_uid, position) => {
        //                 self.send_to_project(ProjectServiceInput::PatternArrange(
        //                     track_uid,
        //                     pattern_uid,
        //                     position,
        //                 ));
        //             }
        //         },
        //         DragSource::ControlSource(source_uid) => match target {
        //             DropTarget::Controllable(target_uid, index) => {
        //                 self.send_to_project(ProjectServiceInput::ProjectLinkControl(
        //                     source_uid, target_uid, index,
        //                 ));
        //             }
        //             DropTarget::Track(_) => todo!(),
        //             DropTarget::TrackPosition(_, _) => todo!(),
        //         },
        //     }
        // }
    }

    #[allow(dead_code)]
    fn send_to_audio(&self, input: AudioServiceInput) {
        if let Err(e) = self.audio_sender.send(input) {
            eprintln!("Error {e} while sending AudioServiceInput");
        }
    }

    #[allow(dead_code)]
    fn send_to_midi(&self, input: MidiServiceInput) {
        if let Err(e) = self.midi_sender.send(input) {
            eprintln!("Error {e} while sending MidiServiceInput");
        }
    }

    fn send_to_project(&self, input: ProjectServiceInput) {
        if let Err(e) = self.project_sender.send(input) {
            eprintln!("Error {e} while sending ProjectServiceInput");
        }
    }

    fn calculate_project_view_range(
        time_signature: &TimeSignature,
        extent: TimeRange,
    ) -> ViewRange {
        ViewRange(extent.0.start..extent.0.end + MusicalTime::new_with_bars(time_signature, 1))
    }
}
impl App for MiniDaw {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        self.handle_events(ctx);
        self.handle_input_events(ctx);

        TopBottomPanel::top("top-panel")
            .resizable(false)
            .exact_height(64.0)
            .show(ctx, |ui| self.show_top(ui));
        TopBottomPanel::bottom("bottom-panel")
            .resizable(false)
            .exact_height(24.0)
            .show(ctx, |ui| self.show_bottom(ui));
        SidePanel::left("left-panel")
            .resizable(true)
            .default_width(240.0)
            .width_range(160.0..=480.0)
            .show(ctx, |ui| self.show_left(ui));
        SidePanel::right("right-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0)
            .show(ctx, |ui| self.show_right(ui));
        CentralPanel::default().show(ctx, |ui| self.show_center(ui));

        self.show_settings_panel(ctx);

        self.check_drag_and_drop();

        if self.exit_requested {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.settings.has_been_saved() {
            let _ = self.settings.save();
        }
        let _ = self.aggregator.sender().send(MiniDawInput::Quit);
    }
}
