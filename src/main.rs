use std::{
    error::Error,
    sync::{Arc, Mutex, atomic::AtomicBool},
};

use crate::app::settings::PianoSettings;

mod app;
mod midi;
fn main() -> Result<(), Box<dyn Error>> {
    let current_notes = Arc::new(Mutex::new(Vec::new()));
    let settings = PianoSettings::new();
    
    eframe::run_native(
        "MIDI Display",
        eframe::NativeOptions::default(),
            Box::new(|cc| {
            app::sheetmusic::load_font(&cc.egui_ctx);
            Ok(Box::new(app::gener::AppState {
                current_screen: app::gener::Screen::MainMenu,
                settings: settings,
                current_notes: current_notes.clone(),
                sound_thread: None,
                is_running: Arc::new(AtomicBool::new(true))
            }))
        }),
    )?;

    Ok(())
}
