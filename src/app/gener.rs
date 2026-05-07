use eframe::egui::{self, Ui};
use midir::MidiInput;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

use crate::{app::{settings::PianoSettings, sheetmusic}, midi::connect};
pub enum Screen {
    MainMenu,
    MusicScreen,
    Settings,
}

pub struct AppState {
    pub current_screen: Screen,
    pub settings: PianoSettings,
    pub current_notes: Arc<Mutex<Vec<u8>>>,
    pub sound_thread: Option<JoinHandle<()>>,
    pub is_running: Arc<AtomicBool>,
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_hex("#F9EBDF").unwrap()))
            .show(ctx, |ui| match self.current_screen {
                Screen::MainMenu => {
                    self.main_menu(ui);
                }
                Screen::MusicScreen => {
                    self.music_screen(ui);
                }
                Screen::Settings => {
                    self.settings(ui);
                }
            });

        ctx.request_repaint();
    }
}

impl AppState {

    pub fn main_menu(&mut self, ui: &mut Ui) {
        if ui
            .add(egui::Button::new(
                egui::RichText::new("Sheet Music").size(30.0),
            ))
            .clicked()
        {
            self.set_screen(Screen::MusicScreen);
        }
        if ui
            .add(egui::Button::new(
                egui::RichText::new("Settings").size(30.0),
            ))
            .clicked()
        {
            self.set_screen(Screen::Settings);
        }
    }

    pub fn music_screen(&mut self, ui: &mut Ui) {
        if let None = self.sound_thread {
            self.run_midi();
        }

        if ui
            .add(egui::Button::new(
                egui::RichText::new("Main Menu").size(30.0),
            ))
            .clicked()
        {
            self.set_screen(Screen::MainMenu);
        }

        let notes = self.current_notes.lock().unwrap().clone();
        let note_set = notes
            .iter()
            .map(|n| midi_to_name(n))
            .collect::<Vec<_>>()
            .join(" ");
        ui.label(egui::RichText::new(note_set).size(40.0));

        sheetmusic::display_xml(ui).ok();

    }

    pub fn settings(&mut self, ui: &mut Ui) {
        if ui
            .add(egui::Button::new(
                egui::RichText::new("Main Menu").size(30.0),
            ))
            .clicked()
        {
            self.set_screen(Screen::MainMenu);
        }

        egui::ComboBox::from_label("Select MIDI")
            .selected_text(format!("{}", &self.settings.midi))
            .show_ui(ui, |ui| {
                let midi_in = MidiInput::new("midir input").ok();
                if let Some(midi) = midi_in {
                    let ports = midi.ports();
                    for port in ports {
                        let name = midi.port_name(&port).unwrap_or_else(|_| "Unknown".into());
                        let resp =
                            ui.selectable_value(&mut self.settings.midi, name.clone(), &name);

                        if resp.changed() {
                            self.settings.save();
                        }
                    }
                } else {
                    ui.label("No MIDI devices found");
                }
            });
    }

    // helpers

    fn set_screen(&mut self, target: Screen) {
        // on screen exit
        match self.current_screen {
            Screen::MusicScreen => {
                self.is_running.store(false, Ordering::SeqCst);
                self.stop_midi();
            }
            _ => {}
        }
        // om screen enter
        match target {
            Screen::MusicScreen => {
                self.is_running.store(true, Ordering::SeqCst);
            }
            _ => {}
        }
        self.current_screen = target;
    }

    fn run_midi(&mut self) {
        let port_name = self.settings.midi.to_string();
        let current_notes_copy = Arc::clone(&self.current_notes);
        let is_running = Arc::clone(&self.is_running);
        let thread = std::thread::spawn(move || {
            connect::midi_connection(port_name, current_notes_copy, is_running)
                .inspect_err(|e| eprintln!("Error: {}", e))
                .ok();
        });
        self.sound_thread = Some(thread)
    }

    fn stop_midi(&mut self) {
        if let Some(thread) = self.sound_thread.take() {
            let _ = thread.join();
        };
    }
}

// helpers

fn midi_to_name(note: &u8) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    let note_type = (note % 12) as usize;
    let octave = (note / 12) as i32 - 1;

    format!("{}{}", names[note_type], octave)
}

