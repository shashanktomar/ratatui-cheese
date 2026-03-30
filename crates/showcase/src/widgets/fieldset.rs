use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui_cheese::fieldset::{Fieldset, FieldsetFill};
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorStyles};
use ratatui_cheese::theme::Palette;

use super::Component;

struct FillEntry {
    name: &'static str,
    fill: fn() -> FieldsetFill,
    alignment: Alignment,
}

const FILL_ENTRIES: &[FillEntry] = &[
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

pub struct FieldsetComponent {
    fill_index: usize,
    paginator_state: PaginatorState,
}

impl FieldsetComponent {
    pub fn new() -> Self {
        Self {
            fill_index: 0,
            paginator_state: PaginatorState::new(FILL_ENTRIES.len(), 1),
        }
    }

    fn sync_paginator(&mut self) {
        self.paginator_state = PaginatorState::new(FILL_ENTRIES.len(), 1);
        for _ in 0..self.fill_index {
            self.paginator_state.next_page();
        }
    }
}

impl Component for FieldsetComponent {
    fn name(&self) -> &str {
        "Fieldset"
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('l') | KeyCode::Right => {
                self.fill_index = (self.fill_index + 1) % FILL_ENTRIES.len();
                self.sync_paginator();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.fill_index =
                    if self.fill_index == 0 { FILL_ENTRIES.len() - 1 } else { self.fill_index - 1 };
                self.sync_paginator();
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, palette: &Palette, area: Rect) {
        let entry = &FILL_ENTRIES[self.fill_index];

        let block = Block::bordered()
            .title(format!(" Fieldset: {} ", entry.name))
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let [fieldset_area, _, paginator_area, _, help_area] = Layout::vertical([
            Constraint::Length(10),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(inner);

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

        let fieldset_inner = fieldset.inner(fieldset_area);
        Widget::render(&fieldset, fieldset_area, frame.buffer_mut());

        if fieldset_inner.height > 0 && fieldset_inner.width > 0 {
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
                let y = fieldset_inner.y + 1 + i as u16;
                if y < fieldset_inner.y + fieldset_inner.height {
                    let line_area = Rect::new(
                        fieldset_inner.x + 1,
                        y,
                        fieldset_inner.width.saturating_sub(2),
                        1,
                    );
                    line.clone().render(line_area, frame.buffer_mut());
                }
            }
        }

        // Paginator
        let paginator = Paginator::default().styles(PaginatorStyles::from_palette(palette));
        StatefulWidget::render(
            &paginator,
            paginator_area,
            frame.buffer_mut(),
            &mut self.paginator_state,
        );

        // Help hint
        let help = Line::from(Span::styled(
            "h/l, ←/→: fill style",
            Style::default().fg(palette.faint),
        ));
        help.render(help_area, frame.buffer_mut());
    }
}
