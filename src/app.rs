use std::collections::VecDeque;

use rodio::{Sink, OutputStreamHandle, OutputStream};
use serde::{Serialize, Deserialize};
use std::fs::File;
use crossterm::event::{KeyEvent, KeyCode};

use crate::osu::Song;
use crate::serialize;

#[derive(Copy, Clone, Debug)]
pub enum UIMode {
    Input,
    Normal,
    Playlist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
}

impl Playlist {
    fn new(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    pub song_path: String,
    pub serialized_songs: bool,
    pub serialize_path: String,
    pub playlist_path: String,
}

pub struct App {
    pub current_mode: UIMode,
    pub sink: Sink,
    pub search: String,
    pub shown_songs: Vec<Song>,
    pub listening_songs: VecDeque<Song>,
    pub playlists: Vec<Playlist>,
    pub playlist_index: usize,
    pub index: usize,
    pub new_playlist_name: String,
    pub shown_playlist: Option<Playlist>,
    pub adding_playlist: bool,
    all_songs: Vec<Song>,
    stream_handle: OutputStreamHandle,
    _stream: OutputStream,
    is_playing: bool,
    volume: f32,
    glob_data: AppData,
}

impl App {
    pub fn new(songs: Vec<Song>, glob_data: AppData, playlists: Vec<Playlist>) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::new_idle().0;
        Self {
            search: String::new(),
            all_songs: songs.clone(),
            sink,
            stream_handle,
            _stream,
            shown_songs: songs,
            listening_songs: VecDeque::new(),
            index: 0,
            is_playing: false,
            current_mode: UIMode::Normal,
            volume: 1.0,
            playlists,
            playlist_index: 0,
            new_playlist_name: String::new(),
            shown_playlist: None,
            adding_playlist: false,
            glob_data
        }
    }
    
    pub fn to_valid_path(&self, file: &str) -> String {
        format!("{}/{}", self.glob_data.song_path, file)
    }

    pub fn global_handler(&mut self, key: &KeyEvent) -> Result<(), i32> {
        match key.code {
            KeyCode::Tab => {
                match self.current_mode {
                    UIMode::Input => self.current_mode = UIMode::Playlist,
                    UIMode::Playlist => self.current_mode = UIMode::Normal,
                    UIMode::Normal => self.current_mode = UIMode::Input,
                }
            }
            KeyCode::Esc => {
                match self.current_mode {
                    UIMode::Normal => return Err(1),
                    _ => self.current_mode = UIMode::Normal
                }
            }
            _ => {}
        }
        return Ok(())
    }

    pub fn specific_handler(&mut self, key: &KeyEvent) {
        match self.current_mode {
            UIMode::Input => {
                self.input_handler(key);
            }
            UIMode::Normal => {
                match key.code {
                    KeyCode::Esc => {},
                    _ => self.normal_handler(key)
                }
            }
            UIMode::Playlist => {
                self.playlist_handler(key);
            }
        }
    }

    pub fn get_matching_songs(&mut self) {
        self.shown_songs = vec![];
        for song in self.all_songs.clone() {
            if song.song_name.to_lowercase().contains(&self.search.to_lowercase()) {
                self.shown_songs.push(song);
            }
        }
    }

    pub fn play_next_song(&mut self) {
        if self.is_playing {
            self.listening_songs.pop_front();
        }
        if self.listening_songs.len() == 0 {
            self.is_playing = false;
            return;
        }
        let path = format!("{}{}", self.glob_data.song_path, self.listening_songs[0].audio_path.clone());
        let audio_file = File::open(path).unwrap();
        self.sink = self.stream_handle.play_once(audio_file).unwrap();
        self.sink.set_volume(self.volume);
        self.is_playing = true;
    }

    fn input_handler(&mut self, key: &KeyEvent) {
        match key.code {
            // adds new song to the playlist
            KeyCode::Right => {
                if self.playlists.len() == 0 {
                    return;
                }
                if self.shown_songs.len() == 0 {
                    return;
                }
                let new_song = self.shown_songs[self.index].clone();
                self.playlists[self.playlist_index].songs.push(new_song);
                if self.shown_playlist.is_some() {
                    self.shown_playlist = Some(self.playlists[self.playlist_index].clone());
                }
            }
            KeyCode::Left => {
                if self.playlists.len() == 0 {
                    return;
                }
                if self.shown_playlist.is_none() {
                    self.shown_playlist = Some(self.playlists[self.playlist_index].clone());
                    return;
                }
                self.shown_playlist = None;
                
            }
            KeyCode::Enter => {
                if self.shown_songs.len() == 0 {
                    return;
                }
                let new_song = self.shown_songs[self.index].clone();
                self.listening_songs.push_back(new_song);
            }
            KeyCode::Char(e) => {
                self.search.push(e);
                self.get_matching_songs();
                self.index = 0;
            }
            KeyCode::Backspace => {
                self.search.pop();
                self.get_matching_songs();
            }
            KeyCode::Down => {
                if self.index != self.shown_songs.len() - 1{
                    self.index += 1;
                }
            }
            KeyCode::Up => {
                if self.index != 0 {
                    self.index -= 1;
                }
            }
            _ => {}
        }
    }

    fn normal_handler(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Left => {
                if self.playlists.len() == 0 {
                    return;
                }
                let playlist = self.playlists[self.playlist_index].clone();
                for song in playlist.songs {
                    self.listening_songs.push_back(song);
                }
            }
            KeyCode::Char(' ') => {
                if self.sink.is_paused() {
                    self.sink.play();
                } else {
                    self.sink.pause();
                }
            }
            KeyCode::Delete => {
                self.sink.stop();
            }
            KeyCode::Down => {
                if self.volume == 0.0 {
                    return;
                }
                self.volume -= 0.1;
                self.sink.set_volume(self.volume);
            }
            KeyCode::Up => {
                if self.volume > 2.0 {
                    return;
                }
                self.volume += 0.1;
                self.sink.set_volume(self.volume);
            }
            _ => {}
        }
    }

    fn playlist_handler(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Left => {
                self.shown_playlist = None;
            }
            KeyCode::Backspace => {
                self.new_playlist_name.pop();
            }
            KeyCode::Char(c) => {
                if self.adding_playlist {
                    self.new_playlist_name.push(c);
                    return;
                }
                match c {
                    'n' => self.adding_playlist = true,
                    's' => serialize::serialize(&self.playlists, &self.glob_data.playlist_path),
                    _ => {}
                }
            }
            KeyCode::Enter => {
                if self.shown_playlist.is_some() {
                    for song in self.shown_playlist.clone().unwrap().songs {
                        self.listening_songs.push_back(song);
                    }
                    return;
                }

                if !self.adding_playlist && self.playlists.len() != 0{
                    self.shown_playlist = Some(self.playlists[self.playlist_index].clone())
                }
                if self.new_playlist_name.len() == 0 {
                    return;
                }
                let new_playlist = Playlist::new(self.new_playlist_name.clone());
                self.playlists.push(new_playlist);
                self.new_playlist_name = String::new();
                self.adding_playlist = false;
            }
            KeyCode::Up => {
                if self.playlist_index == 0 {
                    self.playlist_index = self.playlists.len() - 1;
                } else {
                    self.playlist_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.playlist_index == self.playlists.len() - 1 {
                    self.playlist_index = 0;
                } else {
                    self.playlist_index += 1;
                }
            }
            _ => {}
        }
    }
}