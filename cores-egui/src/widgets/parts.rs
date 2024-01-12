// Copyright (c) 2023 Mike Tsao. All rights reserved.


#[derive(Copy, Clone, Debug, Default)]
pub enum UiSize {
    #[default]
    Small,
    Medium,
    Large,
}
impl UiSize {
    pub fn from_height(height: f32) -> UiSize {
        if height <= 32.0 {
            UiSize::Small
        } else if height <= 128.0 {
            UiSize::Medium
        } else {
            UiSize::Large
        }
    }
}
