mod app;
mod game;
mod ui;
mod words;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> io::Result<()> {
    let words = words::load_words();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(words);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('r') | KeyCode::Char('R') if app.is_done() => {
                            app.new_game();
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc if app.is_done() => {
                            app.quit();
                            break;
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            app.quit();
                            break;
                        }
                        KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                            app.type_char(c);
                        }
                        KeyCode::Enter => {
                            app.submit();
                        }
                        KeyCode::Backspace => {
                            app.backspace();
                        }
                        _ => {}
                    }
                }
            }
        }

        app.tick();
    }

    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
