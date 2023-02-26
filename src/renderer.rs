use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Layout, Direction, Constraint, Rect};
use tui::widgets::{Paragraph, Borders, BorderType, Block, ListItem, List};
use tui::style::{Style, Color, Modifier};
use tui::text::{Span, Text, Spans};
use unicode_width::UnicodeWidthStr;

use crate::osu::Mod;
use crate::{App, UIMode};

/// THE GLOBAL RENDERER THAT CALLS EVERYTHING ELSE
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
    playbar(app, f, right_chunks[0]);

    // renders the information for the songs
    song_info(app, f, right_chunks[1]);

    // renders the playlist block including new bar if needed
    match app.is_adding_list {
        true => new_playlist_ui(app, f, right_chunks[2]),
        false => basic_playlist(app, f, right_chunks[2]),
    }

    // sets the cursor to the input field
    f.set_cursor(left_chunks[1].x + app.query.width() as u16 + 1, left_chunks[0].y + 1);
}


/// RENDERS THE SEARCH BAR FOR THE SONGS
fn song_input<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let paragraph_style = match app.current_ui {
        UIMode::Input => Style::default().fg(Color::Yellow),
        _ => Style::default()
    };

    let input_block = Paragraph::new(app.query.as_ref())
        .style(paragraph_style)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(input_block, area);
}

fn song_range(rows: usize, index: usize, songs_len: usize) -> (usize, usize) {
    // if all of the songs can be displayed normally, dont bother
    if rows > songs_len {
        return (0, songs_len)
    }

    // the index isnt big enough to warrant shifting the bottom
    if index < rows / 2 {
        return (0, std::cmp::min(rows, songs_len))
    }

    // the range has to be shifted
    let height = std::cmp::min(index + (rows/2), songs_len);
    return (index - (rows / 2), height)
    
}

/// RENDERS THE RESULTS FROM THE SEARCH BAR
fn song_search<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let rows = area.height - 2;
    let (bottom, top) = song_range(rows as usize, app.query_i, app.queried_songs.len());

    let used_vec = &app.queried_songs[bottom..top];
    let new_query_i = app.query_i - bottom;

    let list_items: Vec<ListItem> = used_vec.iter().enumerate().map(|(i, song)| {
        let (chosen_symbol, line_style) = 
            if new_query_i == i {
                (">", Style::default().fg(Color::Red))
            } else {
                (" ", Style::default())
            };
        let mod_text = match song.modifier {
            Mod::NoMod => "<NM>",
            Mod::DoubleTime => "<DT>",
            Mod::Nightcore => "<NC>"
        };
        let line_content = vec![
            Span::styled(chosen_symbol, Style::default().fg(Color::Red)),
            Span::raw("|"),
            Span::styled(mod_text, Style::default().fg(Color::Cyan)),
            Span::raw("|"),
            Span::styled(&song.artist, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": "),
            Span::styled(&song.song_name, Style::default().fg(Color::Green)),
            Span::raw(" "),
        ];
        ListItem::new(Text::from(Spans::from(line_content))).style(line_style)
    }).collect();

    let style = match app.current_ui {
        UIMode::Input => Style::default().add_modifier(Modifier::ITALIC).fg(Color::Yellow),
        _ => Style::default().add_modifier(Modifier::ITALIC)
    };

    let shown_songs_block = List::new(list_items)
        .block(Block::default().title("songs").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(style))
        .highlight_symbol(">>");
    f.render_widget(shown_songs_block, area);
}

/// RENDERS THE CURRENTLY LISTENING TO BIT
fn playbar<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let playlist_items: Vec<ListItem> = app.player.current_songs.iter().enumerate().map(|(i, s)| {
        let symbol = if i == app.player.hovered_index {
            ">"
        } else {
            " "
        };

        let style = if i == app.player.playing_index {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let content = vec![
            Span::styled(symbol, Style::default().fg(Color::Red)),
            Span::raw("|"),
            Span::styled(&s.song_name, style),
        ];
        ListItem::new(Text::from(Spans::from(content)))
    }).collect();

    let style = match app.current_ui {
        UIMode::PlayBar => Style::default().fg(Color::Yellow),
        _ => Style::default()
    };

    let playlist_block = List::new(playlist_items)
        .block(Block::default().border_style(style).title("currently playing").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(playlist_block, area);
}

/// RENDERS THE SONG INFORMATION
fn song_info<B: Backend>(app: &App, f: &mut Frame<B>, area: Rect) {
    let to_raw_listitem = |i| {ListItem::new(Text::from(Spans::from(Span::raw(i))))};
    
    let content = if app.queried_songs.len() == 0 {
        vec![to_raw_listitem(format!("No song selected"))]
    } else {

        let song = app.queried_songs[app.query_i].clone();
        let length_mult = match song.modifier { 
            Mod::NoMod => 1.0,
            _ => 1.5,
        };
        
        let modded_length = (song.length as f64 / length_mult) as usize;
        let formatted_length = format!("{}:{:02}", modded_length/60, modded_length%60);
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
    let style = match app.current_ui {
        UIMode::Playlist => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };

    let list = match app.player.current_playlist.is_none() {
        true => get_outer_playlist(app, style),
        false => get_inner_playlist(app, style)
    };
    f.render_widget(list, area)
}

/// RETURNS LIST OF SONGS INSIDE PLAYLIST
fn get_inner_playlist(app: &App, style: Style) -> List {
    let playlist = app.player.current_playlist.clone().unwrap();

    let title = format!("name: {} |shuffle: {} |repeat: {} |", playlist.name, playlist.shuffle_on, playlist.repeat_on);
    let list_items: Vec<ListItem> = playlist.songs.iter().map(|s| {
        ListItem::new(Text::from(Spans::from(Span::raw(s.song_name.clone()))))
    }).collect();

    let playlist_block = List::new(list_items)
        .block(Block::default()
        .style(style)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
    );
    playlist_block
}

/// RETURNS LIST OF PLAYLIST
fn get_outer_playlist(app: &App, style: Style) -> List {
    let title = "playlists";

    let list_items: Vec<ListItem> = app.playlists.iter().map(|p| {
        ListItem::new(Text::from(Spans::from(Span::raw(p.name.clone()))))
    }).collect();

    let playlist_block = List::new(list_items)
        .block(Block::default()
        .style(style)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
    );
    playlist_block
}