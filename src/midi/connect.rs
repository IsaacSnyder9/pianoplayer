use std::{
    error::Error,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
};

use crate::midi::instrument::{self, ActiveInstrument};
use midir::MidiInput;
enum State {
    Play { note: u8, velocity: u8 },
    Stop { note: u8 },
    Sustain { trigger: bool },
}

impl State {
    fn call(&self, active_instrument: &mut ActiveInstrument) -> Result<(), Box<dyn Error>> {
        match self {
            State::Play { note, velocity } => {
                active_instrument.play_note(note, velocity)?;
            }
            State::Stop { note } => {
                active_instrument.stop_note(&note)?;
            }
            State::Sustain { trigger } => {
                active_instrument.sustain(trigger)?;
            }
        };
        Ok(())
    }
}

pub fn midi_connection(
    port_name: String,
    current_note: Arc<Mutex<Vec<u8>>>,
    is_running: Arc<AtomicBool>
) -> Result<(), Box<dyn Error>> {
    let midi_in = MidiInput::new("midir input")?;
    let ports = midi_in.ports();
    if ports.is_empty() {
        println!("No MIDI output ports found.");
        return Ok(());
    }
    let selected_port = midi_in.ports().into_iter().find(|p| {
        midi_in
            .port_name(p)
            .map(|name| name.contains(&port_name))
            .unwrap_or(false)
    });
    let selected_port = match selected_port {
        Some(port) => port,
        None => {
            eprintln!("Could not find {} MIDI port.", &port_name);
            return Ok(());
        }
    };
    let port_name = midi_in.port_name(&selected_port)?;

    let sfz_path = "./SalamanderGrandPianoV3RetunedNL.sfz".to_string();
    let mut active_instrument = instrument::ActiveInstrument::new(sfz_path)?;

    let display_notes = current_note.clone();

    let _conn = midi_in.connect(
        &selected_port,
        &port_name,
        move |_, message, _| {
            if let Some(state) = read_note(message) {
                if let Err(e) = state.call(&mut active_instrument) {
                    eprintln!("error: {}", e)
                }
            }
            active_instrument.clean();
            *display_notes.lock().unwrap() = active_instrument.get_current_notes()
        },
        (),
    )?;

    while is_running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    #[allow(unreachable_code)]
    Ok(())
}

fn read_note(message: &[u8]) -> Option<State> {
    if message.len() != 3 {
        return None;
    }
    let note = message[1];
    let velocity = message[2];

    match message[0] & 0xF0 {
        // key released
        0x80 => Some(State::Stop { note }),
        // key pressed
        0x90 => {
            if velocity > 0 {
                Some(State::Play { note, velocity })
            } else {
                Some(State::Stop { note })
            }
        }
        // sustain peddle pressed or released
        0xB0 if note == 64 => Some(State::Sustain {
            trigger: velocity > 0,
        }),
        _ => None,
    }
}
