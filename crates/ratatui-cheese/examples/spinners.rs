//! Spinners example — direct port of bubbletea/examples/spinners
//!
//! Cycles through spinner presets with h/l or arrow keys.
//!
//! Run with: cargo run --example spinners

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::spinner::{Spinner, SpinnerState, SpinnerType};

const SPINNERS: &[SpinnerType] = &[
    SpinnerType::Line,
    SpinnerType::Dot,
    SpinnerType::MiniDot,
    SpinnerType::Jump,
    SpinnerType::Pulse,
    SpinnerType::Points,
    SpinnerType::Globe,
    SpinnerType::Moon,
    SpinnerType::Monkey,
];

struct Model {
    index: usize,
    spinner: Spinner,
    state: SpinnerState,
    last_tick: Instant,
}

impl Model {
    fn new() -> Self {
        let mut m = Self {
            index: 0,
            spinner: Spinner::default(),
            state: SpinnerState::default(),
            last_tick: Instant::now(),
        };
        m.reset_spinner();
        m
    }

    fn reset_spinner(&mut self) {
        self.spinner =
            Spinner::new(SPINNERS[self.index]).style(Style::default().fg(Color::Indexed(69)));
        self.state = SpinnerState::default();
        self.last_tick = Instant::now();
    }

    fn tick(&mut self) {
        let interval = SPINNERS[self.index].interval();
        if self.last_tick.elapsed() >= interval {
            self.state.tick(self.spinner.frames().len());
            self.last_tick = Instant::now();
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

        m.tick();

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('h') | KeyCode::Left => {
                    m.index = if m.index == 0 { SPINNERS.len() - 1 } else { m.index - 1 };
                    m.reset_spinner();
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    m.index = (m.index + 1) % SPINNERS.len();
                    m.reset_spinner();
                }
                _ => {}
            }
        }
    }
}

/// Matches the Go View() output:
///
///  [spinner][gap]Spinning...
///
/// h/l, ←/→: change spinner • q: exit
fn view(frame: &mut Frame, m: &Model) {
    let area = frame.area();

    // Dot frames have a trailing space, so no extra gap
    let gap: u16 = if m.index == 1 { 0 } else { 1 };

    // Render spinner at row 1, col 1 (matching the leading \n and space in Go)
    let spinner_area = Rect::new(1, 1, 10, 1);
    frame.render_stateful_widget(&m.spinner, spinner_area, &mut m.state.clone());

    // "Spinning..." text right after the spinner
    let frames = m.spinner.frames();
    let frame_str = frames[m.state.frame() % frames.len()];
    let spinner_width = Line::raw(frame_str).width() as u16;
    let text_x = 1 + spinner_width + gap;
    let text = Span::styled("Spinning...", Style::default().fg(Color::Indexed(252)));
    let text_area = Rect::new(text_x, 1, area.width.saturating_sub(text_x), 1);
    Widget::render(text, text_area, frame.buffer_mut());

    // Help text at row 3 (matching the \n\n gap in Go)
    let help = Line::from(Span::styled(
        "h/l, ←/→: change spinner • q: exit",
        Style::default().fg(Color::Indexed(241)),
    ));
    let help_area = Rect::new(0, 3, area.width, 1);
    help.render(help_area, frame.buffer_mut());
}
