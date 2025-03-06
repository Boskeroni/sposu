# SPOSU

Inspired by the Spotify-tui project. Wanted to create a music player app but didn't want to download all the music again so I just use the songs from my osu! folder.

It reads the song path from the .osu files to locate the mp3, doesn't contain duplicate copies of the same file nor does it store songs locally. 
It supports playlists being made, edited, shuffled, and deleted. The speed of the songs can also be changed.

Programmed in Rust. Uses the Ratatui library for rendering the TUI interface, and cpal for audio processing.
