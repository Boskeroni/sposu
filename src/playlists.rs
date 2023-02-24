use serde::{Serialize, Deserialize};

use crate::osu::Song;



#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
    pub shuffle_on: bool,
    pub repeat_on: bool,

}
impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
            shuffle_on: false,
            repeat_on: false,
        }
    }
}