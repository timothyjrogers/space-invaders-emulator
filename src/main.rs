#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;

use intel8080;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Space Invaders Emulator",
        native_options,
        Box::new(|cc| Box::new(crate::application::App::new(cc))),
    )
}