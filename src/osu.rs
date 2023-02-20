use std::{fs::{self, File}, io::BufReader, path::PathBuf};
use serde::{Serialize, Deserialize};

use libosu::prelude::Beatmap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Song {
    pub audio_path: String,
    pub song_name: String,
    pub artist: String,
}

impl Song {
    fn new(beatmap: Beatmap, song_path: PathBuf) -> Self {
        let mut audio_path = song_path.to_string_lossy().to_string();
        audio_path.push('/');
        audio_path.push_str(&beatmap.audio_filename);
        Self { 
            audio_path,
            song_name: beatmap.title,
            artist: beatmap.artist,
        }
    }
}

pub fn load_songs(song_path: &str) -> Vec<Song> {
    let mut songs = Vec::new();
    let song_folder = fs::read_dir(song_path).unwrap();
    let mut used_songs = Vec::new();

    for entry in song_folder {
        let try_folder = entry.unwrap();
        if !try_folder.metadata().unwrap().is_dir() {
            continue;
        }
        let check_song_folder = fs::read_dir(try_folder.path().as_path()).unwrap();
        let check_osu_folder = fs::read_dir(try_folder.path().as_path()).unwrap();

        let mut sound_map_ratio = 0;
        
        // removes having to add the same song multiple times
        for file in check_song_folder {
            let trial = file.unwrap().path();
            if trial.is_dir() {
                continue;
            }
            if trial.extension().unwrap() == "mp3" {
                sound_map_ratio += 1;
            }
        }
        
        for file in check_osu_folder {
            let try_path = file.unwrap().path();
            if try_path.is_dir() {
                continue;
            }
            if try_path.extension().unwrap() != "osu" {
                continue;
            }
            
            let osu_file = File::open(&try_path).unwrap();
            let osu_file_reader = BufReader::new(osu_file);
            let parsed_map = Beatmap::parse(osu_file_reader);
    
            // parser sometimes errors, no clue why
            if let Ok(i) = parsed_map {
                let name_artist = format!("{}-{}", i.title, i.artist);
                if used_songs.contains(&name_artist) {
                    continue;
                }
                used_songs.push(name_artist);
                let new_song = Song::new(i, try_folder.path());
                songs.push(new_song);
                sound_map_ratio -= 1;
                if sound_map_ratio == 0 {
                    break;
                }
            }
        }
    }
    songs
}

pub fn serialize_osu_files(songs: &Vec<Song>, path: &str) {
    let serialized_songs = serde_json::to_string(songs).unwrap();
    std::fs::write(path, serialized_songs).unwrap();
}

pub fn deserialize_osu_files(path: &str) -> Vec<Song> {
    let data = fs::read_to_string(path).unwrap();
    let deserialize_data: Vec<Song> = serde_json::from_str(&data).unwrap();
    deserialize_data
}