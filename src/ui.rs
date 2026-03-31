use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Screen};
use crate::game::LetterState;

const TILE_SIZE: u16 = 5;
const TILE_GAP: u16 = 1;
const SHAKE_INTERVAL_MS: u128 = 50;
const SHAKE_DURATION_MS: u128 = 300;

fn get_letter_color(state: LetterState) -> Color {
    match state {
        LetterState::Correct => Color::Green,
        LetterState::Present => Color::Yellow,
        LetterState::Absent => Color::DarkGray,
        LetterState::Unknown => Color::Reset,
    }
}

fn get_letter_style(state: LetterState) -> Style {
    match state {
        LetterState::Correct | LetterState::Present => Style::default()
            .fg(Color::Black)
            .bg(get_letter_color(state))
            .add_modifier(Modifier::BOLD),
        LetterState::Absent => Style::default()
            .fg(Color::White)
            .bg(get_letter_color(state)),
        LetterState::Unknown => Style::default().fg(Color::DarkGray),
    }
}

fn get_border_color(state: LetterState, active: bool) -> Color {
    if active {
        return Color::White;
    }
    match state {
        LetterState::Correct => Color::Green,
        LetterState::Present => Color::Yellow,
        LetterState::Absent => Color::DarkGray,
        LetterState::Unknown => Color::DarkGray,
    }
}

