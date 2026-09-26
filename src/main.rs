#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod hud;
mod ink;
mod range;
mod terrain;
mod theme;
mod transparency;
mod wave;

use eframe::egui;

use app::ArtyBuddy;
use theme::{TITLE, WINDOW_SIZE};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(WINDOW_SIZE)
            .with_max_inner_size(WINDOW_SIZE)
            .with_resizable(false)
            .with_maximize_button(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_title(TITLE),
        ..Default::default()
    };

    eframe::run_native(
        TITLE,
        options,
        Box::new(|cc| Ok(Box::new(ArtyBuddy::new(cc)))),
    )
}
