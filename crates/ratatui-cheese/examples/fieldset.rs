//! Fieldset example — cycles through fill styles with h/l or arrow keys.
//!
//! Run with: cargo run --example fieldset

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::fieldset::{Fieldset, FieldsetFill};
use ratatui_cheese::help::{Binding, Help};
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorStyles};
use ratatui_cheese::theme::Palette;

struct FillEntry {
    name: &'static str,
    fill: fn() -> FieldsetFill,
    alignment: Alignment,
}

const FILLS: &[FillEntry] = &[
    FillEntry {
        name: "Slash",
        fill: || FieldsetFill::Slash,
        alignment: Alignment::Left,
    },
    FillEntry {
        name: "Dash",
        fill: || FieldsetFill::Dash,
        alignment: Alignment::Center,
    },
    FillEntry {
        name: "Dot",
        fill: || FieldsetFill::Dot,
        alignment: Alignment::Right,
    },
    FillEntry {
        name: "Double",
        fill: || FieldsetFill::Double,
        alignment: Alignment::Left,
    },
    FillEntry {
        name: "Thick",
        fill: || FieldsetFill::Thick,
        alignment: Alignment::Center,
    },
    FillEntry {
        name: "Star",
        fill: || FieldsetFill::Star,
        alignment: Alignment::Right,
    },
    FillEntry {
        name: "Custom",
        fill: || FieldsetFill::Custom("/\\".to_string()),
        alignment: Alignment::Center,
    },
];

struct Model {
    fill_index: usize,
    paginator_state: PaginatorState,
}

impl Model {
    fn new() -> Self {
        Self {
            fill_index: 0,
            paginator_state: PaginatorState::new(FILLS.len(), 1),
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
        terminal.draw(|frame| view(frame, &mut m))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('h') | KeyCode::Left => {
                    m.fill_index =
                        if m.fill_index == 0 { FILLS.len() - 1 } else { m.fill_index - 1 };
                    m.paginator_state = PaginatorState::new(FILLS.len(), 1);
                    for _ in 0..m.fill_index {
                        m.paginator_state.next_page();
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    m.fill_index = (m.fill_index + 1) % FILLS.len();
                    m.paginator_state = PaginatorState::new(FILLS.len(), 1);
                    for _ in 0..m.fill_index {
                        m.paginator_state.next_page();
                    }
                }
                _ => {}
            }
        }
    }
}

fn view(frame: &mut Frame, m: &mut Model) {
    let palette = Palette::charm();
    let area = frame.area();

    let content_area = Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );

    let entry = &FILLS[m.fill_index];

    let help = Help::default().bindings(vec![
        Binding::new("h/l", "fill style"),
        Binding::new("q", "quit"),
    ]);
    let help_height = help.required_height();

    let [fieldset_area, _, paginator_area, _, help_area] = Layout::vertical([
        Constraint::Length(10),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(help_height),
    ])
    .areas(content_area);

    let fieldset_area = Rect::new(
        fieldset_area.x,
        fieldset_area.y,
        fieldset_area.width / 2,
        fieldset_area.height,
    );

    let primary_style = Style::default().fg(palette.primary);
    let fieldset = Fieldset::new()
        .title(entry.name)
        .top_alignment(entry.alignment)
        .fill((entry.fill)())
        .title_style(primary_style)
        .rule_style(primary_style);

    let inner = fieldset.inner(fieldset_area);
    Widget::render(&fieldset, fieldset_area, frame.buffer_mut());

    if inner.height > 0 && inner.width > 0 {
        let lines = [
            Line::from(Span::styled(
                "Content inside the fieldset.",
                Style::default().fg(palette.foreground),
            )),
            Line::from(Span::styled(
                format!("Fill: {}", entry.name),
                Style::default().fg(palette.muted),
            )),
        ];
        for (i, line) in lines.iter().enumerate() {
            let y = inner.y + 1 + i as u16;
            if y < inner.y + inner.height {
                let line_area = Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1);
                line.render(line_area, frame.buffer_mut());
            }
        }
    }

    // Paginator
    let paginator = Paginator::default().styles(PaginatorStyles::from_palette(&palette));
    StatefulWidget::render(
        &paginator,
        paginator_area,
        frame.buffer_mut(),
        &mut m.paginator_state,
    );

    Widget::render(&help, help_area, frame.buffer_mut());
}
