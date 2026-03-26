mod app;
mod widgets;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = app::App::new();

    loop {
        terminal.draw(|frame| app.draw(frame))?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && app.handle_key(key.code)
        {
            break;
        }
    }

    ratatui::restore();
    Ok(())
}
