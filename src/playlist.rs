use std::collections::VecDeque;
use std::fs::File;

use rodio::{OutputStream, OutputStreamHandle, Sink};
use serde::{Serialize, Deserialize};
use rand::thread_rng;
use rand::seq::SliceRandom;
use crate::{osu::{Song, Mod}, serialize::{deserialize, self}};

pub struct Player {
    pub sink: Sink,
    pub is_playing: bool,
    pub current_songs: VecDeque<Song>,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    current_songs_i: usize,
    parent_path: String,
    volume: f32,
    is_playing_playlist: bool,
    current_playlist: Option<Playlist>
}

impl Player {
    pub fn new(parent_path: String) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::new_idle().0;
        Self {
            sink,
            _stream,
            stream_handle,
            is_playing: false,
            current_songs: VecDeque::new(),
            current_songs_i: 0,
            parent_path,
            volume: 0.5,
            is_playing_playlist: false,
            current_playlist: None,
        }
    }

    pub fn play_selected_song(&mut self) {
        if self.is_playing {
            self.current_songs.remove(self.current_songs_i);
            if self.current_songs_i != 0 {
                self.current_songs_i -= 1;
            }
        }
        if self.current_songs.len() == 0 {
            self.is_playing = false;
            return;
        }
        let new_song = match self.is_playing_playlist {
          true => {
            let binding = self.current_playlist.as_mut().unwrap().get_next_song();
            binding.unwrap()
        },
          false => self.current_songs[self.current_songs_i].clone(),
        };
        let path = format!("{}{}", self.parent_path, &new_song.audio_path);
        let audio_file = File::open(path).unwrap();

        self.sink = self.stream_handle.play_once(audio_file).unwrap();
        let speed = match new_song.modifier {
            Mod::NoMod => 1.0,
            _ => 1.5,
        };

        self.sink.set_volume(self.volume);
        self.sink.set_speed(speed);
        self.is_playing = true;
    }
}

#[derive(Serialize, Deserialize)]
struct RawPlaylist {
    name: String,
    songs: Vec<Song>,
}

#[derive(Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
    pub shuffle_on: bool,
    pub repeat_on: bool,
    pub index: usize,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
            shuffle_on: false,
            repeat_on: false,
            index: 0
        }
    }

    pub fn from_serialized(path: String) -> Vec<Self> {
        let raw_data: Vec<RawPlaylist> = deserialize(&path).unwrap();
        let playlists = raw_data.iter().map(|p| {
            Self {
                name: p.name.clone(),
                songs: p.songs.clone(),
                shuffle_on: false,
                repeat_on: false,
                index: 0,
            }
        }).collect();
        playlists
    }

    // returns none when end of playlist, shuffle and repeat both off
    pub fn get_next_song(&mut self) -> Option<Song> {
        if self.shuffle_on {
            let new_song = self.songs.choose(&mut thread_rng()).unwrap();
            return Some(new_song.clone());
        }
        self.index += 1;
        if self.index == self.songs.len() {
            self.index = 0;
            if self.repeat_on {
                return Some(self.songs[self.index].clone());
            }
            return None;
        }
        return Some(self.songs[self.index].clone());
    }
}

pub fn serialize_playlists(playlists: &Vec<Playlist>, path: &str) {
    let raw_data: Vec<RawPlaylist> = playlists.iter().map(|p| {
        RawPlaylist {
            name: p.name.clone(),
            songs: p.songs.clone(),
        }
    }).collect();

    serialize::serialize(&raw_data, path)
}