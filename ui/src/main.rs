#![windows_subsystem = "windows"]

mod app;
mod config_manager;
mod decoder_worker;
mod error_manager;
mod file_manager;
mod task_manager;
mod ui_components;

use app::UnlockMusicApp;

use eframe::egui::{IconData, ViewportBuilder};
use std::sync::Arc;

fn load_icon() -> Arc<IconData> {
    let image_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(image_bytes)
        .unwrap()
        .resize(16, 16, image::imageops::FilterType::Nearest)
        .into_rgba8();
    Arc::new(IconData {
        rgba: image.to_vec(),
        width: 16,
        height: 16,
    })
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_icon(load_icon()),

        ..Default::default()
    };

    eframe::run_native(
        "Unlock Music",
        options,
        Box::new(|cc| Ok(Box::new(UnlockMusicApp::new(cc)))),
    )
}
