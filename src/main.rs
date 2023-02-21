use std::thread;
use std::time::Duration;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::{EnableMouseCapture, Event, self, KeyCode, DisableMouseCapture};
use crossterm::execute;
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;
use std::io;

mod osu;
mod renderer;
mod app;
mod serialize;

use app::*;

fn main() -> Result<(), io::Error>{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let global_data: AppData = serialize::deserialize::<AppData>("assets/data.json");

    let songs = match global_data.is_serialized {
        true => serialize::deserialize(&global_data.serialize_path),
        false => {
            let songs = osu::load_songs(&global_data.song_path);
            serialize::serialize(&songs, &global_data.serialize_path);
            songs
        }
    };

    let app = App::new(songs, global_data);
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
        terminal.draw(|f| renderer::render(f, &app))?;
        thread::sleep(Duration::from_nanos(1));
    }
}

