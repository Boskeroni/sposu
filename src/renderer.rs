use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Layout, Direction, Constraint, Rect};
use tui::widgets::{Paragraph, Borders, BorderType, Block, ListItem, List};
use tui::style::{Style, Color, Modifier};
use tui::text::{Span, Text, Spans};
use unicode_width::UnicodeWidthStr;

use crate::osu::Mod;
use crate::{App, UIMode};

pub fn render<B: Backend>(f: &mut Frame<B>, app: &App) {
    let all_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ]).split(f.size());
    
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ]).split(all_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(5),
            Constraint::Percentage(70),
        ]).split(all_chunks[1]);

    // renders the input block
    song_input(app, f, left_chunks[0]);

    // renders the shown songs block
    song_search(app, f, left_chunks[1]);
    
    // renders the plan to listen bit
    listening_to(app, f, right_chunks[0]);

    // renders the information for the songs
    song_info(app, f, right_chunks[1]);

    // renders the playlist block including new bar if needed
    if app.adding_playlist { new_playlist_ui(app, f, right_chunks[2]); } 
    else { basic_playlist(app, f, right_chunks[2]); }

    // sets the cursor to the input field
    f.set_cursor(left_chunks[1].x + app.search.width() as u16 + 1, left_chunks[0].y + 1);
}


/// RENDERS THE SEARCH BAR FOR THE SONGS
fn song_input<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let paragraph_style = match app.current_mode {
        UIMode::Input => Style::default().fg(Color::Yellow),
        _ => Style::default()
    };

    let input_block = Paragraph::new(app.search.as_ref())
        .style(paragraph_style)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(input_block, area);
}

/// RENDERS THE RESULTS FROM THE SEARCH BAR
fn song_search<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let list_items: Vec<ListItem> = app.shown_songs.iter().enumerate().map(|(i, song)| {
        let (chosen_symbol, mod_text) = if i == app.index {
            let mod_text = if let UIMode::Input = app.current_mode {
                match song.modifier {
                    Mod::NoMod => "<NoMod>",
                    Mod::DoubleTime => "<DoubleTime>",
                    Mod::Nightcore => "<Nightcore>",
                }
            } else {
                " "
            };

            (">", mod_text)
        } else {
            (" ", " ")
        };
        let line_content = vec![
            Span::styled(chosen_symbol, Style::default().fg(Color::Red)),
            Span::raw("|"),
            Span::styled(&song.artist, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": "),
            Span::styled(&song.song_name, Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled(mod_text, Style::default().fg(Color::Cyan))
        ];
        ListItem::new(Text::from(Spans::from(line_content)))
    }).collect();

    let style = match app.current_mode {
        UIMode::Input => Style::default().add_modifier(Modifier::ITALIC).fg(Color::Yellow),
        _ => Style::default().add_modifier(Modifier::ITALIC)
    };

    let shown_songs_block = List::new(list_items)
        .block(Block::default().style(style).title("songs").borders(Borders::ALL).border_type(BorderType::Rounded))
        .highlight_symbol(">>");
    f.render_widget(shown_songs_block, area);
}

/// RENDERS THE CURRENTLY LISTENING TO BIT
fn listening_to<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let playlist_items: Vec<ListItem> = app.listening_songs.iter().enumerate().map(|(i, s)| {
        let style = if i == 0 {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };
        let content = vec![
            Span::styled(&s.song_name, style),
        ];
        ListItem::new(Text::from(Spans::from(content)))
    }).collect();

    let playlist_block = List::new(playlist_items)
        .block(Block::default().title("currently playing").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(playlist_block, area);
}

/// RENDERS THE SONG INFORMATION
fn song_info<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let to_raw_listitem = |i| {ListItem::new(Text::from(Spans::from(Span::raw(i))))};
    
    let content = if app.shown_songs.len() == 0 {
        vec![to_raw_listitem(format!("No song selected"))]
    } else {

        let song = app.shown_songs[app.index].clone();
        let path = app.to_valid_path(&song.audio_path);
        let length_mult = match song.modifier { 
            Mod::NoMod => 1.0,
            _ => 1.5,
        };
        
        let file_length = (mp3_duration::from_path(path).unwrap().as_secs() as f32 / length_mult) as usize;
        let formatted_secs = if file_length%60 < 10 {
            format!("0{}", file_length%60)
        } else {
            format!("{}", file_length%60)
        };
        let formatted_length = format!("{}:{}", file_length/60, formatted_secs);
        vec![
            to_raw_listitem(format!("TITLE: {}", &song.song_name)),
            to_raw_listitem(format!("ARTIST: {}", &song.artist)),
            to_raw_listitem(format!("LENGTH: {}", &formatted_length)),
        ]
    };
    let info_block = List::new(content)
        .block(
            Block::default()
            .title("info")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
        );
    f.render_widget(info_block, area);
}

/// RENDERS THE PLAYLIST WHEN NEW INPUT IS ADDED
fn new_playlist_ui<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let playlist_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(70),
        ]).split(area);
    
    let input_block = Paragraph::new(app.new_playlist_name.clone())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("new playlist name"));
    f.render_widget(input_block, playlist_chunk[0]);
    basic_playlist(app, f, playlist_chunk[1]);
}

/// RENDERS THE PLAYLIST OTHERWISE, SOMETIMES DOES SONGS INSIDE PLAYLIST
fn basic_playlist<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let style = match app.current_mode {
        UIMode::Playlist => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };

    let title = if app.shown_playlist.is_some() {
        app.shown_playlist.clone().unwrap().name
    } else {
        "playlists".to_string()
    };

    // renders when shown the playlist's songs
    let content: Vec<String> = if app.shown_playlist.is_some() {
        let mut names = Vec::new();
        for song in app.shown_playlist.clone().unwrap().songs {
            names.push(song.song_name.clone());
        }
        names
    } else {
        let mut names = Vec::new();
        for playlist in app.playlists.clone() {
            names.push(playlist.name.clone())
        }
        names
    };

    let list_items: Vec<ListItem> = content.iter().enumerate().map(|(i, s)| {
        let content = if i == app.playlist_index && app.shown_playlist.is_none() {
            Span::styled(s, Style::default().fg(Color::Red))
        } else {
            Span::raw(s)
        };

        ListItem::new(Text::from(Spans::from(content)))
    }).collect();

    let playlist_block = List::new(list_items)
        .block(Block::default()
            .style(style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
        );

    f.render_widget(playlist_block, area)
}
