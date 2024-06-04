// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `widget-explorer` example is a sandbox for developing egui Ensnare
//! widgets.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use derivative::Derivative;
use eframe::{
    egui::{
        self, warn_if_debug_build, CollapsingHeader, DragValue, Frame, Id, Layout, ScrollArea,
        Slider, Style, Ui, ViewportBuilder, Widget,
    },
    emath::Align,
    epaint::{vec2, Galley},
    CreationContext,
};
use ensnare_toys::prelude::*;
use ensnare_v1::{
    app_version,
    composition::{NoteSequencer, NoteSequencerBuilder},
    egui::{
        widget_explorer::{
            analyze_spectrum, make_title_bar_galley, FrequencyDomainWidget, GridWidget,
            LegendWidget, NoteSequencerWidget, SignalPathWidget, TimeDomainWidget, TitleBarWidget,
            Wiggler,
        },
        ComposerWidget,
    },
    orchestration::TrackTitle,
    prelude::*,
    types::VisualizationQueue,
    util::Rng,
};
use std::sync::Arc;

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct LegendSettings {
    hide: bool,
    #[derivative(Default(
        value = "(MusicalTime::START..MusicalTime::new_with_beats(128)).into()"
    ))]
    range: ViewRange,
}
impl LegendSettings {
    const NAME: &'static str = "Legend";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            ui.add(LegendWidget::widget(&mut self.range));
        }
    }
}
impl Displays for LegendSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide");
        ui.label("View range");
        let mut range_start = self.range.0.start.total_beats();
        let mut range_end = self.range.0.end.total_beats();
        let start_response = ui.add(Slider::new(&mut range_start, 0..=128));
        if start_response.changed() {
            self.range.0.start = MusicalTime::new_with_beats(range_start);
        };
        let end_response = ui.add(Slider::new(&mut range_end, 1..=256));
        if end_response.changed() {
            self.range.0.end = MusicalTime::new_with_beats(range_end);
        };
        start_response | end_response
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct SignalPathSettings {
    hide: bool,
    signal_path: SignalPath,
    #[derivative(Default(
        value = "(MusicalTime::START..MusicalTime::new_with_beats(128)).into()"
    ))]
    range: ViewRange,
}
impl SignalPathSettings {
    const NAME: &'static str = "Signal Path";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            Frame::default().show(ui, |ui| {
                let targets = Vec::default();
                ui.set_max_height(256.0);
                let mut action = None;
                ui.add(SignalPathWidget::widget(
                    &mut self.signal_path,
                    &targets,
                    self.range.clone(),
                    &mut action,
                ))
            });
        }
    }
}
impl Displays for SignalPathSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide");
        ui.label("View range");
        let mut range_start = self.range.0.start.total_beats();
        let mut range_end = self.range.0.end.total_beats();
        let start_response = ui.add(Slider::new(&mut range_start, 0..=128));
        if start_response.changed() {
            self.range.0.start = MusicalTime::new_with_beats(range_start);
        };
        let end_response = ui.add(Slider::new(&mut range_end, 1..=256));
        if end_response.changed() {
            self.range.0.end = MusicalTime::new_with_beats(range_end);
        };
        start_response | end_response
    }
}

#[cfg(obsolete)]
mod obsolete {
    #[derive(Debug, Derivative)]
    #[derivative(Default)]
    struct TrackSettings {
        hide: bool,
        track: Track,
        range: TimeRange,
        view_range: ViewRange,
    }
    impl TrackSettings {
        const NAME: &'static str = "Track";

        fn show(&mut self, ui: &mut eframe::egui::Ui) {
            if !self.hide {
                let mut action = None;
                ui.add(track_widget(
                    TrackUid(1),
                    &mut self.track,
                    false,
                    Some(MusicalTime::ONE_BEAT),
                    &self.view_range,
                    &mut action,
                ));
            }
        }

        fn set_view_range(&mut self, view_range: &ViewRange) {
            self.view_range = view_range.clone();
        }
    }
    impl Default for TrackSettings {
        fn default() -> Self {
            let mut r = Self {
                hide: Default::default(),
                track: Track::default(),
                range: TimeRange(MusicalTime::START..MusicalTime::new_with_beats(128)),
                view_range: ViewRange(MusicalTime::START..MusicalTime::new_with_beats(128)),
            };
            let _ = r
                .track
                .append_entity(Box::<NoteSequencer>::default(), Uid(345));
            r
        }
    }
    impl Displays for TrackSettings {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.checkbox(&mut self.hide, "Hide");
            ui.label("Range");
            let mut range_start = self.range.0.start.total_beats();
            let mut range_end = self.range.0.end.total_beats();
            let start_response = ui.add(Slider::new(&mut range_start, 0..=1024));
            if start_response.changed() {
                self.range.0.start = MusicalTime::new_with_beats(range_start);
            };
            let end_response = ui.add(Slider::new(&mut range_end, 0..=1024));
            if end_response.changed() {
                self.range.0.end = MusicalTime::new_with_beats(range_end);
            };
            start_response | end_response
        }
    }
}
/// Wraps a PretendDevicePalette as a [Widget](eframe::egui::Widget).
fn pretend_device_palette(keys: &[EntityKey]) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| PretendDevicePalette::new(keys).ui(ui)
}

