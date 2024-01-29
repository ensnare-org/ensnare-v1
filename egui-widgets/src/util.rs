// Copyright (c) 2024 Mike Tsao. All rights reserved.

// Call this last in any ui() body if you want to fill the remaining space.
pub fn fill_remaining_ui_space(ui: &mut eframe::egui::Ui) {
    ui.allocate_space(ui.available_size());
}
