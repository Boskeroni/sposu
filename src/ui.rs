use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Layout, Direction, Constraint, Rect};
use tui::widgets::{Paragraph, Borders, BorderType, Block, ListItem, List};
use tui::style::{Style, Color, Modifier};
use tui::text::{Span, Text, Spans};
use unicode_width::UnicodeWidthStr;

use crate::{App, InputMode};

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

    match app.input_mode {
        InputMode::NewPlaylist => new_playlist_ui(app, f, right_chunks[2]),
        _ => basic_playlist(app, f, right_chunks[2])
    }

    // renders the input block
    song_input(app, f, left_chunks[0]);

    // renders the shown songs block
    song_search(app, f, left_chunks[1]);
    
    // renders the plan to listen bit
    listening_to(app, f, right_chunks[0]);

    // renders the information for the songs
    song_info(app, f, right_chunks[1]);

    // sets the cursor to the input field
    f.set_cursor(left_chunks[1].x + app.search.width() as u16 + 1, left_chunks[0].y + 1);
}

fn song_input<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let paragraph_style = match app.input_mode {
        InputMode::Input => Style::default().fg(Color::Yellow),
        _ => Style::default()
    };

    let input_block = Paragraph::new(app.search.as_ref())
        .style(paragraph_style)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(input_block, area);
}

fn song_search<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let list_items: Vec<ListItem> = app.shown_songs.iter().enumerate().map(|(i, song)| {
        let chosen_symbol = if i == app.index {
            ">"
        } else {
            " "
        };
        let line_content = vec![
            Span::styled(chosen_symbol, Style::default().fg(Color::Red)),
            Span::raw("|"),
            Span::styled(&song.artist, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": "),
            Span::styled(&song.song_name, Style::default().fg(Color::Green)),
        ];
        ListItem::new(Text::from(Spans::from(line_content)))
    }).collect();

    let shown_songs_block = List::new(list_items)
        .block(Block::default().title("songs").borders(Borders::ALL).border_type(BorderType::Rounded))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
    f.render_widget(shown_songs_block, area);
}

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

fn song_info<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let to_raw_listitem = |i| {ListItem::new(Text::from(Spans::from(Span::raw(i))))};
    
    let content = if app.shown_songs.len() == 0 {
        vec![to_raw_listitem(format!("No song selected"))]
    } else {
        let selected_song = app.shown_songs[app.index].clone();
        let file_length = mp3_duration::from_path(&selected_song.audio_path).unwrap().as_secs();
        let formatted_secs = if file_length%60 < 10 {
            format!("0{}", file_length%60)
        } else {
            format!("{}", file_length%60)
        };
        let formatted_length = format!("{}:{}", file_length/60, formatted_secs);
        vec![
            to_raw_listitem(format!("TITLE: {}", &selected_song.song_name)),
            to_raw_listitem(format!("ARTIST: {}", &selected_song.artist)),
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

fn new_playlist_ui<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let playlist_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ]).split(area);
    
    let input_block = Paragraph::new(app.new_playlist_name.clone())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("new playlist name"));
    f.render_widget(input_block, playlist_chunk[0]);
    basic_playlist(app, f, playlist_chunk[1]);
}

fn basic_playlist<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    if let Some(playlist) = &app.shown_playlist {
        let playlist_items: Vec<ListItem> = playlist.songs.iter().map(|s| {
            let content = Span::styled(&s.song_name, Style::default().fg(Color::Cyan));
            ListItem::new(Text::from(Spans::from(content)))
        }).collect();
        let playlist_block = List::new(playlist_items)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("playlists"));

        f.render_widget(playlist_block, area);
        return
    }
    let playlist_items: Vec<ListItem> = app.playlists.iter().enumerate().map(|(i, s)| {
        let content = if i == app.playlist_index {
            Span::styled(&s.name, Style::default().fg(Color::Red))
        } else {
            Span::raw(&s.name)
        };

        ListItem::new(Text::from(Spans::from(content)))
    }).collect();

    let playlist_block = List::new(playlist_items)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("playlists"));

    f.render_widget(playlist_block, area)
}
