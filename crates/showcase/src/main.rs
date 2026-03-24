use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Widget};
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::spinner::{Spinner, SpinnerState, SpinnerType};

// -- Widget registry --
// Each widget in the showcase gets an entry here. As new widgets are added,
// just append to this list.

const WIDGETS: &[&str] = &["Spinner"];

// -- Spinner config --

struct SpinnerEntry {
    name: &'static str,
    state: SpinnerState,
    spinner: Spinner,
}

fn spinner_entries() -> Vec<SpinnerEntry> {
    let style = Style::default().fg(Color::Indexed(69));

    let presets: &[(SpinnerType, &str)] = &[
        (SpinnerType::Line, "Line"),
        (SpinnerType::Dot, "Dot"),
        (SpinnerType::MiniDot, "MiniDot"),
        (SpinnerType::Jump, "Jump"),
        (SpinnerType::Pulse, "Pulse"),
        (SpinnerType::Points, "Points"),
        (SpinnerType::Globe, "Globe"),
        (SpinnerType::Moon, "Moon"),
        (SpinnerType::Monkey, "Monkey"),
        (SpinnerType::Meter, "Meter"),
        (SpinnerType::Hamburger, "Hamburger"),
        (SpinnerType::Ellipsis, "Ellipsis"),
    ];

    let mut entries: Vec<SpinnerEntry> = presets
        .iter()
        .map(|(t, name)| SpinnerEntry {
            name,
            state: SpinnerState::new(*t),
            spinner: Spinner::default().style(style),
        })
        .collect();

    entries.push(SpinnerEntry {
        name: "Custom",
        state: SpinnerState::custom(vec!["⠇", "⠸"], Duration::from_millis(300)),
        spinner: Spinner::default().style(Style::default().fg(Color::Indexed(212))),
    });

    entries
}

// -- App state --

struct App {
    // Widget list selection
    selected_widget: usize,

    // Spinner state
    spinner_index: usize,
    entries: Vec<SpinnerEntry>,
    last_tick: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            selected_widget: 0,
            spinner_index: 0,
            entries: spinner_entries(),
            last_tick: Instant::now(),
        }
    }

    fn select_widget_next(&mut self) {
        self.selected_widget = (self.selected_widget + 1) % WIDGETS.len();
    }

    fn select_widget_prev(&mut self) {
        self.selected_widget = if self.selected_widget == 0 {
            WIDGETS.len() - 1
        } else {
            self.selected_widget - 1
        };
    }

    fn set_spinner(&mut self, index: usize) {
        self.spinner_index = index;
        self.last_tick = Instant::now();
    }

    fn next_spinner(&mut self) {
        let index = (self.spinner_index + 1) % self.entries.len();
        self.set_spinner(index);
    }

    fn prev_spinner(&mut self) {
        let index = if self.spinner_index == 0 {
            self.entries.len() - 1
        } else {
            self.spinner_index - 1
        };
        self.set_spinner(index);
    }

    fn current_entry(&self) -> &SpinnerEntry {
        &self.entries[self.spinner_index]
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_tick;
        self.last_tick = now;
        self.entries[self.spinner_index].state.tick(dt);
    }
}

// -- Main --

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        app.tick();

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                // Widget list navigation
                KeyCode::Char('j') | KeyCode::Down => app.select_widget_next(),
                KeyCode::Char('k') | KeyCode::Up => app.select_widget_prev(),
                // Widget-specific controls
                KeyCode::Char('l') | KeyCode::Right => {
                    if WIDGETS[app.selected_widget] == "Spinner" {
                        app.next_spinner();
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if WIDGETS[app.selected_widget] == "Spinner" {
                        app.prev_spinner();
                    }
                }
                _ => {}
            }
        }
    }
}

// -- Drawing --

fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Two-column layout: sidebar (fixed) | detail (fill)
    let [sidebar_area, detail_area] =
        Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(area);

    draw_sidebar(frame, app, sidebar_area);
    draw_detail(frame, app, detail_area);
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .title(" Widgets ")
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    for (i, name) in WIDGETS.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let style = if i == app.selected_widget {
            Style::default()
                .fg(Color::Indexed(69))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Indexed(252))
        };
        let prefix = if i == app.selected_widget { "▸ " } else { "  " };
        let line = Line::from(Span::styled(format!("{prefix}{name}"), style));
        let line_area = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        line.render(line_area, frame.buffer_mut());
    }
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    if WIDGETS[app.selected_widget] == "Spinner" {
        draw_spinner_detail(frame, app, area);
    }
}

fn draw_spinner_detail(frame: &mut Frame, app: &App, area: Rect) {
    let entry = app.current_entry();

    let block = Block::bordered()
        .title(format!(" Spinner: {} ", entry.name))
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.is_empty() {
        return;
    }

    // Spinner + "Spinning..." on the first line
    let gap = " ";

    let spinner_area = Rect::new(inner.x, inner.y, 10, 1);
    frame.render_stateful_widget(&entry.spinner, spinner_area, &mut entry.state.clone());

    let frames = entry.state.frames();
    let spinner_width = frames
        .iter()
        .map(|f| unicode_display_width(f))
        .max()
        .unwrap_or(0);
    let text_x = inner.x + spinner_width + gap.len() as u16;
    let text = Span::styled("Spinning...", Style::default().fg(Color::Indexed(252)));
    let text_area = Rect::new(
        text_x,
        inner.y,
        inner.width.saturating_sub(text_x - inner.x),
        1,
    );
    Widget::render(text, text_area, frame.buffer_mut());

    // Help text
    if inner.height > 2 {
        let help = Line::from(Span::styled(
            "h/l, ←/→: change spinner",
            Style::default().fg(Color::Indexed(241)),
        ));
        let help_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
        help.render(help_area, frame.buffer_mut());
    }
}

/// Returns the display width of a string in terminal cells.
fn unicode_display_width(s: &str) -> u16 {
    use ratatui::text::Line;
    Line::raw(s).width() as u16
}