#[derive(Debug)]
struct PretendDevicePalette {
    keys: Vec<EntityKey>,
}
impl PretendDevicePalette {
    fn new(keys: &[EntityKey]) -> Self {
        Self {
            keys: Vec::from(keys),
        }
    }
}
impl Widget for PretendDevicePalette {
    fn ui(self, ui: &mut egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), 32.0);
        ui.allocate_ui(desired_size, |ui| {
            ScrollArea::horizontal()
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        for key in self.keys.iter() {
                            ui.dnd_drag_source(Id::new(key), key.clone(), |ui| {
                                ui.label(key.to_string())
                            });
                        }
                    })
                    .response
                })
                .inner
        })
        .response
    }
}

#[derive(Debug)]
struct DevicePaletteSettings {
    hide: bool,
    keys: Vec<EntityKey>,
}
impl DevicePaletteSettings {
    const NAME: &'static str = "Device Palette";

    fn new(factory: EntityFactory<dyn EntityBounds>) -> Self {
        Self {
            hide: Default::default(),
            keys: factory.sorted_keys().to_vec(),
        }
    }

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            ui.add(pretend_device_palette(&self.keys));
        }
    }
}
impl Displays for DevicePaletteSettings {
    fn ui(&mut self, ui: &mut egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}

#[cfg(obsolete)]
mod obsolete {
    #[derive(Debug, Default)]
    struct SignalChainSettings {
        hide: bool,
        is_large_size: bool,
        track: Track,
    }
    impl SignalChainSettings {
        const NAME: &'static str = "Signal Chain";

        fn show(&mut self, ui: &mut eframe::egui::Ui) {
            if !self.hide {
                ui.scope(|ui| {
                    // TODO: who should own this value?
                    ui.set_max_height(32.0);
                    let mut action = None;
                    ui.add(signal_chain(
                        TrackUid::default(),
                        &mut self.track,
                        &mut action,
                    ));
                    if action.is_some() {
                        todo!();
                    }
                });
            }
        }
    }
    impl Displays for SignalChainSettings {
        fn ui(&mut self, ui: &mut egui::Ui) -> eframe::egui::Response {
            ui.checkbox(&mut self.hide, "Hide") | ui.checkbox(&mut self.is_large_size, "Large size")
        }
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct GridSettings {
    hide: bool,
    #[derivative(Default(
        value = "(MusicalTime::START..MusicalTime::new_with_beats(128)).into()"
    ))]
    range: ViewRange,
    #[derivative(Default(
        value = "(MusicalTime::START..MusicalTime::new_with_beats(128)).into()"
    ))]
    view_range: ViewRange,
}
impl GridSettings {
    const NAME: &'static str = "Grid";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            ui.add_enabled(
                false,
                GridWidget::widget(self.range.clone(), self.view_range.clone()),
            );
        }
    }

    fn set_view_range(&mut self, view_range: &ViewRange) {
        self.view_range = view_range.clone();
    }
}
impl Displays for GridSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct NoteSequencerSettings {
    hide: bool,
    #[derivative(Default(value = "Self::make_default_sequencer()"))]
    sequencer: NoteSequencer,
    view_range: ViewRange,
}
impl Displays for NoteSequencerSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl NoteSequencerSettings {
    const NAME: &'static str = "Note Sequencer";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            ui.add(NoteSequencerWidget::widget(
                &mut self.sequencer,
                &self.view_range,
            ));
        }
    }

    fn set_view_range(&mut self, view_range: &ViewRange) {
        self.view_range = view_range.clone();
    }

    fn make_default_sequencer() -> NoteSequencer {
        let mut rng = Rng::default();
        NoteSequencerBuilder::default()
            .random(
                &mut rng,
                TimeRange(MusicalTime::START..MusicalTime::new_with_beats(128)),
            )
            .build()
            .unwrap()
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct ToySynthSettings {
    hide: bool,
    #[derivative(Default(value = "ToySynth::new_with(
        Uid::default(),
        OscillatorBuilder::default().build().unwrap(),
        EnvelopeBuilder::safe_default().build().unwrap(),
        Dca::default(),
    )"))]
    toy_synth: ToySynth,
}
impl Displays for ToySynthSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ToySynthSettings {
    const NAME: &'static str = "Toy Synth";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            self.toy_synth.ui(ui);
        }
    }
}

