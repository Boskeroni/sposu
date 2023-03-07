use std::fs::File;

use rand::Rng;
use rodio::{Sink, OutputStream, OutputStreamHandle};
use serde::{Deserialize, Serialize};
use crate::{osu::{Song, Mod}, serialize::{deserialize, self}};

/// where the playbar is getting its songs from
/// NONE => not playing rn
/// PLAYLIST => from playlist
/// NORMAL => from search part
pub enum PlaybarSource {
    None,
    Playlist,
    Normal,
}

pub struct Player<'a> {
    // rodio
    pub sink: Sink,
    pub _stream: OutputStream,
    pub stream_handle: OutputStreamHandle,

    // playbar
    pub playbar_source: PlaybarSource,
    pub current_songs: Vec<&'a Song>,
    pub current_playlist: Option<&'a Playlist<'a>>,
    pub hovered_index: usize,
    pub parent_path: String,
    pub playing_index: usize,
    pub volume: f32,
    pub is_playing: bool,
}

impl Player<'_> {
    pub fn new(parent_path: String) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::new_idle().0;
        let currently_playing = PlaybarSource::None;
        Self {
            sink,
            _stream,
            stream_handle,
            playbar_source: currently_playing,
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
        // the playing song is automatically removed if the sink stops
        if self.playing_index == self.hovered_index {
            self.sink.stop();
            // playlist songs have to be manually removed
            if let PlaybarSource::Playlist = self.playbar_source {
                self.current_playlist.as_mut().unwrap().remove_playbar_song(self.hovered_index);
                self.current_songs.remove(self.hovered_index);

            }
        } else {
            self.current_playlist.as_mut().unwrap().remove_playbar_song(self.hovered_index);
            self.current_songs.remove(self.hovered_index);
        }

        if self.hovered_index == self.current_songs.len() {
            self.hovered_index -= 1;
        }
    }

    // removes the playlist from the whole struct
    pub fn pop_current_playlist(&mut self) -> &Playlist {
        self.playbar_source = PlaybarSource::None;
        let temp = self.current_playlist.unwrap();
        self.current_playlist = None;
        &temp
    }

    // loads the playlist
    pub fn load_playlist(&mut self, playlist: &Playlist) {
        self.current_playlist = Some(playlist);
    }

    // loads playlist into paybar
    pub fn load_playbar_playlist(&mut self) {
        // don't do anything if the list is empty
        if self.current_playlist.unwrap().songs.is_empty() {
            return;
        }

        self.current_songs = Vec::new();
        self.sink.stop();
        self.current_playlist.unwrap().prepare_playlist();
        self.current_songs.append(&mut self.current_playlist.unwrap().songs);
        self.playbar_source = PlaybarSource::Playlist;
        self.playing_index = 0;
    }

    // removes the playlist from the playbar
    pub fn unload_playbar_playlist(&mut self) {
        self.current_songs = Vec::new();
        self.playbar_source = PlaybarSource::None;
        self.sink.stop();
    }

    // adds regular song
    pub fn add_normal_song(&mut self, song: &Song) {
        self.current_songs.push(song);
        self.playbar_source = PlaybarSource::Normal;
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

        let new_song = match self.playbar_source {
            PlaybarSource::Playlist => {
                let data = self.current_playlist.clone().unwrap().get_next_song();
                if data.is_none() { return; }
                
                let (song, index) = data.unwrap();
                self.playing_index = index;
                song
            }
            PlaybarSource::Normal => {
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

                self.current_songs[self.playing_index]
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
pub struct Playlist<'a> {
    pub name: String,
    pub songs: Vec<&'a Song>,
    pub shuffle_on: bool,
    pub repeat_on: bool,
    song_index: usize,
    song_choice: Vec<&'a Song>,
    removed_played: bool,
}

impl Playlist<'_> {
    pub fn new_empty(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
            shuffle_on: false,
            repeat_on: false,
            song_index: 0,
            song_choice: Vec::new(),
            removed_played: false,
        }
    }

    pub fn new_loaded(serialized: &SerializedPlaylist) -> Self {
        Self {
            name: serialized.name.clone(),
            songs: serialized.songs.iter().collect(),
            shuffle_on: false,
            repeat_on: false,
            song_index: 0,
            song_choice: Vec::new(),
            removed_played: false,
        }
    }

    pub fn prepare_playlist(&mut self) {
        self.song_choice = self.songs;
    }

    pub fn remove_playbar_song(&mut self, index: usize) {
        self.song_choice.remove(index);
        if self.song_index == index {
            self.removed_played = true;
        }
    }

    pub fn get_next_song(&mut self) -> Option<(&Song, usize)> {
        if self.shuffle_on {
            let song_index = rand::thread_rng().gen_range(0..self.song_choice.len()-1);
            let new_song = self.song_choice[song_index];
            return Some((new_song, song_index));
        }
        if self.removed_played {
            self.removed_played = false;
            return Some((self.song_choice[self.song_index], self.song_index));
        }
        self.song_index += 1;
        if self.song_index < self.song_choice.len() {
            return Some((self.song_choice[self.song_index-1], self.song_index-1));
        }

        self.song_index = 0;
        if self.repeat_on {
            return Some((self.song_choice[self.song_index], self.song_index));
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
            name: p.name,
            songs: p.songs.iter().map(|s| **s).collect()
        }
    }).collect();

    serialize::serialize(&raw_data, path);
}

pub fn deserialize_playlist(path: &str) -> Option<Vec<Playlist>> {
    let raw_data: Result<Vec<SerializedPlaylist>, _> = deserialize(path);
    if raw_data.is_err() {
        return None;
    }
    let playlist_data = raw_data.unwrap();
    let playlists = playlist_data.iter().map(|p| {
        Playlist::new_loaded(p)
    }).collect();
    Some(playlists)
}
