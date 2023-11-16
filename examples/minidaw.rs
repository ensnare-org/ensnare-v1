// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `minidaw` example is a minimal digital audio workstation, using the
//! sound engine and some of the GUI components to create a simple
//! music-production application.

#![deny(rustdoc::broken_intra_doc_links)]

use anyhow::anyhow;
use crossbeam_channel::Select;
use eframe::{
    egui::{
        self, warn_if_debug_build, Button, Context, Direction, FontData, FontDefinitions, Layout,
        ScrollArea, TextStyle,
    },
    emath::{Align, Align2},
    epaint::{FontFamily, FontId},
    CreationContext,
};
use egui_toast::{Toast, ToastOptions, Toasts};
use ensnare::{
    app_version,
    arrangement::ProjectTitle,
    prelude::*,
    ui::widgets::{audio_settings, midi_settings},
};
use ensnare_core::{types::TrackTitle, uid::TrackUid};
use ensnare_orchestration::{egui::entity_palette, DescribesProject};
use ensnare_services::{control_bar_widget, ControlBarAction};
use std::{
    collections::HashMap,
    io::{Read, Write},
    ops::DerefMut,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
struct Settings {
    audio_settings: AudioSettings,
    midi_settings: Arc<Mutex<MidiSettings>>,
}
impl Settings {
    const FILENAME: &'static str = "settings.json";

    fn load() -> anyhow::Result<Self> {
        let settings_path = PathBuf::from(Self::FILENAME);
        let mut contents = String::new();
        let mut file = std::fs::File::open(settings_path.clone())
            .map_err(|e| anyhow::format_err!("Couldn't open {settings_path:?}: {}", e))?;
        file.read_to_string(&mut contents)
            .map_err(|e| anyhow::format_err!("Couldn't read {settings_path:?}: {}", e))?;
        // serde_json::from_str(&contents)
        //     .map_err(|e| anyhow::format_err!("Couldn't parse {settings_path:?}: {}", e))
        Ok(Self::default())
    }

    fn save(&mut self) -> anyhow::Result<()> {
        let settings_path = PathBuf::from(Self::FILENAME);
        // let json = serde_json::to_string_pretty(&self)
        //     .map_err(|_| anyhow::format_err!("Unable to serialize settings JSON"))?;
        let json = "{}";
        if let Some(dir) = settings_path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| {
                anyhow::format_err!(
                    "Unable to create {settings_path:?} parent directories: {}",
                    e
                )
            })?;
        }

        let mut file = std::fs::File::create(settings_path.clone())
            .map_err(|e| anyhow::format_err!("Unable to create {settings_path:?}: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| anyhow::format_err!("Unable to write {settings_path:?}: {}", e))?;

        self.mark_clean();
        Ok(())
    }
}
impl HasSettings for Settings {
    fn has_been_saved(&self) -> bool {
        let has_midi_been_saved = {
            if let Ok(midi) = self.midi_settings.lock() {
                midi.has_been_saved()
            } else {
                true
            }
        };
        self.audio_settings.has_been_saved() || has_midi_been_saved
    }

    fn needs_save(&mut self) {
        panic!("TODO: this struct has no settings of its own, so there shouldn't be a reason to mark it dirty.")
    }

    fn mark_clean(&mut self) {
        self.audio_settings.mark_clean();
        if let Ok(mut midi) = self.midi_settings.lock() {
            midi.mark_clean();
        }
    }
}

// Settings are unique to each app, so this particular one is here in this
// example code rather than part of the crate. As much as possible, we're
// composing it from reusable parts.
#[derive(Debug)]
struct SettingsPanel {
    settings: Settings,
    audio_panel: AudioService,
    midi_panel: MidiService,

    midi_inputs: Vec<MidiPortDescriptor>,
    midi_outputs: Vec<MidiPortDescriptor>,

    is_open: bool,
}
impl SettingsPanel {
    /// Creates a new [SettingsPanel].
    pub fn new_with(settings: Settings, orchestrator: Arc<Mutex<Orchestrator>>) -> Self {
        let midi_panel = MidiService::new_with(Arc::clone(&settings.midi_settings));
        let midi_panel_sender = midi_panel.sender().clone();
        let needs_audio_fn: NeedsAudioFn = {
            Box::new(move |audio_queue, samples_requested| {
                if let Ok(mut o) = orchestrator.lock() {
                    let mut helper = OrchestratorHelper::new_with(o.deref_mut());
                    helper.render_and_enqueue(samples_requested, audio_queue, &mut |event| {
                        if let EntityEvent::Midi(channel, message) = event {
                            let _ =
                                midi_panel_sender.send(MidiInterfaceInput::Midi(channel, message));
                        }
                    });
                }
            })
        };
        Self {
            settings,
            audio_panel: AudioService::new_with(needs_audio_fn),
            midi_panel,
            midi_inputs: Default::default(),
            midi_outputs: Default::default(),
            is_open: Default::default(),
        }
    }

    /// Whether the panel is currently visible.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Toggle visibility.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// The owned [AudioPanel].
    pub fn audio_panel(&self) -> &AudioService {
        &self.audio_panel
    }

    /// The owned [MidiPanel].
    pub fn midi_panel(&self) -> &MidiService {
        &self.midi_panel
    }

    /// Asks the panel to shut down any services associated with contained panels.
    pub fn exit(&self) {
        self.audio_panel.exit();
        self.midi_panel.exit();
    }

    fn handle_midi_port_refresh(&mut self) {
        self.midi_inputs = self.midi_panel.inputs().lock().unwrap().clone();
        self.midi_outputs = self.midi_panel.outputs().lock().unwrap().clone();
    }
}
impl Displays for SettingsPanel {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut new_input = None;
        let mut new_output = None;
        let response = {
            ui.heading("Audio");
            ui.add(audio_settings(&mut self.settings.audio_settings))
        } | {
            ui.heading("MIDI");
            let mut settings = self.settings.midi_settings.lock().unwrap();
            ui.add(midi_settings(
                &mut settings,
                &self.midi_inputs,
                &self.midi_outputs,
                &mut new_input,
                &mut new_output,
            ))
        };

        if let Some(new_input) = &new_input {
            self.midi_panel.select_input(new_input);
        }
        if let Some(new_output) = &new_output {
            self.midi_panel.select_output(new_output);
        }

        #[cfg(debug_assertions)]
        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "🐛 Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }
        response
    }
}

