#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod memory;
mod space_invaders_memory;
mod cpu;
mod conditions;
mod emulator;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Space Invaders",
        native_options,
        Box::new(|cc| Box::new(crate::emulator::TemplateApp::new(cc))),
    )
}
/*
fn main() {
    let memory = Box::new(space_invaders_memory::SpaceInvadersMemory::new([0; 8_192]));
    let c = cpu::Cpu::new(memory);
    println!("{}", c);
}
*/