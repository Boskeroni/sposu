use std::fs::File;

use rand::Rng;
use rodio::{Sink, OutputStream, OutputStreamHandle};
use serde::{Deserialize, Serialize};
use crate::{osu::{Song, Mod}, serialize::{deserialize, self}};

pub enum PlayBarOutput {
    None,
    Playlist,
    Normal,
}

pub struct Player {
    pub sink: Sink,
    pub _stream: OutputStream,
    pub stream_handle: OutputStreamHandle,
    pub playbar_output: PlayBarOutput,
    pub current_songs: Vec<Song>,
    pub current_playlist: Option<Playlist>,
    pub hovered_index: usize,
    pub parent_path: String,
    pub playing_index: usize,
    pub volume: f32,
    pub is_playing: bool,
}

impl Player {
    pub fn new(parent_path: String) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::new_idle().0;
        let currently_playing = PlayBarOutput::None;
        Self {
            sink,
            _stream,
            stream_handle,
            playbar_output: currently_playing,
            current_songs: Vec::new(),
            current_playlist: None,
            hovered_index: 0,
            parent_path,
            playing_index: 0,
            volume: 0.5,
            is_playing: false
        }
    }

    // removes the hovered song from the playbar
    pub fn remove_hovered_song(&mut self) {
        if self.playing_index == self.hovered_index {
            self.sink.stop();
        }
        self.current_songs.remove(self.hovered_index);
    }

    // removes the playlist from the whole struct
    pub fn pop_current_playlist(&mut self) -> Playlist {
        self.playbar_output = PlayBarOutput::None;
        let temp = self.current_playlist.clone().unwrap();
        self.current_playlist = None;
        temp
    }

    // loads the playlist
    pub fn load_playlist(&mut self, playlist: Playlist) {
        self.current_playlist = Some(playlist);
    }

    // loads playlist into paybar
    pub fn load_playbar_playlist(&mut self) {
        self.current_songs = Vec::new();
        self.current_songs.append(&mut self.current_playlist.clone().unwrap().songs.clone());
        self.playbar_output = PlayBarOutput::Playlist;
        self.playing_index = 0;
    }

    // removes the playlist from the playbar
    pub fn unload_playbar_playlist(&mut self) {
        self.current_songs = Vec::new();
        self.playbar_output = PlayBarOutput::None;
        self.sink.stop();
    }

    // adds regular song
    pub fn add_normal_song(&mut self, song: Song) {
        self.current_songs.push(song);
        self.playbar_output = PlayBarOutput::Normal;
    }

    // function used when enter is pressed on now playing
    pub fn force_new_song(&mut self) {
        if self.hovered_index == self.playing_index {
            return;
        }
        let new_song = &self.current_songs[self.hovered_index].clone();
        self.playing_index = self.hovered_index;

        self.add_song_to_sink(new_song);
    }

    // tries loading new song, adds it to sink if successful
    pub fn try_new_song(&mut self) {
        if self.current_songs.is_empty() {
            return;
        }

        let new_song = match self.playbar_output {
            PlayBarOutput::Playlist => {
                let data = self.current_playlist.clone().unwrap().get_next_song();
                if data.is_none() { return; }
                
                let (song, index) = data.unwrap();
                self.playing_index = index;
                song
            }
            PlayBarOutput::Normal => {
                if self.is_playing {
                    self.current_songs.remove(self.playing_index);
                }
                if self.current_songs.is_empty() {
                    self.is_playing = false;
                    return;
                }

                if self.current_songs.len() == self.playing_index {
                    self.playing_index -= 1;
                }

                self.current_songs[self.playing_index].clone()
            }
            _ => return,
        };

        self.add_song_to_sink(&new_song);
    }

    // adds a song to the sink
    fn add_song_to_sink(&mut self, song: &Song) {
        let path = format!("{}{}", self.parent_path, song.audio_path);
        let audio_file = File::open(path).unwrap();
        self.sink = self.stream_handle.play_once(audio_file).unwrap();

        let speed = match song.modifier {
            Mod::NoMod => 1.0,
            _ => 1.5,
        };

        self.sink.set_speed(speed);
        self.sink.set_volume(self.volume);

        self.is_playing = true;
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedPlaylist {
    name: String,
    songs: Vec<Song>,
}

#[derive(Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
    pub shuffle_on: bool,
    pub repeat_on: bool,
    song_index: usize,
    _hovered_index: usize,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
            shuffle_on: false,
            repeat_on: false,
            song_index: 0,
            _hovered_index: 0,
        }
    }

    pub fn from_serialized(path: &str) -> Vec<Self> {
        let raw_data: Vec<SerializedPlaylist> = deserialize(path).unwrap();
        let playlists = raw_data.iter().map(|p| {
            Self {
                name: p.name.clone(),
                songs: p.songs.clone(),
                shuffle_on: false,
                repeat_on: false,
                song_index: 0,
                _hovered_index: 0,
            }
        }).collect();
        playlists
    }

    pub fn get_next_song(&mut self) -> Option<(Song, usize)> {
        if self.shuffle_on {
            let song_index = rand::thread_rng().gen_range(0..self.songs.len()-1);
            let new_song = self.songs[song_index].clone();
            return Some((new_song, song_index));
        }
        self.song_index += 1;
        if self.song_index < self.songs.len() {
            return Some((self.songs[self.song_index-1].clone(), self.song_index-1));
        }

        self.song_index = 0;
        if self.repeat_on {
            return Some((self.songs[self.song_index].clone(), self.song_index));
        }
        return None;
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle_on = !self.shuffle_on;
    }
    
    pub fn toggle_repeat(&mut self) {
        self.repeat_on = !self.repeat_on;
    }
}

pub fn serialize_playlists(playlists: &Vec<Playlist>, path: &str) {
    let raw_data: Vec<SerializedPlaylist> = playlists.iter().map(|p| {
        SerializedPlaylist {
            name: p.name.clone(),
            songs: p.songs.clone()
        }
    }).collect();

    serialize::serialize(&raw_data, path);
}