#[derive(Clone, Debug)]
enum MenuBarAction {
    Quit,
    ProjectNew,
    ProjectOpen,
    ProjectSave,
    TrackNewMidi,
    TrackNewAudio,
    TrackNewAux,
    TrackDuplicate,
    TrackDelete,
    TrackRemoveSelectedPatterns,
    TrackAddEntity(EntityKey),
    ComingSoon,
}

#[derive(Debug)]
struct MenuBarItem {
    name: String,
    children: Option<Vec<MenuBarItem>>,
    action: Option<MenuBarAction>,
    enabled: bool,
}
impl MenuBarItem {
    fn node(name: &str, children: Vec<MenuBarItem>) -> Self {
        Self {
            name: name.to_string(),
            children: Some(children),
            action: None,
            enabled: true,
        }
    }
    fn leaf(name: &str, action: MenuBarAction, enabled: bool) -> Self {
        Self {
            name: name.to_string(),
            children: None,
            action: Some(action),
            enabled,
        }
    }
    fn show(&self, ui: &mut eframe::egui::Ui) -> Option<MenuBarAction> {
        let mut action = None;
        if let Some(children) = self.children.as_ref() {
            ui.menu_button(&self.name, |ui| {
                for child in children.iter() {
                    if let Some(a) = child.show(ui) {
                        action = Some(a);
                    }
                }
            });
        } else if let Some(action_to_perform) = &self.action {
            if ui
                .add_enabled(self.enabled, Button::new(&self.name))
                .clicked()
            {
                ui.close_menu();
                action = Some(action_to_perform.clone());
            }
        }
        action
    }
}

