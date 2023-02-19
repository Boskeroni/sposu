use std::{thread, fs};
use std::time::Duration;
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, 
    event::{EnableMouseCapture, Event, self, KeyCode, DisableMouseCapture},
    execute
};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal
};

mod osu;
mod ui;
mod app;

use app::*;

fn main() -> Result<(), io::Error>{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let global_data: AppData = serde_json::from_str(&fs::read_to_string("assets/data.json").unwrap()).unwrap();

    let songs = if global_data.is_serialized {
        osu::deserialize_osu_files(&global_data.serialize_path)
    } else {
        osu::load_songs(&global_data.song_path)
    };

    let app = App::new(songs);
    let res = main_loop(&mut terminal, app);
    
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    if let Err(e) = res {
        println!("{:?}", e);
    }

    Ok(())
}

fn main_loop<B: Backend>(
    terminal: &mut Terminal<B>, 
    mut app: App,
) -> io::Result<()> {
    loop {
        if app.sink.empty() {
            app.play_next_song()
        }
        if crossterm::event::poll(Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Input => {
                        app.input_mode_handler(key);
                    }
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Esc => return Ok(()),
                            _ => app.normal_mode_handler(key)
                        }
                    }
                }
                
            }
        }
        terminal.draw(|f| ui::render(f, &app))?;
        thread::sleep(Duration::from_nanos(1));
    }
}

