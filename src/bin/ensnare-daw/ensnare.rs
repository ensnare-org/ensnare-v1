// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! Main struct for Ensnare DAW application.

use crate::{
    events::{EnsnareEvent, EnsnareEventAggregationService},
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
use egui_toast::{Toast, ToastOptions, Toasts};
use ensnare::{app_version, prelude::*};
use ensnare_new_stuff::project::Project;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

// TODO: clean these up. An app should need to use only the top ensnare crate,
// and ideally it can get by with just importing prelude::*.
use ensnare_cores_egui::widgets::timeline::{timeline_icon_strip, TimelineIconStripAction};
use ensnare_egui_widgets::{oblique_strategies, ObliqueStrategiesManager};
use ensnare_orchestration::{egui::entity_palette, ProjectAction};
use ensnare_services::{control_bar_widget, AudioServiceInput, ControlBarAction};

#[derive(Debug, Default)]
pub(super) struct RenderingState {
    pub(super) is_piano_roll_visible: bool,
    pub(super) is_settings_panel_open: bool,
    pub(super) view_range: ViewRange,
    pub(super) is_detail_open: bool,
    pub(super) detail_uid: Option<Uid>,
    pub(super) detail_title: String,
}

#[derive(Debug, Default)]
pub(super) struct EnsnareEphemeral {
    pub(super) is_project_performing: bool,
}

pub(super) struct Ensnare {
    // factory creates new entities.
    factory: Arc<EntityFactory<dyn EntityBounds>>,

    // Takes a number of individual services' event channels and aggregates them
    // into a single stream that the app can consume.
    aggregator: EnsnareEventAggregationService,

    // Channels for sending commands to services.
    audio_sender: Sender<AudioServiceInput>,
    midi_sender: Sender<MidiInterfaceInput>,
    project_sender: Sender<ProjectServiceInput>,

    // A non-owning ref to the project. (ProjectService is the owner.)
    project: Option<Arc<RwLock<Project>>>,

    menu_bar: MenuBar,
    control_bar: ControlBar,
    //    orchestrator_service: OrchestratorService<dyn EntityBounds>,
    settings: Settings,

    toasts: Toasts,

    oblique_strategies_mgr: ObliqueStrategiesManager,

    exit_requested: bool,

    rendering_state: RenderingState,

    e: EnsnareEphemeral,

    // Copy of keyboard modifier state at top of frame
    modifiers: Modifiers,
}
impl Ensnare {
    /// The user-visible name of the application.
    pub(super) const NAME: &'static str = "Ensnare";

    pub(super) fn new(cc: &CreationContext, factory: EntityFactory<dyn EntityBounds>) -> Self {
        let factory = Arc::new(factory);

        let settings = Settings::load().unwrap_or_default();
        let audio_service = AudioService::new_with();
        let midi_service = MidiService::new_with(&settings.midi_settings);
        let project_service = ProjectService::new_with(&factory);

        let control_bar = ControlBar::default();
        // let settings_panel = SettingsPanel::new_with(
        //     settings,
        //     &orchestrator,
        //     Some(control_bar.sample_channel.sender.clone()),
        // );
        let mut r = Self {
            aggregator: EnsnareEventAggregationService::new_with(
                midi_service.receiver(),
                audio_service.receiver(),
                project_service.receiver(),
            ),
            audio_sender: audio_service.sender().clone(),
            midi_sender: midi_service.sender().clone(),
            project_sender: project_service.sender().clone(),
            project: Default::default(),
            menu_bar: MenuBar::new_with(&factory),
            factory,
            settings,
            control_bar,
            toasts: Toasts::new()
                .anchor(eframe::emath::Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(eframe::egui::Direction::BottomUp),
            oblique_strategies_mgr: Default::default(),
            exit_requested: Default::default(),
            rendering_state: Default::default(),
            e: Default::default(),
            modifiers: Modifiers::default(),
        };
        // TODO TEMP to make initial project more interesting
        r.send_to_project(ProjectServiceInput::TempInsert16RandomPatterns);

        r.spawn_app_channel_watcher(cc.egui_ctx.clone());
        r
    }

    fn set_project(&mut self, project: Project) {
        // self.orchestrator_service
        //     .send_to_service(OrchestratorInput::SetOrchestrator(Arc::clone(
        //         &project.orchestrator,
        //     )));
        // self.settings_panel.exit();
        // self.settings_panel = SettingsPanel::new_with(
        //     Settings::default(), // TODO
        //     &project.orchestrator,
        //     Some(self.control_bar.sample_channel.sender.clone()),
        // );
        // self.keyboard_events_sender = project
        //     .orchestrator
        //     .lock()
        //     .unwrap()
        //     .keyboard_controller
        //     .sender()
        //     .clone();
        // self.project = project;
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

    /// Processes all the aggregated events
    fn handle_app_event_channel(&mut self, ctx: &eframe::egui::Context) {
        // As long the channel has messages in it, we'll keep handling them. We
        // don't expect a giant number of messages; otherwise we'd worry about
        // blocking the UI.
        loop {
            if let Ok(m) = self.aggregator.receiver().try_recv() {
                match m {
                    EnsnareEvent::MidiPanelEvent(event) => {
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
                            MidiPanelEvent::InputPortsRefreshed(ports) => {
                                // TODO: remap any saved preferences to ports that we've found
                                self.settings.handle_midi_input_port_refresh(&ports);
                            }
                            MidiPanelEvent::OutputPortsRefreshed(ports) => {
                                // TODO: remap any saved preferences to ports that we've found
                                self.settings.handle_midi_output_port_refresh(&ports);
                            }
                        }
                    }
                    EnsnareEvent::AudioServiceEvent(event) => match event {
                        AudioServiceEvent::Changed(queue) => {
                            self.update_orchestrator_audio_interface_config();
                            self.send_to_project(ProjectServiceInput::AudioQueue(queue));
                        }
                        AudioServiceEvent::NeedsAudio(count) => {
                            self.send_to_project(ProjectServiceInput::NeedsAudio(count));
                        }
                    },
                    EnsnareEvent::ProjectServiceEvent(event) => match event {
                        ProjectServiceEvent::TitleChanged(title) => {
                            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Title(title));
                        }
                        ProjectServiceEvent::IsPerformingChanged(is_performing) => {
                            self.e.is_project_performing = is_performing;
                        }
                        ProjectServiceEvent::Quit => {
                            // Nothing to do
                        }
                        ProjectServiceEvent::Loaded(new_project) => {
                            if let Ok(project) = new_project.read() {
                                let title = project.title.clone();

                                // TODO: this duplicates TitleChanged. Should
                                // the service be in charge of sending that
                                // event after Loaded? Whose responsibility is it?
                                ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Title(
                                    title.to_string(),
                                ));

                                let load_path = if let Some(load_path) = project.load_path.as_ref()
                                {
                                    load_path.display().to_string()
                                } else {
                                    String::from("unknown")
                                };
                                self.toasts.add(Toast {
                                    kind: egui_toast::ToastKind::Success,
                                    text: format!("Loaded {} from {}", title, load_path).into(),
                                    options: ToastOptions::default()
                                        .duration_in_seconds(2.0)
                                        .show_progress(false),
                                });
                            }
                            self.project = Some(new_project);
                        }
                        ProjectServiceEvent::LoadFailed(e) => todo!("{e:?}"),
                        ProjectServiceEvent::Saved(save_path) => {
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
                        ProjectServiceEvent::SaveFailed(e) => {
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Error,
                                text: format!("Error saving {}", e).into(),
                                options: ToastOptions::default().duration_in_seconds(5.0),
                            });
                        }
                    },
                    // EnsnareEvent::OrchestratorEvent(event) => match event {
                    //     OrchestratorEvent::Tempo(_tempo) => {
                    //         // This is (usually) an acknowledgement that Orchestrator
                    //         // got our request to change, so we don't need to do
                    //         // anything.
                    //     }
                    //     OrchestratorEvent::Quit => {
                    //         eprintln!("OrchestratorEvent::Quit")
                    //     }
                    //     OrchestratorEvent::Loaded(path, title) => {
                    //         let title = title.unwrap_or(ProjectTitle::default().into());
                    //         self.toasts.add(Toast {
                    //             kind: egui_toast::ToastKind::Success,
                    //             text: format!("Loaded {} from {}", title, path.display()).into(),
                    //             options: ToastOptions::default()
                    //                 .duration_in_seconds(2.0)
                    //                 .show_progress(false),
                    //         });
                    //     }
                    //     OrchestratorEvent::LoadError(path, error) => {
                    //         self.toasts.add(Toast {
                    //             kind: egui_toast::ToastKind::Error,
                    //             text: format!("Error loading {}: {}", path.display(), error).into(),
                    //             options: ToastOptions::default().duration_in_seconds(5.0),
                    //         });
                    //     }
                    //     OrchestratorEvent::Saved(path) => {
                    //         // TODO: this should happen only if the save operation was
                    //         // explicit. Autosaves should be invisible.
                    //         self.toasts.add(Toast {
                    //             kind: egui_toast::ToastKind::Success,
                    //             text: format!("Saved to {}", path.display()).into(),
                    //             options: ToastOptions::default()
                    //                 .duration_in_seconds(1.0)
                    //                 .show_progress(false),
                    //         });
                    //     }
                    //     OrchestratorEvent::SaveError(path, error) => {
                    //         self.toasts.add(Toast {
                    //             kind: egui_toast::ToastKind::Error,
                    //             text: format!("Error saving {}: {}", path.display(), error).into(),
                    //             options: ToastOptions::default().duration_in_seconds(5.0),
                    //         });
                    //     }
                    //     OrchestratorEvent::New => {
                    //         // No special UI needed for this.
                    //     }
                    // },
                    EnsnareEvent::ProjectLoaded(result) => match result {
                        Ok(project) => {
                            let title = project.title.clone();
                            let load_path = if let Some(load_path) = project.load_path.as_ref() {
                                load_path.display().to_string()
                            } else {
                                String::from("unknown")
                            };
                            self.set_project(project);
                            self.toasts.add(Toast {
                                kind: egui_toast::ToastKind::Success,
                                text: format!("Loaded {} from {}", title, load_path).into(),
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
                    EnsnareEvent::ProjectSaved(result) => match result {
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
        let sample_rate = self.settings.audio_settings.sample_rate();
        self.send_to_project(ProjectServiceInput::SetSampleRate(sample_rate));
    }

    fn show_top(&mut self, ui: &mut eframe::egui::Ui) {
        // self.menu_bar
        //     .set_is_any_track_selected(self.orchestrator_service.is_any_track_selected());
        self.menu_bar.ui(ui);
        let menu_action = self.menu_bar.take_action();
        self.handle_menu_bar_action(menu_action);
        ui.separator();

        let mut control_bar_action = None;
        ui.horizontal_centered(|ui| {
            if let Some(project) = self.project.as_mut() {
                if let Ok(mut project) = project.write() {
                    ui.add(transport(&mut project.transport));
                }
            } else {
                // there might be some flicker here while we wait for the
                // project to first come into existence
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
        match action {
            ControlBarAction::Play => self.send_to_project(ProjectServiceInput::Play),
            ControlBarAction::Stop => self.send_to_project(ProjectServiceInput::Stop),
            ControlBarAction::New => {
                self.handle_project_new();
            }
            ControlBarAction::Open(path) => {
                self.handle_project_load(path);
            }
            ControlBarAction::Save(path) => {
                self.handle_project_save(Some(path));
            }
            ControlBarAction::ToggleSettings => {
                self.rendering_state.is_settings_panel_open =
                    !self.rendering_state.is_settings_panel_open;
            }
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
                    self.send_to_project(ProjectServiceInput::NextTimelineDisplayer);
                }
                TimelineIconStripAction::ShowPianoRoll => {
                    self.rendering_state.is_piano_roll_visible =
                        !self.rendering_state.is_piano_roll_visible;
                }
            }
        }
        let mut action = None;
        if let Some(project) = self.project.as_mut() {
            if let Ok(mut project) = project.write() {
                project.ui_piano_roll(ui, &mut self.rendering_state.is_piano_roll_visible);
                project.ui_detail(
                    ui,
                    self.rendering_state.detail_uid,
                    &self.rendering_state.detail_title,
                    &mut self.rendering_state.is_detail_open,
                );
                let _ = ui.add(project_widget(
                    &mut project,
                    &mut self.rendering_state.view_range,
                    &mut action,
                ));
            }
        }
        if let Some(action) = action {
            self.handle_action(action);
        }

        // If we're performing, then we know the screen is updating, so we
        // should draw it.
        if self.e.is_project_performing {
            ui.ctx().request_repaint();
        }

        self.toasts.show(ui.ctx());
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
                    } => {
                        if !repeat && !modifiers.any() {
                            self.send_to_project(ProjectServiceInput::KeyEvent(*key, *pressed));
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
                self.send_to_project(ProjectServiceInput::TrackAddEntity(
                    track_uid,
                    EntityKey::from(key),
                ));
            }
            ProjectAction::EntitySelected(uid, title) => {
                // This is a view-only thing, so we can add a field in this
                // struct and use it to decide what to display. No need to get
                // Project involved.
                //                self.project.select_detail(uid, name);
                self.rendering_state.detail_uid = Some(uid);
                self.rendering_state.detail_title = title.clone();
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
                    self.handle_project_load(PathBuf::from("ensnare-project.json"));
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
        todo!()
        // self.send_to_project(ProjectServiceInput::New);
    }

    fn handle_project_save(&mut self, path: Option<PathBuf>) {
        self.send_to_project(ProjectServiceInput::Save(path));
    }

    fn handle_project_load(&mut self, path: PathBuf) {
        self.send_to_project(ProjectServiceInput::Load(path));
    }

    fn check_drag_and_drop(&mut self) {
        if let Some((source, target)) = DragDropManager::check_and_clear_drop_event() {
            match source {
                DragSource::NewDevice(ref key) => match target {
                    DropTarget::Controllable(_, _) => todo!(),
                    DropTarget::Track(track_uid) => {
                        self.send_to_project(ProjectServiceInput::TrackAddEntity(
                            track_uid,
                            EntityKey::from(key),
                        ));
                    }
                    DropTarget::TrackPosition(_, _) => {
                        eprintln!("DropTarget::TrackPosition not implemented - ignoring");
                    }
                },
                DragSource::Pattern(pattern_uid) => match target {
                    DropTarget::Controllable(_, _) => todo!(),
                    DropTarget::Track(_) => todo!(),
                    DropTarget::TrackPosition(track_uid, position) => {
                        self.send_to_project(ProjectServiceInput::PatternArrange(
                            track_uid,
                            pattern_uid,
                            position,
                        ));
                    }
                },
                DragSource::ControlSource(source_uid) => match target {
                    DropTarget::Controllable(target_uid, index) => {
                        self.send_to_project(ProjectServiceInput::LinkControl(
                            source_uid, target_uid, index,
                        ));
                    }
                    DropTarget::Track(_) => todo!(),
                    DropTarget::TrackPosition(_, _) => todo!(),
                },
            }
        }
    }

    fn send_to_audio(&self, input: AudioServiceInput) {
        if let Err(e) = self.audio_sender.send(input) {
            eprintln!("Error {e} while sending AudioServiceInput");
        }
    }

    fn send_to_midi(&self, input: MidiInterfaceInput) {
        if let Err(e) = self.midi_sender.send(input) {
            eprintln!("Error {e} while sending MidiInterfaceInput");
        }
    }

    fn send_to_project(&self, input: ProjectServiceInput) {
        if let Err(e) = self.project_sender.send(input) {
            eprintln!("Error {e} while sending ProjectServiceInput");
        }
    }
}
impl App for Ensnare {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        self.handle_app_event_channel(ctx);
        self.handle_input_events(ctx);
        // self.orchestrator_service
        //     .set_control_only_down(self.modifiers.command_only());

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
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.settings.has_been_saved() {
            let _ = self.settings.save();
        }
        self.send_to_audio(AudioServiceInput::Quit);
        self.send_to_midi(MidiInterfaceInput::Quit);
        self.send_to_project(ProjectServiceInput::Quit);
    }
}