#[derive(Debug, Default)]
struct MenuBar {}
impl MenuBar {
    fn show_with_action(
        &mut self,
        ui: &mut eframe::egui::Ui,
        is_track_selected: bool,
    ) -> Option<MenuBarAction> {
        let mut action = None;

        // Menus should look like menus, not buttons
        ui.style_mut().visuals.button_frame = false;

        ui.horizontal(|ui| {
            let mut device_submenus = Vec::default();
            if is_track_selected {
                device_submenus.push(MenuBarItem::node("New", self.new_entity_menu()));
            }
            device_submenus.extend(vec![
                MenuBarItem::leaf("Shift Left", MenuBarAction::ComingSoon, true),
                MenuBarItem::leaf("Shift Right", MenuBarAction::ComingSoon, true),
                MenuBarItem::leaf("Move Up", MenuBarAction::ComingSoon, true),
                MenuBarItem::leaf("Move Down", MenuBarAction::ComingSoon, true),
            ]);
            let menus = vec![
                MenuBarItem::node(
                    "Project",
                    vec![
                        MenuBarItem::leaf("New", MenuBarAction::ProjectNew, true),
                        MenuBarItem::leaf("Open", MenuBarAction::ProjectOpen, true),
                        MenuBarItem::leaf("Save", MenuBarAction::ProjectSave, true),
                        MenuBarItem::leaf("Quit", MenuBarAction::Quit, true),
                    ],
                ),
                MenuBarItem::node(
                    "Track",
                    vec![
                        MenuBarItem::leaf("New MIDI", MenuBarAction::TrackNewMidi, true),
                        MenuBarItem::leaf("New Audio", MenuBarAction::TrackNewAudio, true),
                        MenuBarItem::leaf("New Aux", MenuBarAction::TrackNewAux, true),
                        MenuBarItem::leaf(
                            "Duplicate",
                            MenuBarAction::TrackDuplicate,
                            is_track_selected,
                        ),
                        MenuBarItem::leaf("Delete", MenuBarAction::TrackDelete, is_track_selected),
                        MenuBarItem::leaf(
                            "Remove Selected Patterns",
                            MenuBarAction::TrackRemoveSelectedPatterns,
                            true,
                        ), // TODO: enable only if some patterns selected
                    ],
                ),
                MenuBarItem::node("Device", device_submenus),
                MenuBarItem::node(
                    "Control",
                    vec![
                        MenuBarItem::leaf("Connect", MenuBarAction::ComingSoon, true),
                        MenuBarItem::leaf("Disconnect", MenuBarAction::ComingSoon, true),
                    ],
                ),
            ];
            for item in menus.iter() {
                if let Some(a) = item.show(ui) {
                    action = Some(a);
                }
            }
        });
        action
    }

    fn new_entity_menu(&self) -> Vec<MenuBarItem> {
        vec![MenuBarItem::node(
            "Entities",
            EntityFactory::global()
                .keys()
                .iter()
                .map(|k| {
                    MenuBarItem::leaf(
                        &k.to_string(),
                        MenuBarAction::TrackAddEntity(k.clone()),
                        true,
                    )
                })
                .collect(),
        )]
    }
}

struct MiniDaw {
    orchestrator: Arc<Mutex<Orchestrator>>,
    track_titles: HashMap<TrackUid, TrackTitle>,
    title: ProjectTitle,
    track_frontmost_uids: HashMap<TrackUid, Uid>,

    menu_bar: MenuBar,
    control_bar: ControlBar,
    orchestrator_panel: OrchestratorService,
    settings_panel: SettingsPanel,

    view_range: ViewRange,

    exit_requested: bool,

    toasts: Toasts,
}
impl MiniDaw {
    pub const FONT_REGULAR: &'static str = "font-regular";
    pub const FONT_BOLD: &'static str = "font-bold";
    pub const FONT_MONO: &'static str = "font-mono";
    pub const APP_NAME: &'static str = "MiniDAW";

