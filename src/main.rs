//! This crate provides a Rust implementation along with a graphical user interface to interact
//! with the many files of Sonic Riders for the GameCube. Previously, a lot of these tools were
//! behind multiple other tools, along with most of them being command-line tools. This crate aims
//! to alleviate some of that, by unifying a lot of the tools into a singular, centralized
//! application.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![warn(missing_docs)]

mod app;
pub mod riders;
pub mod util;

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Riders Toolkit",
        native_options,
        Box::new(|cc| Ok(Box::new(app::EguiApp::new(cc)))),
    )
}
