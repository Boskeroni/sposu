use std::{fs::{self, File}, io::BufReader};
use serde::{Serialize, Deserialize};

use libosu::prelude::Beatmap;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Mod {
    NoMod,
    Nightcore,
    DoubleTime,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Song {
    pub audio_path: String,
    pub song_name: String,
    pub artist: String,
    pub modifier: Mod,
}

impl Song {
    fn new(beatmap: Beatmap, song_path: String) -> Self {
        let audio_path = format!("{}/{}", song_path, &beatmap.audio_filename);
        Self { 
            audio_path,
            song_name: beatmap.title,
            artist: beatmap.artist,
            modifier: Mod::NoMod
        }
    }
}

/// LOADS ALL THE SONGS IN THE SONG FOLDER PROVIDED
/// IT ERRORS ON SOME OF THE FILES WHICH IS BAD BUT IDK HOW TO FIX IT
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
        let mp3_folder = fs::read_dir(folder.path().as_path()).unwrap();

        let mut mp3_ratio = 0;
        for file in mp3_folder {
            if let Some(e) = file.unwrap().path().extension() {
                if e == "mp3" {
                    mp3_ratio += 1;
                }
            }
        }
        
        for file in valid_folder {
            let file = file.unwrap();
            if let Some(e) = file.path().extension() {
                if e.to_string_lossy() != "osu" {
                    continue;
                }
            } else {
                continue;
            };

            let osu_file_reader = BufReader::new(File::open(&file.path()).unwrap());    
            if let Ok(i) = Beatmap::parse(osu_file_reader) {
                let name_artist = format!("{}-{}", i.title, i.artist);
                if used_songs.iter().any(|e| e == &name_artist) {
                    continue;
                }
                used_songs.push(name_artist);
                let path = folder.path().to_string_lossy().replace(&song_path, "");
                songs.push(Song::new(i, path));
                mp3_ratio -= 1;
                if mp3_ratio == 0 {
                    break;
                }
            }
        }
    }
    songs
}