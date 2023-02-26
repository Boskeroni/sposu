use serde::{Serialize, Deserialize};
use crossterm::event::{KeyEvent, KeyCode};

use crate::osu::{Song, Mod};
use crate::player::{Playlist, Player, serialize_playlists, PlaybarSource};

#[derive(Copy, Clone, Debug)]
pub enum UIMode {
    Input,
    PlayBar,
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
    pub query: String,
    pub queried_songs: Vec<Song>,
    pub playlists: Vec<Playlist>,
    pub playlist_i: usize,
    pub query_i: usize,
    pub new_playlist_name: String,
    pub is_adding_list: bool,
    pub glob_data: AppData,
    pub player: Player,
    all_songs: Vec<Song>,
}

impl App {
    pub fn new(songs: Vec<Song>, glob_data: AppData, playlists: Vec<Playlist>) -> Self {
        let player = Player::new(glob_data.song_path.clone());
        Self {
            query: String::new(),
            all_songs: songs.clone(),
            queried_songs: songs,
            query_i: 0,
            current_ui: UIMode::PlayBar,
            playlists,
            playlist_i: 0,
            new_playlist_name: String::new(),
            is_adding_list: false,
            glob_data,
            player,
        }
    }

    /// HANDLES THE EVENTS WHICH MATTER REGARDLESS OF STATE
    pub fn event_handler(&mut self, key: &KeyEvent) -> Result<(), i32> {
        // handle the only bit that can error first
        if let KeyCode::Esc = key.code {
            if let UIMode::PlayBar = self.current_ui {
                match self.player.playbar_source {
                    PlaybarSource::Playlist => {},
                    _ => return Err(1)
                }
            }
        }

        // TAB is global thing which does the same thing
        if let KeyCode::Tab = key.code {
            match self.current_ui {
                UIMode::Input => self.current_ui = UIMode::Playlist,
                UIMode::Playlist => self.current_ui = UIMode::PlayBar,
                UIMode::PlayBar => self.current_ui = UIMode::Input,
            }
            return Ok(())
        }

        // relegate input to the other functions
        match self.current_ui {
            UIMode::Input => self.search_mode_handler(key),
            UIMode::Playlist => self.playlist_handler(key),
            UIMode::PlayBar => self.play_bar_handler(key)
        }

        return Ok(())
    }

    /// GETS ALL THE MATCHING SONGS FROM SEARCH QUERY
    pub fn get_matching_songs(&mut self) {
        self.queried_songs = vec![];
        for song in self.all_songs.clone() {
            if song.song_name.to_lowercase().contains(&self.query.to_lowercase()) {
                self.queried_songs.push(song);
            }
        }
    }

