use std::collections::VecDeque;

use rodio::{Sink, OutputStreamHandle, OutputStream};
use serde::{Serialize, Deserialize};
use std::fs::File;
use crossterm::event::{KeyEvent, KeyCode};

use crate::osu::{Song, Mod};
use crate::playlists::Playlist;
use crate::serialize;

#[derive(Copy, Clone, Debug)]
pub enum UIMode {
    Input,
    SongQueue,
    Playlist,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    pub song_path: String,
    pub serialized_songs: bool,
    pub serialize_path: String,
    pub playlist_path: String,
}

pub struct App {
    pub current_ui: UIMode,
    pub sink: Sink,
    pub query: String,
    pub displayed_songs: Vec<Song>,
    pub now_playing: VecDeque<Song>,
    pub playlists: Vec<Playlist>,
    pub playlist_i: usize,
    pub query_i: usize,
    pub new_playlist_name: String,
    pub current_playlist: Option<Playlist>,
    pub is_adding_list: bool,
    pub playing_bar_i: usize,
    pub currently_playing_i: usize,
    all_songs: Vec<Song>,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    is_playing: bool,
    volume: f32,
    glob_data: AppData,
}

impl App {
    pub fn new(songs: Vec<Song>, glob_data: AppData, playlists: Vec<Playlist>) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::new_idle().0;
        Self {
            query: String::new(),
            all_songs: songs.clone(),
            sink,
            stream_handle,
            displayed_songs: songs,
            now_playing: VecDeque::new(),
            query_i: 0,
            is_playing: false,
            current_ui: UIMode::SongQueue,
            volume: 0.5,
            playlists,
            playlist_i: 0,
            new_playlist_name: String::new(),
            current_playlist: None,
            is_adding_list: false,
            glob_data,
            playing_bar_i: 0,
            currently_playing_i: 0,
            _stream
        }
    }
    
    /// CONVERTS THE FILE PATH INTO A VALID SONG PATH
    pub fn to_valid_path(&self, file: &str) -> String {
        format!("{}/{}", self.glob_data.song_path, file)
    }

    /// HANDLES THE EVENTS WHICH MATTER REGARDLESS OF STATE
    pub fn global_handler(&mut self, key: &KeyEvent) -> Result<(), i32> {
        match key.code {
            KeyCode::Tab => {
                match self.current_ui {
                    UIMode::Input => self.current_ui = UIMode::Playlist,
                    UIMode::Playlist => self.current_ui = UIMode::SongQueue,
                    UIMode::SongQueue => self.current_ui = UIMode::Input,
                }
            }
            KeyCode::Esc => {
                match self.current_ui {
                    UIMode::SongQueue => return Err(1),
                    _ => self.current_ui = UIMode::SongQueue
                }
            }
            _ => {}
        }
        return Ok(())
    }

    /// RELEGATES THE OTHER EVENTS TO THEIR SPECIFIC STATES
    pub fn specific_handler(&mut self, key: &KeyEvent) {
        match self.current_ui {
            UIMode::Input => {
                self.input_handler(key);
            }
            UIMode::SongQueue => {
                match key.code {
                    KeyCode::Esc => {},
                    _ => self.play_now_handler(key)
                }
            }
            UIMode::Playlist => {
                self.playlist_handler(key);
            }
        }
    }

    /// GETS ALL THE MATCHING SONGS FROM SEARCH QUERY
    pub fn get_matching_songs(&mut self) {
        self.displayed_songs = vec![];
        for song in self.all_songs.clone() {
            if song.song_name.to_lowercase().contains(&self.query.to_lowercase()) {
                self.displayed_songs.push(song);
            }
        }
    }

    /// PLAYS THE CURRENTLY SELECTED SONG
    pub fn play_selected_song(&mut self) {
        if self.is_playing {
            self.now_playing.remove(self.currently_playing_i);
            self.playing_bar_i -= 1;
        }
        if self.now_playing.len() == 0 {
            self.is_playing = false;
            return;
        }
        let next_song = self.now_playing[self.playing_bar_i].clone();
        self.currently_playing_i = self.playing_bar_i;

        let (speed, _amp) = match next_song.modifier {
            Mod::Nightcore => (1.5, 1.0),
            Mod::NoMod => (1.0, 1.0),
            Mod::DoubleTime => (1.5, 0.66),
        };

        let path = format!("{}{}", &self.glob_data.song_path, &next_song.audio_path);
        let audio_file = File::open(path).unwrap();

        
        self.sink = self.stream_handle.play_once(audio_file).unwrap();

        self.sink.set_volume(self.volume);
        self.sink.set_speed(speed);
        self.is_playing = true;

    }

    /// HANDLES INPUT FOR SONG QUERY BLOCK
    fn input_handler(&mut self, key: &KeyEvent) {
        match key.code {
            // adds new song to the playlist
            KeyCode::Right => {
                if self.playlists.len() == 0 {
                    return;
                }
                if self.displayed_songs.len() == 0 {
                    return;
                }
                let new_song = self.displayed_songs[self.query_i].clone();
                self.playlists[self.playlist_i].songs.push(new_song);
                if self.current_playlist.is_some() {
                    self.current_playlist = Some(self.playlists[self.playlist_i].clone());
                }
            }
            KeyCode::Left => {
                if self.displayed_songs.len() == 0 {
                    return;
                }
                let current_mod = self.displayed_songs[self.query_i].modifier;
                self.displayed_songs[self.query_i].modifier = match current_mod {
                    Mod::NoMod => Mod::DoubleTime,
                    Mod::DoubleTime => Mod::Nightcore,
                    Mod::Nightcore => Mod::NoMod,
                }
            }
            KeyCode::Enter => {
                if self.displayed_songs.len() == 0 {
                    return;
                }
                let new_song = self.displayed_songs[self.query_i].clone();
                self.now_playing.push_back(new_song);
            }
            KeyCode::Char(e) => {
                self.query.push(e);
                self.get_matching_songs();
                self.query_i = 0;
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.get_matching_songs();
            }
            KeyCode::Down => {
                if self.query_i != self.displayed_songs.len() - 1{
                    self.query_i += 1;
                }
            }
            KeyCode::Up => {
                if self.query_i != 0 {
                    self.query_i -= 1;
                }
            }
            _ => {}
        }
    }

    /// HANDLES INPUT FOR NOW PLAYING BLOCK
    fn play_now_handler(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Left => {
                if self.playlists.len() == 0 {
                    return;
                }
                let playlist = self.playlists[self.playlist_i].clone();
                for song in playlist.songs {
                    self.now_playing.push_back(song);
                }
            }
            KeyCode::Enter => {
                if self.playing_bar_i == self.currently_playing_i {
                    return;
                }
                self.sink.stop();
                self.play_selected_song();
            }

            KeyCode::Char(' ') => {
                if self.sink.is_paused() {
                    self.sink.play();
                    return;
                }
                self.sink.pause();
            }
            KeyCode::Delete => {
                if self.playing_bar_i == self.currently_playing_i {
                    self.sink.stop();
                    return;
                }
                self.now_playing.remove(self.playing_bar_i);
            }
            KeyCode::Up => {
                if self.playing_bar_i == 0 {
                    self.playing_bar_i = self.now_playing.len();
                } else {
                    self.playing_bar_i -= 1;
                }
            }
            KeyCode::Down => {
                if self.playing_bar_i == self.now_playing.len() {
                    self.playing_bar_i = 0;
                    return;
                }
                self.playing_bar_i += 1;
            }
            _ => {}
        }
    }

    /// HANDLES INPUT FOR PLAYLISTS
    fn playlist_handler(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Left => {
                self.current_playlist = if self.current_playlist.is_none() {
                    Some(self.playlists[self.playlist_i].clone())
                } else {
                    None
                };
            }
            KeyCode::Backspace => {
                self.new_playlist_name.pop();
            }
            KeyCode::Char(c) => {
                if self.is_adding_list {
                    self.new_playlist_name.push(c);
                    return;
                }
                match c {
                    'n' => self.is_adding_list = true,
                    'q' => serialize::serialize(&self.playlists, &self.glob_data.playlist_path),
                    'w' => {
                        if self.current_playlist.is_some() {
                            self.playlists[self.playlist_i].repeat_on = !self.playlists[self.playlist_i].repeat_on;
                        }
                    }
                    'e' => {
                        if self.current_playlist.is_some() {
                            self.playlists[self.playlist_i].shuffle_on = self.playlists[self.playlist_i].shuffle_on;
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => {
                if self.current_playlist.is_some() {
                    for song in self.current_playlist.clone().unwrap().songs {
                        self.now_playing.push_back(song);
                    }
                    return;
                }

                if !self.is_adding_list && self.playlists.len() != 0{
                    self.current_playlist = Some(self.playlists[self.playlist_i].clone())
                }
                if self.new_playlist_name.len() == 0 {
                    return;
                }
                let new_playlist = Playlist::new(self.new_playlist_name.clone());
                self.playlists.push(new_playlist);
                self.new_playlist_name = String::new();
                self.is_adding_list = false;
            }
            KeyCode::Up => {
                if self.playlist_i == 0 {
                    self.playlist_i = self.playlists.len() - 1;
                } else {
                    self.playlist_i -= 1;
                }
            }
            KeyCode::Down => {
                if self.playlist_i == self.playlists.len() - 1 {
                    self.playlist_i = 0;
                } else {
                    self.playlist_i += 1;
                }
            }
            _ => {}
        }
    }
}