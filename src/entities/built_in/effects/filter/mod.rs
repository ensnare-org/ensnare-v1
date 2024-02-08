// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{
    cores::effects::{
        BiQuadFilterAllPassCore, BiQuadFilterBandPassCore, BiQuadFilterBandStopCore,
        BiQuadFilterHighPassCore, BiQuadFilterLowPass24dbCore,
    },
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
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterBandPass {
    uid: Uid,
    inner: BiQuadFilterBandPassCore,
}
impl BiQuadFilterBandPass {
    pub fn new_with(uid: Uid, inner: BiQuadFilterBandPassCore) -> Self {
        Self { uid, inner }
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
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterBandStop {
    uid: Uid,
    inner: BiQuadFilterBandStopCore,
}
impl BiQuadFilterBandStop {
    pub fn new_with(uid: Uid, inner: BiQuadFilterBandStopCore) -> Self {
        Self { uid, inner }
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
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterLowPass24db {
    uid: Uid,
    inner: BiQuadFilterLowPass24dbCore,
}
impl BiQuadFilterLowPass24db {
    pub fn new_with(uid: Uid, inner: BiQuadFilterLowPass24dbCore) -> Self {
        Self { uid, inner }
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
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterHighPass {
    uid: Uid,
    inner: BiQuadFilterHighPassCore,
}
impl BiQuadFilterHighPass {
    pub fn new_with(uid: Uid, inner: BiQuadFilterHighPassCore) -> Self {
        Self { uid, inner }
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
#[entity(HandlesMidi, GeneratesStereoSample, Ticks, Controls, SkipInner)]
pub struct BiQuadFilterAllPass {
    uid: Uid,
    inner: BiQuadFilterAllPassCore,
}
impl BiQuadFilterAllPass {
    pub fn new_with(uid: Uid, inner: BiQuadFilterAllPassCore) -> Self {
        Self { uid, inner }
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
            ui.add(BiQuadFilterBandPassWidget::widget(&mut self.inner))
        }
    }

    impl Displays for BiQuadFilterBandStop {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.add(BiQuadFilterBandStopWidget::widget(&mut self.inner))
        }
    }

    impl Displays for BiQuadFilterHighPass {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.add(BiQuadFilterHighPassWidget::widget(&mut self.inner))
        }
    }

    impl Displays for BiQuadFilterLowPass24db {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.add(BiQuadFilterLowPass24dbWidget::widget(&mut self.inner))
        }
    }

    impl Displays for BiQuadFilterAllPass {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.add(BiQuadFilterAllPassWidget::widget(&mut self.inner))
        }
    }
}