#[derive(Debug, Default)]
struct ToyControllerSettings {
    hide: bool,
    toy: ToyController,
}
impl Displays for ToyControllerSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ToyControllerSettings {
    const NAME: &'static str = "Toy Controller";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            self.toy.ui(ui);
        }
    }
}

#[derive(Debug, Default)]
struct ToyEffectSettings {
    hide: bool,
    toy: ToyEffect,
}
impl Displays for ToyEffectSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ToyEffectSettings {
    const NAME: &'static str = "Toy Effect";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            self.toy.ui(ui);
        }
    }
}

#[derive(Debug, Default)]
struct ToyInstrumentSettings {
    hide: bool,
    toy: ToyInstrument,
}
impl Displays for ToyInstrumentSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ToyInstrumentSettings {
    const NAME: &'static str = "Toy Instrument";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            self.toy.ui(ui);
        }
    }
}

#[derive(Debug, Default)]
struct TitleBarSettings {
    hide: bool,
    title: TrackTitle,
    font_galley: Option<Arc<Galley>>,
}

impl Displays for TitleBarSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if self.font_galley.is_none() {
            self.font_galley = Some(make_title_bar_galley(ui, &self.title));
        }
        ui.checkbox(&mut self.hide, "Hide");
        let response = ui.text_edit_singleline(&mut self.title.0);
        if response.changed() {
            self.font_galley = None;
        }
        response
    }
}
impl TitleBarSettings {
    const NAME: &'static str = "Title Bar";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            if let Some(font_galley) = &self.font_galley {
                let mut action = None;
                ui.add(TitleBarWidget::widget(
                    Some(Arc::clone(font_galley)),
                    &mut action,
                ));
            }
        }
    }
}

#[derive(Debug)]
struct ComposerSettings {
    hide: bool,
    notes: Vec<Note>,
}
impl Default for ComposerSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            notes: Default::default(),
        }
    }
}
impl Displays for ComposerSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ComposerSettings {
    const NAME: &'static str = "Composer";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            ui.add(ComposerWidget::widget(&mut self.notes));
        }
    }
}

#[derive(Debug, Default)]
struct WigglerSettings {
    hide: bool,
}

impl Displays for WigglerSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl WigglerSettings {
    const NAME: &'static str = "Wiggler";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.hide {
            ui.add(Wiggler::widget());
        }
    }
}

trait RandomizesBuffer {
    fn visualization_queue(&self) -> &VisualizationQueue;
    fn rng(&mut self) -> &mut Rng;

    fn add_noise_to_buffer(&mut self) {
        let queue = Arc::clone(&self.visualization_queue().0);
        for _ in 0..8 {
            queue.write().unwrap().push_back(Sample::from(Normal::from(
                self.rng().rand_u64() as f64 / u64::MAX as f64,
            )));
        }
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct TimeDomainSettings {
    hide: bool,
    #[derivative(Default(value = "128.0"))]
    max_width: f32,
    #[derivative(Default(value = "64.0"))]
    max_height: f32,
    visualization_queue: VisualizationQueue,
    rng: Rng,
}

impl Displays for TimeDomainSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
            | ui.add(DragValue::new(&mut self.max_width).prefix("width: "))
            | ui.add(DragValue::new(&mut self.max_height).prefix("height: "))
    }
}
impl RandomizesBuffer for TimeDomainSettings {
    fn visualization_queue(&self) -> &VisualizationQueue {
        &self.visualization_queue
    }

    fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}
impl TimeDomainSettings {
    const NAME: &'static str = "Audio Time Domain";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        self.add_noise_to_buffer();
        if !self.hide {
            ui.scope(|ui| {
                ui.set_max_width(self.max_width);
                ui.set_max_height(self.max_height);
                if let Ok(queue) = self.visualization_queue.0.read() {
                    let (slice_1, slice_2) = queue.as_slices();
                    ui.add(TimeDomainWidget::widget(slice_1, slice_2));
                }
            });
        }
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct FrequencyDomainSettings {
    hide: bool,
    #[derivative(Default(value = "128.0"))]
    max_width: f32,
    #[derivative(Default(value = "64.0"))]
    max_height: f32,
    visualization_queue: VisualizationQueue,
    rng: Rng,

    fft_calc_counter: u8, // Used to test occasional recomputation of FFT
    fft_buffer: Vec<f32>,
}
impl RandomizesBuffer for FrequencyDomainSettings {
    fn visualization_queue(&self) -> &VisualizationQueue {
        &self.visualization_queue
    }

    fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}
impl Displays for FrequencyDomainSettings {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
            | ui.add(DragValue::new(&mut self.max_width).prefix("width: "))
            | ui.add(DragValue::new(&mut self.max_height).prefix("height: "))
    }
}
impl FrequencyDomainSettings {
    const NAME: &'static str = "Audio Frequency Domain";

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        self.add_noise_to_buffer();

        // We act on 0 so that it's always initialized by the time we get to the
        // end of this method.
        if self.fft_calc_counter == 0 {
            if let Ok(queue) = self.visualization_queue.0.read() {
                let (slice_1, slice_2) = queue.as_slices();
                self.fft_buffer = analyze_spectrum(slice_1, slice_2).unwrap();
            }
        }
        self.fft_calc_counter += 1;
        if self.fft_calc_counter > 4 {
            self.fft_calc_counter = 0;
        }
        if !self.hide {
            ui.scope(|ui| {
                ui.set_max_width(self.max_width);
                ui.set_max_height(self.max_height);
                ui.add(FrequencyDomainWidget::widget(&self.fft_buffer));
            });
        }
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
struct SampleClip(#[derivative(Default(value = "Self::init_random_samples()"))] [Sample; 256]);
impl SampleClip {
    /// Creates 256 samples of noise.
    fn init_random_samples() -> [Sample; 256] {
        let mut r = [Sample::default(); 256];
        let mut rng = Rng::default();
        for s in &mut r {
            let value = rng.rand_float().fract() * 2.0 - 1.0;
            *s = Sample::from(value);
        }
        r
    }
}

#[derive(Debug)]
struct WidgetExplorer {
    legend: LegendSettings,
    grid: GridSettings,
    signal_path: SignalPathSettings,
    device_palette: DevicePaletteSettings,
    // signal_chain: SignalChainSettings,
    note_sequencer: NoteSequencerSettings,
    title_bar: TitleBarSettings,
    composer: ComposerSettings,
    wiggler: WigglerSettings,
    time_domain: TimeDomainSettings,
    frequency_domain: FrequencyDomainSettings,
    toy_synth: ToySynthSettings,
    toy_controller: ToyControllerSettings,
    toy_effect: ToyEffectSettings,
    toy_instrument: ToyInstrumentSettings,
}
impl WidgetExplorer {
    pub const NAME: &'static str = "Widget Explorer";

    pub fn new(_cc: &CreationContext, factory: EntityFactory<dyn EntityBounds>) -> Self {
        Self {
            legend: Default::default(),
            grid: Default::default(),
            signal_path: Default::default(),
            device_palette: DevicePaletteSettings::new(factory),
            note_sequencer: Default::default(),
            title_bar: Default::default(),
            composer: Default::default(),
            wiggler: Default::default(),
            time_domain: Default::default(),
            frequency_domain: Default::default(),
            toy_synth: Default::default(),
            toy_controller: Default::default(),
            toy_effect: Default::default(),
            toy_instrument: Default::default(),
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
            Self::wrap_settings(TimeDomainSettings::NAME, ui, |ui| self.time_domain.ui(ui));
            Self::wrap_settings(FrequencyDomainSettings::NAME, ui, |ui| {
                self.frequency_domain.ui(ui)
            });
            Self::wrap_settings(LegendSettings::NAME, ui, |ui| self.legend.ui(ui));
            Self::wrap_settings(SignalPathSettings::NAME, ui, |ui| self.signal_path.ui(ui));
            //            Self::wrap_settings(TrackSettings::NAME, ui, |ui| self.track_widget.ui(ui));
            Self::wrap_settings(DevicePaletteSettings::NAME, ui, |ui| {
                self.device_palette.ui(ui)
            });
            // Self::wrap_settings(SignalChainSettings::NAME, ui, |ui| self.signal_chain.ui(ui));
            Self::wrap_settings(ComposerSettings::NAME, ui, |ui| self.composer.ui(ui));
            Self::wrap_settings(GridSettings::NAME, ui, |ui| self.grid.ui(ui));
            Self::wrap_settings(NoteSequencerSettings::NAME, ui, |ui| {
                self.note_sequencer.ui(ui)
            });

            Self::wrap_settings(ToySynthSettings::NAME, ui, |ui| self.toy_synth.ui(ui));
            Self::wrap_settings(ToyControllerSettings::NAME, ui, |ui| {
                self.toy_controller.ui(ui)
            });
            Self::wrap_settings(ToyEffectSettings::NAME, ui, |ui| self.toy_effect.ui(ui));
            Self::wrap_settings(ToyInstrumentSettings::NAME, ui, |ui| {
                self.toy_instrument.ui(ui)
            });

            Self::wrap_settings(TitleBarSettings::NAME, ui, |ui| self.title_bar.ui(ui));
            Self::wrap_settings(WigglerSettings::NAME, ui, |ui| self.wiggler.ui(ui));
            self.debug_ui(ui);
        });
    }

    fn wrap_settings(
        name: &str,
        ui: &mut eframe::egui::Ui,
        add_body: impl FnOnce(&mut Ui) -> eframe::egui::Response,
    ) {
        CollapsingHeader::new(name)
            .show_background(true)
            .show_unindented(ui, add_body);
    }

    fn wrap_item(name: &str, ui: &mut eframe::egui::Ui, add_body: impl FnOnce(&mut Ui)) {
        ui.heading(name);
        add_body(ui);
        ui.separator();
    }

    fn debug_ui(&mut self, ui: &mut eframe::egui::Ui) {
        #[cfg(debug_assertions)]
        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "🐛 Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }
        let style: Style = (*ui.ctx().style()).clone();
        let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
        if let Some(visuals) = new_visuals {
            ui.ctx().set_visuals(visuals);
        }
    }

    fn show_center(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            //            self.track_widget.set_view_range(&self.legend.range);
            self.grid.set_view_range(&self.legend.range);
            self.note_sequencer.set_view_range(&self.legend.range);

            ui.horizontal_top(|ui| {
                ui.scope(|ui| {
                    ui.set_max_height(64.0);
                    Self::wrap_item(TimeDomainSettings::NAME, ui, |ui| self.time_domain.show(ui));
                    Self::wrap_item(FrequencyDomainSettings::NAME, ui, |ui| {
                        self.frequency_domain.show(ui)
                    });
                });
            });
            ui.heading("Timeline");
            self.legend.show(ui);
            Self::wrap_item(SignalPathSettings::NAME, ui, |ui| self.signal_path.show(ui));
            //         self.track_widget.show(ui);

            Self::wrap_item(DevicePaletteSettings::NAME, ui, |ui| {
                self.device_palette.show(ui)
            });
            // Self::wrap_item(SignalChainSettings::NAME, ui, |ui| {
            //     self.signal_chain.show(ui)
            // });
            Self::wrap_item(ComposerSettings::NAME, ui, |ui| self.composer.show(ui));

            Self::wrap_item(GridSettings::NAME, ui, |ui| self.grid.show(ui));
            Self::wrap_item(NoteSequencerSettings::NAME, ui, |ui| {
                self.note_sequencer.show(ui)
            });

            Self::wrap_item(ToySynthSettings::NAME, ui, |ui| self.toy_synth.show(ui));
            Self::wrap_item(ToyControllerSettings::NAME, ui, |ui| {
                self.toy_controller.show(ui)
            });
            Self::wrap_item(ToyEffectSettings::NAME, ui, |ui| self.toy_effect.show(ui));
            Self::wrap_item(ToyInstrumentSettings::NAME, ui, |ui| {
                self.toy_instrument.show(ui)
            });

            Self::wrap_item(TitleBarSettings::NAME, ui, |ui| self.title_bar.show(ui));
            Self::wrap_item(WigglerSettings::NAME, ui, |ui| self.wiggler.show(ui));
        });
    }
}
impl eframe::App for WidgetExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let bottom = egui::TopBottomPanel::bottom("bottom-panel")
            .resizable(false)
            .exact_height(24.0);
        let left = egui::SidePanel::left("left-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0);
        let center = egui::CentralPanel::default();

        bottom.show(ctx, |ui| {
            self.show_bottom(ui);
        });
        left.show(ctx, |ui| {
            self.show_left(ui);
        });
        center.show(ctx, |ui| {
            self.show_center(ui);
        });
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(WidgetExplorer::NAME)
            .with_maximized(true) // Not sure why this doesn't work
            .with_inner_size(eframe::epaint::vec2(1920.0, 1080.0))
            .to_owned(),
        ..Default::default()
    };

    let factory = BuiltInEntities::register(EntityFactory::default()).finalize();

    if let Err(e) = eframe::run_native(
        WidgetExplorer::NAME,
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(WidgetExplorer::new(cc, factory))
        }),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
