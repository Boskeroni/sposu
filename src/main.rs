use std::{thread, fs};
use std::time::Duration;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::{EnableMouseCapture, Event, self, KeyCode, DisableMouseCapture};
use crossterm::execute;
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;
use std::io;

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
                if let Err(_) = app.global_handler(&key) {
                    return Ok(())
                };
                app.specific_handler(&key);
                match key.code {
                    KeyCode::Tab => {
                        app.input_mode_stack.pop();
                        match app.current_mode {
                            UIMode::Input => app.current_mode = UIMode::NewPlaylist,
                            UIMode::NewPlaylist => app.current_mode = UIMode::Normal,
                            UIMode::Normal => app.current_mode = UIMode::Input,
                        }
                        app.input_mode_stack.push(app.current_mode);
                    }
                    KeyCode::Esc => {
                        let popped_mode = app.input_mode_stack.pop().unwrap();
                        match popped_mode {
                            UIMode::Normal => return Ok(()),
                            _ => app.current_mode = popped_mode
                        }
                    }
                    _ => {

                    }
                }
            }
        }
        terminal.draw(|f| ui::render(f, &app))?;
        thread::sleep(Duration::from_nanos(1));
    }
}