fn render_tile(
    f: &mut Frame,
    area: Rect,
    letter: char,
    state: LetterState,
    active: bool,
    show_cursor: bool,
) {
    let border_color = get_border_color(state, active);
    let letter_style = get_letter_style(state);

    let tile = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    f.render_widget(tile, area);

    if letter != ' ' {
        let text = Span::raw(letter.to_ascii_uppercase().to_string());
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(letter_style);
        f.render_widget(paragraph, area);
    } else if active && show_cursor {
        let cursor = Span::raw("█".to_string());
        let paragraph = Paragraph::new(cursor).alignment(Alignment::Center).style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
        f.render_widget(paragraph, area);
    }
}

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(14),
            Constraint::Length(2),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new("W O R D L E")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title, chunks[0]);

    let is_game_over = matches!(app.screen, Screen::Won | Screen::Lost);

    if !is_game_over {
        let grid_area = chunks[1];
        let grid_width = 5 * TILE_SIZE + 4 * TILE_GAP;
        let grid_height = 6 * TILE_SIZE + 5;
        let grid_rect = Rect::new(
            grid_area.x + (grid_area.width.saturating_sub(grid_width)) / 2,
            grid_area.y + (grid_area.height.saturating_sub(grid_height)) / 2,
            grid_width,
            grid_height.min(grid_area.height),
        );

        let current_row = app.game.guesses.len();

        let row_constraints: Vec<Constraint> =
            (0..6).map(|_| Constraint::Length(TILE_SIZE)).collect();

        for (row_idx, row_rect) in Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(grid_rect)
            .iter()
            .enumerate()
        {
            let col_constraints: Vec<Constraint> =
                (0..5).map(|_| Constraint::Length(TILE_SIZE)).collect();
            let col_rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .spacing(TILE_GAP)
                .split(*row_rect);

            for (col_idx, tile_rect) in col_rects.iter().enumerate() {
                let letter = if let Some(guess) = app.game.guesses.get(row_idx) {
                    guess.letters[col_idx]
                } else if row_idx == current_row && col_idx < app.game.current.len() {
                    app.game.current[col_idx]
                } else if row_idx == current_row {
                    ' '
                } else {
                    ' '
                };

                let state = if let Some(guess) = app.game.guesses.get(row_idx) {
                    guess.states[col_idx]
                } else {
                    LetterState::Unknown
                };

                let active = row_idx == current_row;
                let show_cursor = app.cursor_blink;

                let shake_offset = if let Some(shake_time) = app.shake {
                    let elapsed = std::time::Instant::now()
                        .duration_since(shake_time)
                        .as_millis();
                    if elapsed < SHAKE_DURATION_MS {
                        ((elapsed / SHAKE_INTERVAL_MS) % 3) as i32 - 1
                    } else {
                        0
                    }
                } else {
                    0
                };

                let shifted_rect = Rect::new(
                    (tile_rect.x as i32 + shake_offset).max(0) as u16,
                    tile_rect.y,
                    tile_rect.width,
                    tile_rect.height,
                );

                render_tile(f, shifted_rect, letter, state, active, show_cursor);
            }
        }

        render_keyboard(f, chunks[2], app);

        // Ctrl+Q hint
        let quit_hint = Paragraph::new("Ctrl+Q to quit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(quit_hint, chunks[3]);
    }

    if let Some(msg) = &app.msg {
        let msg_widget = Paragraph::new(msg.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        f.render_widget(msg_widget, chunks[4]);
    }

    match app.screen {
        Screen::Won => render_overlay(
            f,
            area,
            "YOU WON!",
            &app.game.get_target_word(),
            app.game.guesses.len(),
        ),
        Screen::Lost => render_overlay(f, area, "GAME OVER", &app.game.get_target_word(), 0),
        _ => {}
    }
}

fn render_keyboard(f: &mut Frame, area: Rect, app: &App) {
    let rows = vec!["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];

    // Keyboard is 30 wide (10 keys * 3), align with grid which is 29 wide
    // Center point is the same, so calculate from same reference
    let keyboard_width = 10 * 3; // 30

    // Use same centering math as grid
    let keyboard_area = Rect::new(
        area.x + (area.width.saturating_sub(keyboard_width)) / 2,
        area.y,
        keyboard_width,
        3 * 4,
    );

    for (row_idx, row) in rows.iter().enumerate() {
        let y = keyboard_area.y + row_idx as u16 * 4;
        let x_offset = if row_idx == 2 { 3 } else { 0 };

        for (col_idx, c) in row.chars().enumerate() {
            let x = keyboard_area.x + x_offset as u16 + col_idx as u16 * 3;
            let state = app
                .game
                .keyboard
                .get(&c)
                .copied()
                .unwrap_or(LetterState::Unknown);
            let color = get_letter_color(state);

            let key_rect = Rect::new(x, y, 3, 3);
            let key = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .style(
                    Style::default()
                        .bg(if state != LetterState::Unknown {
                            color
                        } else {
                            Color::Reset
                        })
                        .fg(Color::White),
                );

            f.render_widget(key, key_rect);

            let letter = Span::raw(c.to_string()).style(Style::default().fg(
                if state == LetterState::Present {
                    Color::Black
                } else {
                    Color::White
                },
            ));
            let p = Paragraph::new(letter).alignment(Alignment::Center);
            f.render_widget(p, key_rect);
        }
    }
}

fn render_overlay(f: &mut Frame, area: Rect, title: &str, word: &str, guesses: usize) {
    let popup_width = 30;
    let popup_height = 10;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let rect = Rect::new(x, y, popup_width, popup_height);

    let bg = Block::default()
        .style(Style::default().bg(Color::DarkGray))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    f.render_widget(bg, rect);

    let title_text = if guesses == 1 {
        "GENIUS!"
    } else if guesses == 2 {
        "MAGNIFICENT!"
    } else if guesses == 3 {
        "IMPRESSIVE!"
    } else if guesses == 4 {
        "SPLENDID!"
    } else if guesses == 5 {
        "GREAT!"
    } else if guesses == 6 {
        "PHEW!"
    } else {
        title
    };

    let title_widget = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(
        title_widget,
        Rect::new(rect.x + 1, rect.y + 1, popup_width - 2, 1),
    );

    let word_widget = Paragraph::new(word.to_uppercase())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(
        word_widget,
        Rect::new(rect.x + 1, rect.y + 3, popup_width - 2, 1),
    );

    let hint = Paragraph::new("[R] new game  [Q] quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
    f.render_widget(hint, Rect::new(rect.x + 1, rect.y + 6, popup_width - 2, 1));
}
