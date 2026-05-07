use std::{fs, path::Path};

use config::{Config, File};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct PianoSettings {
    pub midi: String
}

impl PianoSettings {
    pub fn new() -> Self{
        let s = Config::builder()
            .set_default("midi", "None").unwrap()
            .add_source(File::with_name("settings.toml").required(false))
            .build().unwrap();
        
        s.try_deserialize().unwrap_or_else(|_| PianoSettings::default())
    }

    pub fn save(&mut self){
        let path = Path::new("settings.toml");
        
        match toml::to_string_pretty(self) {
            Ok(s) => {
                if let Err(e) = fs::write(path, s) {
                    eprintln!("could not save settings {}", e);
                }
            }
            Err(e) => eprintln!("could not serialize settings {}", e)
        };
    }
}