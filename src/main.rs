use std::thread;
use std::time::Duration;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::{EnableMouseCapture, Event, self, DisableMouseCapture};
use crossterm::execute;
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;
use std::io;

mod osu;
mod renderer;
mod app;
mod serialize;
mod playlists;

use app::*;

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

    let global_data: AppData = serialize::deserialize::<AppData>("assets/data.json").unwrap();

    let songs = match global_data.serialized_songs {
        true => serialize::deserialize(&global_data.serialize_path).unwrap(),
        false => {
            let songs = osu::load_songs(&global_data.song_path);
            serialize::serialize(&songs, &global_data.serialize_path);
            songs
        }
    };

    let playlists = serialize::deserialize(&global_data.playlist_path).unwrap();

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
        if app.sink.empty() {
            app.play_selected_song()
        }
        if crossterm::event::poll(Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                if let Err(_) = app.global_handler(&key) {
                    return Ok(())
                };
                app.specific_handler(&key);
            }
        }
        terminal.draw(|f| renderer::render(f, &app))?;
        thread::sleep(Duration::from_nanos(1));
    }
}