    pub fn new(cc: &CreationContext) -> Self {
        Self::initialize_fonts(cc);
        Self::initialize_style(&cc.egui_ctx);

        let settings = Settings::load().unwrap_or_default();
        let orchestrator_panel = OrchestratorService::default();
        let orchestrator = Arc::clone(&orchestrator_panel.orchestrator);
        let orchestrator_for_settings_panel = Arc::clone(&orchestrator);
        let mut r = Self {
            orchestrator,
            track_titles: Default::default(),
            title: Default::default(),
            track_frontmost_uids: Default::default(),
            menu_bar: Default::default(),
            control_bar: Default::default(),
            orchestrator_panel,
            settings_panel: SettingsPanel::new_with(settings, orchestrator_for_settings_panel),

            view_range: Default::default(),

            exit_requested: Default::default(),

            toasts: Toasts::new()
                .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(Direction::BottomUp),
        };
        r.spawn_channel_watcher(cc.egui_ctx.clone());
        r
    }

    fn initialize_fonts(cc: &CreationContext) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            Self::FONT_REGULAR.to_owned(),
            FontData::from_static(include_bytes!("../res/fonts/jost/static/Jost-Regular.ttf")),
        );
        fonts.font_data.insert(
            Self::FONT_BOLD.to_owned(),
            FontData::from_static(include_bytes!("../res/fonts/jost/static/Jost-Bold.ttf")),
        );
        fonts.font_data.insert(
            Self::FONT_MONO.to_owned(),
            FontData::from_static(include_bytes!("../res/fonts/cousine/Cousine-Regular.ttf")),
        );
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

        cc.egui_ctx.set_fonts(fonts);
    }

    fn initialize_style(ctx: &Context) {
        let mut style = (*ctx.style()).clone();

        // Disabled this because it stops text from getting highlighted when the
        // user hovers over a widget.
        //
        // style.visuals.override_text_color = Some(Color32::LIGHT_GRAY);

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

    fn handle_message_channels(&mut self) {
        // As long as any channel had a message in it, we'll keep handling them.
        // We don't expect a giant number of messages; otherwise we'd worry
        // about blocking the UI.
        loop {
            if !(self.handle_midi_panel_channel()
                || self.handle_audio_panel_channel()
                || self.handle_orchestrator_channel())
            {
                break;
            }
        }
    }

    fn handle_midi_panel_channel(&mut self) -> bool {
        if let Ok(m) = self.settings_panel.midi_panel().receiver().try_recv() {
            match m {
                MidiPanelEvent::Midi(channel, message) => {
                    self.orchestrator_panel
                        .send_to_service(OrchestratorInput::Midi(channel, message));
                }
                MidiPanelEvent::MidiOut => {
                    // a chance to indicate that outgoing MIDI data happened
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
            true
        } else {
            false
        }
    }

    fn handle_audio_panel_channel(&mut self) -> bool {
        if let Ok(m) = self.settings_panel.audio_panel().receiver().try_recv() {
            match m {
                AudioPanelEvent::InterfaceChanged => {
                    self.update_orchestrator_audio_interface_config();
                }
            }
            true
        } else {
            false
        }
    }

    fn handle_orchestrator_channel(&mut self) -> bool {
        if let Ok(m) = self.orchestrator_panel.receiver().try_recv() {
            match m {
                OrchestratorEvent::Tempo(_tempo) => {
                    // This is (usually) an acknowledgement that Orchestrator
                    // got our request to change, so we don't need to do
                    // anything.
                }
                OrchestratorEvent::Quit => {
                    eprintln!("OrchestratorEvent::Quit")
                }
                OrchestratorEvent::Loaded(path, _) => {
                    // TODO - it's unclear whether this event should still know
                    // about the project title, since it now belongs to Project
                    // rather than Orchestrator.
                    self.toasts.add(Toast {
                        kind: egui_toast::ToastKind::Success,
                        text: format!(
                            "Loaded {} from {}",
                            <ProjectTitle as Into<String>>::into(self.title.clone()),
                            path.display()
                        )
                        .into(),
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
            }
            true
        } else {
            false
        }
    }

    // Watches certain channels and asks for a repaint, which triggers the
    // actual channel receiver logic, when any of them has something receivable.
    //
    // https://docs.rs/crossbeam-channel/latest/crossbeam_channel/struct.Select.html#method.ready
    //
    // We call ready() rather than select() because select() requires us to
    // complete the operation that is ready, while ready() just tells us that a
    // recv() would not block.
    fn spawn_channel_watcher(&mut self, ctx: Context) {
        let r1 = self.settings_panel.midi_panel().receiver().clone();
        let r2 = self.settings_panel.audio_panel().receiver().clone();
        let r3 = self.orchestrator_panel.receiver().clone();
        let _ = std::thread::spawn(move || {
            let mut sel = Select::new();
            let _ = sel.recv(&r1);
            let _ = sel.recv(&r2);
            let _ = sel.recv(&r3);
            loop {
                let _ = sel.ready();
                ctx.request_repaint();
            }
        });
    }

    fn update_orchestrator_audio_interface_config(&mut self) {
        let sample_rate = self.settings_panel.audio_panel().sample_rate();
        if let Ok(mut o) = self.orchestrator.lock() {
            o.update_sample_rate(sample_rate);
        }
    }

    fn handle_control_panel_action(&mut self, action: ControlBarAction) {
        let input = match action {
            ControlBarAction::Play => Some(OrchestratorInput::ProjectPlay),
            ControlBarAction::Stop => Some(OrchestratorInput::ProjectStop),
            ControlBarAction::New => Some(OrchestratorInput::ProjectNew),
            ControlBarAction::Open(path) => Some(OrchestratorInput::ProjectOpen(path)),
            ControlBarAction::Save(path) => Some(OrchestratorInput::ProjectSave(path)),
            ControlBarAction::ToggleSettings => {
                self.settings_panel.toggle();
                None
            }
        };
        if let Some(input) = input {
            self.orchestrator_panel.send_to_service(input);
        }
    }

    fn handle_menu_bar_action(&mut self, action: MenuBarAction) {
        let mut input = None;
        match action {
            MenuBarAction::Quit => self.exit_requested = true,
            MenuBarAction::TrackNewAudio => input = Some(OrchestratorInput::TrackNewAudio),
            MenuBarAction::TrackNewAux => input = Some(OrchestratorInput::TrackNewAux),
            MenuBarAction::TrackNewMidi => input = Some(OrchestratorInput::TrackNewMidi),
            MenuBarAction::TrackDelete => input = Some(OrchestratorInput::TrackDeleteSelected),
            MenuBarAction::TrackDuplicate => {
                input = Some(OrchestratorInput::TrackDuplicateSelected)
            }
            MenuBarAction::TrackRemoveSelectedPatterns => {
                input = Some(OrchestratorInput::TrackPatternRemoveSelected)
            }
            MenuBarAction::ComingSoon => {
                self.toasts.add(Toast {
                    kind: egui_toast::ToastKind::Info,
                    text: "Coming soon!".into(),
                    options: ToastOptions::default(),
                });
            }
            MenuBarAction::ProjectNew => input = Some(OrchestratorInput::ProjectNew),
            MenuBarAction::ProjectOpen => {
                input = Some(OrchestratorInput::ProjectOpen(PathBuf::from(
                    "minidaw.json",
                )))
            }
            MenuBarAction::ProjectSave => {
                input = Some(OrchestratorInput::ProjectSave(PathBuf::from(
                    "minidaw.json",
                )))
            }
            MenuBarAction::TrackAddEntity(_key) => {
                //                input = Some(OrchestratorInput::TrackAddEntity(key))
            }
        }
        if let Some(input) = input {
            self.orchestrator_panel.send_to_service(input);
        }
    }

    fn show_top(&mut self, ui: &mut eframe::egui::Ui) {
        if let Some(action) = self
            .menu_bar
            .show_with_action(ui, self.orchestrator_panel.is_any_track_selected())
        {
            self.handle_menu_bar_action(action);
        }
        ui.separator();
        let mut control_bar_action = None;
        ui.add(control_bar_widget(
            &mut self.control_bar,
            &mut control_bar_action,
        ));

        if let Some(action) = control_bar_action {
            self.handle_control_panel_action(action);
        }
    }

    fn show_bottom(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            warn_if_debug_build(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(app_version())
            });
        });
    }

    fn show_left(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::horizontal().show(ui, |ui| {
            ui.add(entity_palette(EntityFactory::global().sorted_keys()))
        });
    }

    fn show_right(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::horizontal().show(ui, |ui| ui.label("Under Construction"));
    }

    fn show_center(&mut self, ui: &mut eframe::egui::Ui, is_control_only_down: bool) {
        ScrollArea::vertical().show(ui, |ui| {
            self.orchestrator_panel
                .set_control_only_down(is_control_only_down);
            if let Ok(mut o) = self.orchestrator.lock() {
                #[derive(Debug)]
                struct ProjectDescriber<'a> {
                    track_titles: &'a HashMap<TrackUid, TrackTitle>,
                    track_frontmost_uids: &'a HashMap<TrackUid, Uid>,
                }
                impl<'a> DescribesProject for ProjectDescriber<'a> {
                    fn track_title(&self, track_uid: &TrackUid) -> Option<&TrackTitle> {
                        self.track_titles.get(track_uid)
                    }

                    fn track_frontmost_timeline_displayer(
                        &self,
                        track_uid: &TrackUid,
                    ) -> Option<Uid> {
                        if let Some(uid) = self.track_frontmost_uids.get(track_uid) {
                            Some(*uid)
                        } else {
                            None
                        }
                    }
                }
                let project_describer = ProjectDescriber {
                    track_titles: &self.track_titles,
                    track_frontmost_uids: &self.track_frontmost_uids,
                };
                let mut view_range = self.view_range.clone();
                let mut action = None;
                let _ = ui.add(project_widget(
                    &project_describer,
                    o.deref_mut(),
                    &mut view_range,
                    &mut action,
                ));
                self.view_range = view_range;
                if let Some(action) = action {
                    todo!("deal with this! {action:?}");
                }
            }
        });
    }

    fn update_window_title(&mut self, frame: &mut eframe::Frame) {
        // TODO: it seems like the window remembers its title, so this isn't
        // something we should be doing on every frame.
        let full_title = format!(
            "{} - {}",
            Self::APP_NAME,
            <ProjectTitle as Into<String>>::into(self.title.clone())
        );
        frame.set_window_title(&full_title);
    }

    fn show_settings_panel(&mut self, ctx: &Context) {
        let mut is_settings_open = self.settings_panel.is_open();
        egui::Window::new("Settings")
            .open(&mut is_settings_open)
            .show(ctx, |ui| self.settings_panel.ui(ui));
        if self.settings_panel.is_open() && !is_settings_open {
            self.settings_panel.toggle();
        }
    }
}
impl eframe::App for MiniDaw {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.handle_message_channels();
        self.update_window_title(frame);

        let mut is_control_only_down = false;
        ctx.input(|i| {
            if i.modifiers.command_only() {
                is_control_only_down = true;
            }
        });

        let top = egui::TopBottomPanel::top("top-panel")
            .resizable(false)
            .exact_height(64.0);
        let bottom = egui::TopBottomPanel::bottom("bottom-panel")
            .resizable(false)
            .exact_height(24.0);
        let left = egui::SidePanel::left("left-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0);
        let right = egui::SidePanel::right("right-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0);
        let center = egui::CentralPanel::default();

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
            self.show_center(ui, is_control_only_down);
            self.toasts.show(ctx);
        });

        self.show_settings_panel(ctx);

        if self.exit_requested {
            frame.close();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.settings_panel.settings.has_been_saved() {
            let _ = self.settings_panel.settings.save();
        }
        self.settings_panel.exit();
        self.orchestrator_panel.exit();
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1366.0, 768.0)),
        ..Default::default()
    };

    let mut factory = EntityFactory::default();
    register_factory_entities(&mut factory);
    if EntityFactory::initialize(factory).is_err() {
        return Err(anyhow!("Couldn't set EntityFactory once_cell"));
    }
    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        return Err(anyhow!("Couldn't set DragDropManager once_cell"));
    }

    if let Err(e) = eframe::run_native(
        MiniDaw::APP_NAME,
        options,
        Box::new(|cc| Box::new(MiniDaw::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
