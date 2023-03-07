use std::io;
use std::time::Duration;
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::{EnableMouseCapture, Event, self, DisableMouseCapture};
use tui::Terminal;
use tui::backend::{Backend, CrosstermBackend};

mod osu;
mod renderer;
mod app;
mod serialize;
mod player;

use app::*;
use osu::Song;
use player::deserialize_playlist;

fn load_songs(data: &mut AppData) -> Vec<Song> {
    if data.serialized_songs {
        let pot_data = serialize::deserialize(&data.serialize_path);
        if pot_data.is_ok() {
            return pot_data.unwrap();
        }
    }
    let songs = osu::load_all_songs(&data.song_path);
    serialize::serialize(&songs, &data.serialize_path);
    data.serialized_songs = true;
    serialize::serialize(data, "assets/data.json");
    songs
}

/// INITIALISES APP
fn main() -> Result<(), io::Error>{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout, 
        EnterAlternateScreen, 
        EnableMouseCapture,
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut global_data: AppData = serialize::deserialize::<AppData>("assets/data.json").unwrap();
    let songs = load_songs(&mut global_data);
    let playlists = deserialize_playlist(&global_data.playlist_path.clone()).unwrap();

    let app = App::new(songs, global_data, playlists);
    let res = main_loop(&mut terminal, app);
    
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    if let Err(e) = res {
        println!("{:?}", e);
    }

    Ok(())
}

/// RUNS THE APP
fn main_loop<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| renderer::render(f, &app))?;
        if app.player.sink.empty() {
            app.player.try_new_song()
        }
        if !crossterm::event::poll(Duration::from_millis(20))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            app.event_handler(&key).unwrap();
        }
    }
}

