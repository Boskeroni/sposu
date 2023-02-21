use std::{fs::{self, File}, io::BufReader};
use serde::{Serialize, Deserialize};

use libosu::prelude::Beatmap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Song {
    pub audio_path: String,
    pub song_name: String,
    pub artist: String,
}

impl Song {
    fn new(beatmap: Beatmap, song_path: String) -> Self {
        let audio_path = format!("{}/{}", song_path, &beatmap.audio_filename);
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

    // for every folder in the directory
    for folder in song_folder {
        let folder = folder.unwrap();
        if !folder.metadata().unwrap().is_dir() {
            continue;
        }
        let valid_folder = fs::read_dir(folder.path().as_path()).unwrap();
        
        for file in valid_folder {
            let file = file.unwrap().path();
            if file.is_dir() {
                continue;
            }
            if file.extension().unwrap() != "osu" {
                continue;
            }
            
            let osu_file = File::open(&file).unwrap();
            let osu_file_reader = BufReader::new(osu_file);
            let parsed_map = Beatmap::parse(osu_file_reader);
    
            // parser sometimes errors, no clue why
            if let Ok(i) = parsed_map {
                let name_artist = format!("{}-{}", i.title, i.artist);
                if used_songs.iter().any(|e| e == &name_artist) {
                    continue;
                }
                used_songs.push(name_artist);
                let path = folder.path().to_string_lossy().replace(&song_path, "");
                let new_song = Song::new(i, path);
                songs.push(new_song);
            }
        }
    }
    songs
}