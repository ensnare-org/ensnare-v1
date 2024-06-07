// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::prelude::*;
use anyhow::{anyhow, Error};
use eframe::egui::{ComboBox, DragValue, Label, RichText, Ui, Widget};
use ensnare::elements::Transport;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, IntoStaticStr};

/// Renders a [Transport].
#[derive(Debug)]
pub struct TransportWidget<'a> {
    transport: &'a mut Transport,
}
impl<'a> TransportWidget<'a> {
    fn new_with(transport: &'a mut Transport) -> Self {
        Self { transport }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(transport: &'a mut Transport) -> impl Widget + 'a {
        move |ui: &mut Ui| TransportWidget::new_with(transport).ui(ui)
    }
}
impl<'a> Widget for TransportWidget<'a> {
    fn ui(self, ui: &mut Ui) -> eframe::egui::Response {
        ui.horizontal_centered(|ui| {
            ui.add(
                DragValue::new(&mut self.transport.tempo.0)
                    .clamp_range(Tempo::range())
                    .min_decimals(1)
                    .speed(0.1)
                    .suffix(" BPM"),
            ) | ui.add(Label::new(
                RichText::new(format!("{}", self.transport.current_time()))
                    .text_style(eframe::egui::TextStyle::Monospace),
            )) | ui.add(TimeSignatureWidget::widget(
                &mut self.transport.time_signature,
            ))
        })
        .inner
    }
}

/// This enum is a short-term hack to present a simple drop-down menu of more
/// time signatures than just 4/4.
#[derive(Debug, Clone, Copy, EnumIter, AsRefStr, IntoStaticStr, PartialEq)]
enum TimeSignatureTempValues {
    #[strum(serialize = "4/4")]
    FourFour,
    #[strum(serialize = "3/4")]
    ThreeFour,
    #[strum(serialize = "2/2")]
    TwoTwo,
    #[strum(serialize = "6/8")]
    SixEight,
}
impl TryFrom<TimeSignature> for TimeSignatureTempValues {
    type Error = Error;

    fn try_from(value: TimeSignature) -> Result<Self, Self::Error> {
        match value.top {
            2 => match value.bottom {
                2 => {
                    return Ok(Self::TwoTwo);
                }
                _ => {}
            },
            3 => match value.bottom {
                4 => {
                    return Ok(Self::ThreeFour);
                }
                _ => {}
            },
            4 => match value.bottom {
                4 => {
                    return Ok(Self::FourFour);
                }
                _ => {}
            },
            6 => match value.bottom {
                8 => {
                    return Ok(Self::SixEight);
                }
                _ => {}
            },
            _ => {}
        }
        Err(anyhow!("Unrecognized time signature {:?}", value))
    }
}
impl From<TimeSignatureTempValues> for TimeSignature {
    fn from(value: TimeSignatureTempValues) -> Self {
        match value {
            TimeSignatureTempValues::FourFour => Self::new_with(4, 4).unwrap(),
            TimeSignatureTempValues::ThreeFour => Self::new_with(3, 4).unwrap(),
            TimeSignatureTempValues::TwoTwo => Self::new_with(2, 2).unwrap(),
            TimeSignatureTempValues::SixEight => Self::new_with(6, 8).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct TimeSignatureWidget<'a> {
    time_signature: &'a mut TimeSignature,
}
impl<'a> TimeSignatureWidget<'a> {
    fn new_with(time_signature: &'a mut TimeSignature) -> Self {
        Self { time_signature }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(time_signature: &'a mut TimeSignature) -> impl Widget + 'a {
        move |ui: &mut Ui| TimeSignatureWidget::new_with(time_signature).ui(ui)
    }
}
impl<'a> Widget for TimeSignatureWidget<'a> {
    fn ui(self, ui: &mut Ui) -> eframe::egui::Response {
        let ts = TimeSignatureTempValues::try_from(*self.time_signature).unwrap();
        let mut changed = false;
        let s: &str = ts.into();
        let mut response = ComboBox::from_label("")
            .selected_text(s)
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                for ts in TimeSignatureTempValues::iter() {
                    let s: &str = ts.into();
                    let mut ts_copy = ts.clone();
                    if ui.selectable_value(&mut ts_copy, ts, s).clicked() {
                        *self.time_signature = ts_copy.into();
                        changed = true;
                    }
                }
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }
}
