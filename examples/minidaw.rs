// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `minidaw` example is a minimal digital audio workstation, using the
//! sound engine and some of the GUI components to create a simple
//! music-production application.

#![deny(rustdoc::broken_intra_doc_links)]

use anyhow::anyhow;
use crossbeam_channel::{Select, Sender};
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
use ensnare::{app_version, prelude::*};
use ensnare_core::{types::TrackTitle, uid::TrackUid};
use ensnare_entities::BuiltInEntities;
use ensnare_entity::traits::EntityBounds;
use ensnare_new_stuff::{
    egui::ProjectWidget,
    project::{Project, ProjectTitle},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Settings {
    audio_settings: AudioSettings,
    midi_settings: Arc<RwLock<MidiSettings>>,

    #[serde(skip)]
    midi_sender: Option<Sender<MidiInterfaceServiceInput>>,

    // Cached options for fast menu drawing.
    #[serde(skip)]
    midi_inputs: Vec<MidiPortDescriptor>,
    #[serde(skip)]
    midi_outputs: Vec<MidiPortDescriptor>,
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

    fn handle_midi_input_port_refresh(&mut self, ports: &[MidiPortDescriptor]) {
        self.midi_inputs = ports.to_vec();
    }

    fn handle_midi_output_port_refresh(&mut self, ports: &[MidiPortDescriptor]) {
        self.midi_outputs = ports.to_vec();
    }
}
impl HasSettings for Settings {
    fn has_been_saved(&self) -> bool {
        let has_midi_been_saved = {
            if let Ok(midi) = self.midi_settings.read() {
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
        if let Ok(mut midi) = self.midi_settings.write() {
            midi.mark_clean();
        }
    }
}
impl Displays for Settings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut new_input = None;
        let mut new_output = None;
        let response = {
            ui.heading("Audio");
            ui.add(audio_settings(&mut self.audio_settings))
        } | {
            ui.heading("MIDI");
            let mut settings = self.midi_settings.write().unwrap();
            ui.add(midi_settings(
                &mut settings,
                &self.midi_inputs,
                &self.midi_outputs,
                &mut new_input,
                &mut new_output,
            ))
        };

        if let Some(sender) = &self.midi_sender {
            if let Some(new_input) = &new_input {
                let _ = sender.send(MidiInterfaceServiceInput::SelectMidiInput(
                    new_input.clone(),
                ));
            }
            if let Some(new_output) = &new_output {
                let _ = sender.send(MidiInterfaceServiceInput::SelectMidiOutput(
                    new_output.clone(),
                ));
            }
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

// Settings are unique to each app, so this particular one is here in this
// example code rather than part of the crate. As much as possible, we're
// composing it from reusable parts.
#[derive(Debug)]
struct SettingsPanel {
    settings: Settings,
    audio_service: AudioService,
    midi_service: MidiService,

    midi_inputs: Vec<MidiPortDescriptor>,
    midi_outputs: Vec<MidiPortDescriptor>,

    is_open: bool,
}
impl SettingsPanel {
    /// Creates a new [SettingsPanel].
    pub fn new_with(settings: Settings) -> Self {
        let midi_service = MidiService::new_with(&settings.midi_settings);
        let _midi_service_sender = midi_service.sender().clone();
        Self {
            settings,
            audio_service: AudioService::new(),
            midi_service,
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
        &self.audio_service
    }

    /// The owned [MidiPanel].
    pub fn midi_panel(&self) -> &MidiService {
        &self.midi_service
    }

    /// Asks the panel to shut down any services associated with contained panels.
    pub fn exit(&self) {
        let _ = self.audio_service.sender().send(AudioServiceInput::Quit);
        self.midi_service.exit();
    }

    fn handle_midi_port_refresh(&mut self) {
        self.midi_inputs = self.midi_service.inputs().lock().unwrap().clone();
        self.midi_outputs = self.midi_service.outputs().lock().unwrap().clone();
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

#[derive(Debug)]
struct MenuBar {
    factory: Arc<EntityFactory<dyn EntityBounds>>,
}
impl MenuBar {
    fn new(factory: &Arc<EntityFactory<dyn EntityBounds>>) -> Self {
        Self {
            factory: Arc::clone(&factory),
        }
    }
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
            self.factory
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
    factory: Arc<EntityFactory<dyn EntityBounds>>,
    audio_service: AudioService,
    midi_service: MidiService,
    project_service: ProjectService,
    project: Option<Arc<RwLock<Project>>>,
    track_titles: HashMap<TrackUid, TrackTitle>,
    title: Option<ProjectTitle>,
    track_frontmost_uids: HashMap<TrackUid, Uid>,

    menu_bar: MenuBar,
    control_bar: ControlBar,
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

    pub fn new(cc: &CreationContext, factory: EntityFactory<dyn EntityBounds>) -> Self {
        Self::initialize_fonts(cc);
        Self::initialize_style(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let settings = Settings::load().unwrap_or_default();
        let factory = Arc::new(factory);
        let audio_service = AudioService::new();
        let midi_service = MidiService::new_with(&settings.midi_settings);
        let project_service = ProjectService::new_with(&factory);
        let mut r = Self {
            factory: Arc::clone(&factory),
            audio_service,
            midi_service,
            project_service,
            project: Default::default(),
            track_titles: Default::default(),
            title: Default::default(),
            track_frontmost_uids: Default::default(),
            menu_bar: MenuBar::new(&factory),
            control_bar: Default::default(),
            settings_panel: SettingsPanel::new_with(settings),

            view_range: ViewRange(MusicalTime::START..(MusicalTime::DURATION_WHOLE * 4)),

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
        while self.handle_midi_panel_channel()
            || self.handle_audio_panel_channel()
            || self.handle_project_channel()
        {}
    }

    fn handle_midi_panel_channel(&mut self) -> bool {
        if let Ok(m) = self.settings_panel.midi_panel().receiver().try_recv() {
            match m {
                MidiServiceEvent::Midi(channel, message) => {
                    self.send_to_project(ProjectServiceInput::Midi(channel, message));
                }
                MidiServiceEvent::MidiOut => {
                    // a chance to indicate that outgoing MIDI data happened
                }
                MidiServiceEvent::SelectInput(_) => {
                    // TODO: save selection in prefs
                }
                MidiServiceEvent::SelectOutput(_) => {
                    // TODO: save selection in prefs
                }
                MidiServiceEvent::InputPortsRefreshed(ports) => {
                    // TODO: remap any saved preferences to ports that we've found
                    self.settings_panel
                        .settings
                        .handle_midi_input_port_refresh(&ports);
                }
                MidiServiceEvent::OutputPortsRefreshed(ports) => {
                    // TODO: remap any saved preferences to ports that we've found
                    self.settings_panel
                        .settings
                        .handle_midi_output_port_refresh(&ports);
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
                AudioServiceEvent::Reset(_sample_rate, _channel_count, ref queue) => {
                    let _ = self
                        .project_service
                        .sender()
                        .send(ProjectServiceInput::AudioQueue(Arc::clone(queue)));
                }

                AudioServiceEvent::NeedsAudio(count) => {
                    let _ = self
                        .project_service
                        .sender()
                        .send(ProjectServiceInput::NeedsAudio(count));
                }
                AudioServiceEvent::Underrun => eprintln!("AudioServiceEvent::Underrun"),
            }
            true
        } else {
            false
        }
    }

    fn handle_project_channel(&mut self) -> bool {
        if let Ok(m) = self.project_service.receiver().try_recv() {
            match m {
                ProjectServiceEvent::Loaded(_) => todo!(),
                ProjectServiceEvent::LoadFailed(_, _) => todo!(),
                ProjectServiceEvent::Saved(_) => todo!(),
                ProjectServiceEvent::SaveFailed(_) => todo!(),
                ProjectServiceEvent::TitleChanged(_) => todo!(),
                ProjectServiceEvent::IsPerformingChanged(_) => todo!(),
                ProjectServiceEvent::Quit => {
                    // Good to know, but no need to do anything.
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
        let r1 = self.midi_service.receiver().clone();
        let r2 = self.audio_service.receiver().clone();
        let r3 = self.project_service.receiver().clone();
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

    fn handle_control_panel_action(&mut self, action: ControlBarAction) {
        let input = match action {
            ControlBarAction::Play => Some(ProjectServiceInput::ProjectPlay),
            ControlBarAction::Stop => Some(ProjectServiceInput::ProjectStop),
            ControlBarAction::New => Some(ProjectServiceInput::ProjectNew),
            ControlBarAction::Open(path) => Some(ProjectServiceInput::ProjectLoad(path)),
            ControlBarAction::Save(path) => Some(ProjectServiceInput::ProjectSave(Some(path))),
            ControlBarAction::ToggleSettings => {
                self.settings_panel.toggle();
                None
            }
        };
        if let Some(input) = input {
            self.send_to_project(input);
        }
    }

    fn handle_menu_bar_action(&mut self, action: MenuBarAction) {
        let mut input = None;
        let mut coming_soon = false;
        match action {
            MenuBarAction::Quit => self.exit_requested = true,
            MenuBarAction::TrackNewAudio => coming_soon = true,
            MenuBarAction::TrackNewAux => coming_soon = true,
            MenuBarAction::TrackNewMidi => coming_soon = true,
            MenuBarAction::TrackDelete => coming_soon = true,
            MenuBarAction::TrackDuplicate => coming_soon = true,
            MenuBarAction::TrackRemoveSelectedPatterns => coming_soon = true,
            MenuBarAction::ComingSoon => coming_soon = true,
            MenuBarAction::ProjectNew => input = Some(ProjectServiceInput::ProjectNew),
            MenuBarAction::ProjectOpen => {
                input = Some(ProjectServiceInput::ProjectLoad(PathBuf::from(
                    "minidaw.json",
                )))
            }
            MenuBarAction::ProjectSave => {
                input = Some(ProjectServiceInput::ProjectSave(Some(PathBuf::from(
                    "minidaw.json",
                ))))
            }
            MenuBarAction::TrackAddEntity(_key) => coming_soon = true,
        }
        if coming_soon {
            self.toasts.add(Toast {
                kind: egui_toast::ToastKind::Info,
                text: "Coming soon!".into(),
                options: ToastOptions::default(),
            });
        }
        if let Some(input) = input {
            self.send_to_project(input);
        }
    }

    fn show_top(&mut self, ui: &mut eframe::egui::Ui) {
        if let Some(action) = self.menu_bar.show_with_action(ui, false) {
            self.handle_menu_bar_action(action);
        }
        ui.separator();
        let mut control_bar_action = None;
        ui.add(ControlBarWidget::widget(
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
        ScrollArea::horizontal().show(ui, |ui| ui.add(entity_palette(self.factory.sorted_keys())));
    }

    fn show_right(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::horizontal().show(ui, |ui| ui.label("Under Construction"));
    }

    fn show_center(&mut self, ui: &mut eframe::egui::Ui, _is_control_only_down: bool) {
        ScrollArea::vertical().show(ui, |ui| {
            // self.orchestrator_panel
            //     .set_control_only_down(is_control_only_down);
            if let Some(project) = self.project.as_ref() {
                if let Ok(mut project) = project.write() {
                    let mut action = None;
                    let _ = ui.add(ProjectWidget::widget(&mut project, &mut action));
                    if let Some(action) = action {
                        todo!("deal with this! {action:?}");
                    }
                }
            }
        });
    }

    fn show_settings_panel(&mut self, ctx: &Context) {
        let mut is_settings_open = self.settings_panel.is_open();
        egui::Window::new("Settings")
            .open(&mut is_settings_open)
            .show(ctx, |ui| self.settings_panel.settings.ui(ui));
        if self.settings_panel.is_open() && !is_settings_open {
            self.settings_panel.toggle();
        }
    }

    fn send_to_project(&self, input: ProjectServiceInput) {
        let _ = self.project_service.sender().send(input);
    }

    fn send_to_audio(&self, input: AudioServiceInput) {
        let _ = self.audio_service.sender().send(input);
    }

    fn send_to_midi(&self, input: MidiServiceInput) {
        let _ = self.midi_service.input_channels.sender.send(input);
    }
}
impl eframe::App for MiniDaw {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.handle_message_channels();

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
            // 0.24            frame.close();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.settings_panel.settings.has_been_saved() {
            let _ = self.settings_panel.settings.save();
        }
        self.send_to_audio(AudioServiceInput::Quit);
        self.send_to_midi(MidiServiceInput::Quit);
        self.send_to_project(ProjectServiceInput::ServiceQuit);
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("MiniDAW")
            .with_inner_size(eframe::epaint::vec2(1024.0, 768.0))
            .to_owned(),
        ..Default::default()
    };

    let factory = BuiltInEntities::register(EntityFactory::default()).finalize();
    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        return Err(anyhow!("Couldn't set DragDropManager once_cell"));
    }

    if let Err(e) = eframe::run_native(
        MiniDaw::APP_NAME,
        options,
        Box::new(|cc| Box::new(MiniDaw::new(cc, factory))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
