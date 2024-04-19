// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    cores::effects::{
        BiQuadFilterAllPassCore, BiQuadFilterBandPassCore, BiQuadFilterBandStopCore,
        BiQuadFilterHighPassCore, BiQuadFilterLowPass24dbCore,
    },
    egui::BiQuadFilterWidgetAction,
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterBandPass {
    uid: Uid,
    inner: BiQuadFilterBandPassCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterBandPass {
    pub fn new_with(uid: Uid, inner: BiQuadFilterBandPassCore) -> Self {
        Self {
            uid,
            inner,
            widget_action: Default::default(),
            action: Default::default(),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterBandStop {
    uid: Uid,
    inner: BiQuadFilterBandStopCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterBandStop {
    pub fn new_with(uid: Uid, inner: BiQuadFilterBandStopCore) -> Self {
        Self {
            uid,
            inner,
            widget_action: Default::default(),
            action: Default::default(),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterLowPass24db {
    uid: Uid,
    inner: BiQuadFilterLowPass24dbCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterLowPass24db {
    pub fn new_with(uid: Uid, inner: BiQuadFilterLowPass24dbCore) -> Self {
        Self {
            uid,
            inner,
            widget_action: Default::default(),
            action: Default::default(),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterHighPass {
    uid: Uid,
    inner: BiQuadFilterHighPassCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterHighPass {
    pub fn new_with(uid: Uid, inner: BiQuadFilterHighPassCore) -> Self {
        Self {
            uid,
            inner,
            widget_action: Default::default(),
            action: Default::default(),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterAllPass {
    uid: Uid,
    inner: BiQuadFilterAllPassCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterAllPass {
    pub fn new_with(uid: Uid, inner: BiQuadFilterAllPassCore) -> Self {
        Self {
            uid,
            inner,
            widget_action: Default::default(),
            action: Default::default(),
        }
    }
}

#[cfg(feature = "egui")]
mod egui {
    use super::*;
    use crate::egui::{
        BiQuadFilterAllPassWidget, BiQuadFilterBandPassWidget, BiQuadFilterBandStopWidget,
        BiQuadFilterHighPassWidget, BiQuadFilterLowPass24dbWidget,
    };

    impl Displays for BiQuadFilterBandPass {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut action = None;
            let response = ui.add(BiQuadFilterBandPassWidget::widget(
                &mut self.inner,
                &mut action,
            ));
            response
        }

        fn set_action(&mut self, action: DisplaysAction) {
            self.action = Some(action);
        }

        fn take_action(&mut self) -> Option<DisplaysAction> {
            self.action.take()
        }
    }

    impl Displays for BiQuadFilterBandStop {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut action = None;
            let response = ui.add(BiQuadFilterBandStopWidget::widget(
                &mut self.inner,
                &mut action,
            ));
            response
        }

        fn set_action(&mut self, action: DisplaysAction) {
            self.action = Some(action);
        }

        fn take_action(&mut self) -> Option<DisplaysAction> {
            self.action.take()
        }
    }

    impl Displays for BiQuadFilterHighPass {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut action = None;
            let response = ui.add(BiQuadFilterHighPassWidget::widget(
                &mut self.inner,
                &mut action,
            ));
            response
        }

        fn set_action(&mut self, action: DisplaysAction) {
            self.action = Some(action);
        }

        fn take_action(&mut self) -> Option<DisplaysAction> {
            self.action.take()
        }
    }

    impl Displays for BiQuadFilterLowPass24db {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut action = None;
            let response = ui.add(BiQuadFilterLowPass24dbWidget::widget(
                &mut self.inner,
                &mut action,
            ));
            if let Some(action) = self.widget_action.take() {
                match action {
                    BiQuadFilterWidgetAction::Link(uid, index) => {
                        self.set_action(DisplaysAction::Link(uid, index));
                    }
                }
            }
            response
        }

        fn set_action(&mut self, action: DisplaysAction) {
            self.action = Some(action);
        }

        fn take_action(&mut self) -> Option<DisplaysAction> {
            self.action.take()
        }
    }

    impl Displays for BiQuadFilterAllPass {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut action = None;
            let response = ui.add(BiQuadFilterAllPassWidget::widget(
                &mut self.inner,
                &mut action,
            ));
            response
        }

        fn set_action(&mut self, action: DisplaysAction) {
            self.action = Some(action);
        }

        fn take_action(&mut self) -> Option<DisplaysAction> {
            self.action.take()
        }
    }
}
