use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::Path,
    time::{Duration, Instant},
};

use kira::{
    AudioManager, AudioManagerSettings, Capacities, DefaultBackend, Semitones, Tween,
    sound::{
        PlaybackState,
        static_sound::{StaticSoundData, StaticSoundHandle},
    },
};
use vibelang_sfz::{
    RoundRobinState, SfzInstrument, TriggerMode, find_matching_regions, load_sfz_instrument,
};

pub type Note = u8;
pub type ActivePlayers = HashMap<Note, ActiveNote>;
#[derive(Debug)]
pub struct ActiveNote {
    sound_handles: Vec<StaticSoundHandle>,
    pub off_time: Option<Instant>,
}

type BufferId = i32;
pub type SampleBank = HashMap<BufferId, StaticSoundData>;

pub struct ActiveInstrument {
    manager: AudioManager,
    sample_bank: HashMap<BufferId, StaticSoundData>,
    instrument: SfzInstrument,
    rr_state: RoundRobinState,
    active_players: ActivePlayers,
    is_sustaining: bool,
    sustaining: HashSet<u8>,
}

impl ActiveInstrument {
    pub fn new(sfz_path: String) -> Result<Self, Box<dyn Error>> {
        let (instrument, sample_bank) = Self::preload_samples(sfz_path)?;
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings {
            capacities: Capacities {
                send_track_capacity: 128,
                ..Default::default()
            },
            ..Default::default()
        })?;
        let rr_state = vibelang_sfz::RoundRobinState::new();
        let active_players = ActivePlayers::new();
        Ok(Self {
            manager,
            sample_bank,
            instrument,
            rr_state,
            active_players,
            is_sustaining: false,
            sustaining: HashSet::new(),
        })
    }

    fn load_sample(path: &Path) -> Result<StaticSoundData, Box<dyn Error>> {
        //println!("loading: {:?}", path);
        let source = StaticSoundData::from_file(path)?;
        Ok(source)
    }

    fn preload_samples(
        sfz_path: String,
    ) -> Result<(SfzInstrument, SampleBank), Box<dyn std::error::Error>> {
        let mut sample_bank = SampleBank::new();

        let mut next_buffer_id = 100;
        println!("starting sfz load");

        let instrument = load_sfz_instrument(
            sfz_path,
            "new_instrument".to_string(),
            &mut |path: &Path, buffer_id| {
                //println!("callback called for {} -> {}", path.display(), buffer_id);
                let loaded_sample = Self::load_sample(path).unwrap();

                sample_bank.insert(buffer_id, loaded_sample);
                Ok(())
            },
            &mut next_buffer_id,
        )?;

        println!("regions: {}", instrument.regions.len());
        println!("samples loaded: {}", sample_bank.len());

        Ok((instrument, sample_bank))
    }

    pub fn play_note(&mut self, note: &u8, velocity: &u8) -> Result<(), Box<dyn Error>> {
        self.sustaining.remove(&note);

        if let Some(note) = self.active_players.remove(note) {
            for mut handle in note.sound_handles {
                handle.stop(Default::default());
            }
        }

        let regions = find_matching_regions(
            &self.instrument,
            *note,
            *velocity,
            TriggerMode::Attack,
            &mut self.rr_state,
        );

        let mut handles = Vec::new();

        for region in regions {
            let sample = self
                .sample_bank
                .get(&region.buffer_id)
                .ok_or("Sample not found in sample bank")?;

            let keycenter = region.opcodes.pitch_keycenter.unwrap_or(*note) as f64;
            let semitone = *note as f64 - keycenter;

            let mut sound = self.manager.play(sample.clone())?;

            let fade_duration = Duration::from_millis(5);
            sound.set_playback_rate(Semitones(semitone), Tween::default());
            Self::fade(&mut sound, Some(-60.0), 0.0, fade_duration);

            handles.push(sound);
        }

        if !handles.is_empty() {
            self.active_players.insert(
                *note,
                ActiveNote {
                    sound_handles: handles,
                    off_time: None,
                },
            );
        }

        println!("total sounds: {}", self.total_sounds());

        Ok(())
    }
    pub fn stop_note(&mut self, note: &u8) -> Result<(), Box<dyn Error>> {
        if self.is_sustaining {
            self.sustaining.insert(*note);
            return Ok(());
        }

        if let Some(active_note) = self.active_players.get_mut(note) {
            if active_note.off_time.is_none() {
                let now = Instant::now();
                let fade_duration = Duration::from_millis(250);

                for sound in active_note.sound_handles.iter_mut() {
                    sound.stop(Tween {
                        start_time: kira::StartTime::Immediate,
                        duration: fade_duration,
                        easing: kira::Easing::Linear,
                    });
                    //Self::fade(&mut active_note.sound_handle, None, -60.0, fade_duration);
                }
                active_note.off_time = Some(now);
            }
        };
        Ok(())
    }

    pub fn sustain(&mut self, sustain_trigger: &bool) -> Result<(), Box<dyn Error>> {
        self.is_sustaining = *sustain_trigger;
        if !sustain_trigger {
            let notes: Vec<_> = self.sustaining.drain().collect();

            for note in notes {
                self.stop_note(&note)?;
            }
        };
        Ok(())
    }

    pub fn clean(&mut self) {
        self.active_players.retain(|_, active_note| {
            active_note.sound_handles.retain_mut(|handle| {
                handle.state() != PlaybackState::Stopped
            });

            !active_note.sound_handles.is_empty()
        });
    }

    fn total_sounds(&self) -> usize {
        self.active_players
            .values()
            .map(|notes| notes.sound_handles.len())
            .sum()
    }

    pub fn get_current_notes(&self) -> Vec<u8> {
        self.active_players
            .iter()
            .filter(|(_, active_note)| active_note.off_time.is_none())
            .map(|(&key, _)| key)
            .collect()
    }

    fn fade(sound: &mut StaticSoundHandle, from: Option<f32>, to: f32, duration: Duration) {
        if let Some(vol) = from {
            sound.set_volume(vol, Tween::default());
        }
        sound.set_volume(
            to,
            Tween {
                duration,
                ..Default::default()
            },
        );
    }
}
