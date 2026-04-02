use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Widget};
use ratatui_cheese::spinner::{Spinner, SpinnerState, SpinnerType};
use ratatui_cheese::theme::Palette;

use super::Component;

struct Entry {
    name: &'static str,
    state: SpinnerState,
}

pub struct SpinnerComponent {
    entries: Vec<Entry>,
    index: usize,
    last_tick: Instant,
}

impl SpinnerComponent {
    pub fn new() -> Self {
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

        let mut entries: Vec<Entry> = presets
            .iter()
            .map(|(t, name)| Entry {
                name,
                state: SpinnerState::new(*t),
            })
            .collect();

        entries.push(Entry {
            name: "Custom",
            state: SpinnerState::custom(vec!["⠇".into(), "⠸".into()], Duration::from_millis(300)),
        });

        Self {
            entries,
            index: 0,
            last_tick: Instant::now(),
        }
    }
}

impl Component for SpinnerComponent {
    fn name(&self) -> &str {
        "Spinner"
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('l') | KeyCode::Right => {
                self.index = (self.index + 1) % self.entries.len();
                self.last_tick = Instant::now();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.index = if self.index == 0 { self.entries.len() - 1 } else { self.index - 1 };
                self.last_tick = Instant::now();
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, palette: &Palette, area: Rect, focused: bool) {
        let entry = &mut self.entries[self.index];

        let border_style = if focused {
            Style::default().fg(palette.foreground)
        } else {
            Style::default().fg(palette.faint)
        };
        let block = Block::bordered()
            .title(format!(" Spinner: {} ", entry.name))
            .border_style(border_style)
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let spinner = Spinner::default().style(Style::default().fg(palette.primary));
        let spinner_area = Rect::new(inner.x, inner.y, 10, 1);
        frame.render_stateful_widget(&spinner, spinner_area, &mut entry.state);

        let frames = entry.state.frames();
        let spinner_width = frames
            .iter()
            .map(|f| {
                use ratatui::text::Line;
                Line::raw(f).width() as u16
            })
            .max()
            .unwrap_or(0);
        let text_x = inner.x + spinner_width + 1;
        let text = Span::styled("Spinning...", Style::default().fg(palette.foreground));
        let text_area = Rect::new(
            text_x,
            inner.y,
            inner.width.saturating_sub(text_x - inner.x),
            1,
        );
        Widget::render(text, text_area, frame.buffer_mut());

        if inner.height > 2 {
            let help = Line::from(Span::styled(
                "h/l, ←/→: change spinner",
                Style::default().fg(palette.faint),
            ));
            let help_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
            help.render(help_area, frame.buffer_mut());
        }
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_tick;
        self.last_tick = now;
        self.entries[self.index].state.tick(dt);
    }
}
