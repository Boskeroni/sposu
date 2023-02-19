use std::collections::VecDeque;

use rodio::{Sink, OutputStreamHandle, OutputStream};
use serde::{Serialize, Deserialize};
use std::fs::File;
use crossterm::event::{KeyEvent, KeyCode};

use crate::osu::Song;

pub enum InputMode {
    Input,
    Normal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    pub song_path: String,
    pub is_serialized: bool,
    pub serialize_path: String,
    pub playlist_path: String
}

pub struct App {
    pub input_mode: InputMode,
    pub sink: Sink,
    pub search: String,
    pub shown_songs: Vec<Song>,
    pub listening_songs: VecDeque<Song>,
    pub playlist: Vec<Song>,
    pub playlist_index: usize,
    pub index: usize,
    all_songs: Vec<Song>,
    stream_handle: OutputStreamHandle,
    _stream: OutputStream,
    is_playing: bool,
    volume: f32,
}

impl App {
    pub fn new(songs: Vec<Song>) -> Self {
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
            input_mode: InputMode::Normal,
            volume: 1.0,
            playlist: Vec::new(),
            playlist_index: 0,
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

        let audio_file = File::open(self.listening_songs[0].audio_path.clone()).unwrap();
        self.sink = self.stream_handle.play_once(audio_file).unwrap();
        self.sink.set_volume(self.volume);
        self.is_playing = true;
    }

    pub fn input_mode_handler(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                if self.shown_songs.len() == 0 {
                    return;
                }
                let new_song = self.shown_songs[self.index].clone();
                self.playlist.push(new_song);
            }
            KeyCode::Enter => {
                if self.shown_songs.len() == 0 {
                    return;
                }
                let new_song = self.shown_songs[self.index].clone();
                self.listening_songs.push_back(new_song);
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal
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

    pub fn normal_mode_handler(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('i') => self.input_mode = InputMode::Input,
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
}