    /// HANDLES INPUT FOR SONG QUERY BLOCK
    fn search_mode_handler(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                if self.query.len() != 0 {
                    self.query = String::new();
                    self.get_matching_songs();
                    return;
                }
                self.current_ui = UIMode::PlayBar;
            }
            // adds song to playlsit
            KeyCode::Right => {
                if self.playlists.len() == 0 {
                    return;
                }
                if self.queried_songs.len() == 0 {
                    return;
                }
                let new_song = self.queried_songs[self.query_i].clone();
                self.playlists[self.playlist_i].songs.push(new_song);
                if self.player.current_playlist.is_some() {
                    self.player.current_playlist = Some(self.playlists[self.playlist_i].clone());
                }
            }
            // changes the mod of the song
            KeyCode::Left => {
                if self.queried_songs.len() == 0 {
                    return;
                }
                let current_mod = self.queried_songs[self.query_i].modifier;
                self.queried_songs[self.query_i].modifier = match current_mod {
                    Mod::NoMod => Mod::DoubleTime,
                    Mod::DoubleTime => Mod::Nightcore,
                    Mod::Nightcore => Mod::NoMod,
                }
            }
            // adds song to playbar
            KeyCode::Enter => {
                if self.queried_songs.len() == 0 {
                    return;
                }
                let new_song = self.queried_songs[self.query_i].clone();
                self.player.add_normal_song(new_song);
            }
            // adds char to search query
            KeyCode::Char(e) => {
                self.query.push(e);
                self.get_matching_songs();
                self.query_i = 0;
            }
            // backspace on search query
            KeyCode::Backspace => {
                self.query.pop();
                self.get_matching_songs();
            }
            // move down results
            KeyCode::Down => {
                if self.query_i != self.queried_songs.len() - 1{
                    self.query_i += 1;
                    return;
                }
                self.query_i = 0;
            }
            // move up results
            KeyCode::Up => {
                if self.query_i != 0 {
                    self.query_i -= 1;
                    return;
                }
                self.query_i = self.queried_songs.len() - 1;
            }
            _ => {}
        }
    }

    /// HANDLES INPUT FOR NOW PLAYING BLOCK
    fn play_bar_handler(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // if it makes it here, we know user wants to unload playlist
                self.player.unload_playbar_playlist(); 
            }
            // plays song currently being hovered
            KeyCode::Enter => self.player.force_new_song(),
            // pause / unpause song
            KeyCode::Char(' ') => {
                if self.player.sink.is_paused() {
                    self.player.sink.play();
                    return;
                }
                self.player.sink.pause();
            }
            // remove hovered song from playbar
            KeyCode::Delete => self.player.remove_hovered_song(),
            // move hover up playbar
            KeyCode::Up => {
                if self.player.current_songs.is_empty() {
                    return;
                }
                if self.player.hovered_index == 0 {
                    self.player.hovered_index = self.player.current_songs.len() - 1;
                    return;
                }
                self.player.hovered_index -= 1;
            }
            // move hover down playbar
            KeyCode::Down => {
                if self.player.current_songs.is_empty() {
                    return;
                }
                self.player.hovered_index += 1;
                if self.player.hovered_index == self.player.current_songs.len() {
                    self.player.hovered_index = 0;
                }
            }
            _ => {}
        }
    }

    /// HANDLES INPUT FOR PLAYLISTS
    fn playlist_handler(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // makes sense to have two ways to remove playlist
                if let PlaybarSource::Playlist = self.player.playbar_source {
                    self.player.unload_playbar_playlist();
                    return;
                }
                if self.player.current_playlist.is_some() {
                    self.player.pop_current_playlist();
                    return;
                }
                self.current_ui = UIMode::PlayBar;
            }
            // unloads the playlsit
            KeyCode::Left => {
                if self.player.current_playlist.is_some() {
                    self.playlists[self.playlist_i] = self.player.pop_current_playlist();
                }
            }
            // removes char from new playlist
            KeyCode::Backspace => {
                self.new_playlist_name.pop();
            }
            KeyCode::Char(c) => {
                // checks if char is for new name
                if self.is_adding_list {
                    self.new_playlist_name.push(c);
                    return;
                }
                // matches it to shortcuts
                match c {
                    'n' => self.is_adding_list = true,
                    'q' => serialize_playlists(&self.playlists, &self.glob_data.playlist_path.clone()),
                    'w' => {
                        // change the playlist stored in the player and return it to the vec when finished.
                        if let Some(mut p) = self.player.current_playlist.clone() {
                            p.toggle_repeat();
                            self.player.current_playlist = Some(p);
                        }
                    }
                    'e' => {
                        if let Some(mut p) = self.player.current_playlist.clone() {
                            p.toggle_shuffle();
                            self.player.current_playlist = Some(p);
                        }
                    },
                    _ => {}
                }
            }
            KeyCode::Enter => {
                // loads current playlist into now playing bar
                if self.player.current_playlist.is_some() {
                    self.player.load_playbar_playlist();
                    return;
                }
                // loads song to player
                if !self.is_adding_list && self.playlists.len() != 0{
                    self.player.load_playlist(self.playlists[self.playlist_i].clone());
                }
                // cant make a new list without a name
                if self.new_playlist_name.len() == 0 {
                    return;
                }

                // creates the playlist
                let new_playlist = Playlist::new_empty(self.new_playlist_name.clone());
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