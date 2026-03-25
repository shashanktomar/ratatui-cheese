//! Help example — direct port of bubbletea/examples/help
//!
//! Demonstrates the help widget with short/full toggle.
//!
//! Run with: cargo run --example help

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::help::{Binding, Help, KeyMap};

struct Keys;

impl KeyMap for Keys {
    fn short_help(&self) -> Vec<Binding> {
        vec![Binding::new("?", "toggle help"), Binding::new("q", "quit")]
    }

    fn full_help(&self) -> Vec<Vec<Binding>> {
        vec![
            vec![
                Binding::new("↑/k", "move up"),
                Binding::new("↓/j", "move down"),
                Binding::new("←/h", "move left"),
                Binding::new("→/l", "move right"),
            ],
            vec![Binding::new("?", "toggle help"), Binding::new("q", "quit")],
        ]
    }
}

struct Model {
    last_key: String,
    show_all: bool,
}

impl Model {
    fn new() -> Self {
        Self {
            last_key: String::new(),
            show_all: false,
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut m = Model::new();

    loop {
        terminal.draw(|frame| view(frame, &m))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('?') => m.show_all = !m.show_all,
                KeyCode::Up | KeyCode::Char('k') => m.last_key = "↑".into(),
                KeyCode::Down | KeyCode::Char('j') => m.last_key = "↓".into(),
                KeyCode::Left | KeyCode::Char('h') => m.last_key = "←".into(),
                KeyCode::Right | KeyCode::Char('l') => m.last_key = "→".into(),
                _ => {}
            }
        }
    }
}

fn view(frame: &mut Frame, m: &Model) {
    let input_style = Style::default().fg(Color::Rgb(0xFF, 0x75, 0xB7));

    // Status line
    let status = if m.last_key.is_empty() {
        Span::raw("Waiting for input...")
    } else {
        Span::styled(format!("You chose: {}", m.last_key), input_style)
    };
    let status_area = Rect::new(1, 1, 40, 1);
    Widget::render(status, status_area, frame.buffer_mut());

    // Help view below status
    let help = Help::new(&Keys).show_all(m.show_all);
    let help_height = if m.show_all { 4 } else { 1 };
    let help_area = Rect::new(0, 3, 40, help_height);
    Widget::render(&help, help_area, frame.buffer_mut());
}
