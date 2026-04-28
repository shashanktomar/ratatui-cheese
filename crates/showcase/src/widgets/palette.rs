use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Widget};
use ratatui_cheese::theme::Palette;

use super::Component;

fn roles(p: &Palette) -> Vec<(&'static str, Color)> {
    vec![
        ("foreground", p.foreground),
        ("muted", p.muted),
        ("faint", p.faint),
        ("surface", p.surface),
        ("border", p.border),
        ("highlight", p.highlight),
        ("on_highlight", p.on_highlight),
        ("primary", p.primary),
        ("secondary", p.secondary),
    ]
}

const ROLE_NAMES: &[&str] = &[
    "foreground",
    "muted",
    "faint",
    "surface",
    "border",
    "highlight",
    "on_highlight",
    "primary",
    "secondary",
];

pub struct PaletteComponent;

impl PaletteComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for PaletteComponent {
    fn name(&self) -> &str {
        "Palette"
    }

    fn handle_key(&mut self, _key: KeyCode) {}

    fn draw(&mut self, frame: &mut Frame, app_palette: &Palette, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(app_palette.foreground)
        } else {
            Style::default().fg(app_palette.faint)
        };
        let block = Block::bordered()
            .title(" Palette ")
            .border_style(border_style)
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let presets = Palette::presets();

        // Layout: label column + one column per palette
        let label_width = 14u16;
        let swatch_width = 8u16;
        let mut col_constraints = vec![Constraint::Length(label_width)];
        for _ in &presets {
            col_constraints.push(Constraint::Length(swatch_width));
        }
        col_constraints.push(Constraint::Fill(1));
        let columns = Layout::horizontal(col_constraints).split(inner);

        // Row layout: header + one row per role
        let mut row_constraints = vec![Constraint::Length(1)]; // header
        for _ in ROLE_NAMES {
            row_constraints.push(Constraint::Length(1));
        }
        row_constraints.push(Constraint::Fill(1)); // spacer

        // Render label column
        let label_rows = Layout::vertical(&row_constraints).split(columns[0]);
        let header = Line::from(Span::styled(
            " Role",
            Style::default()
                .fg(Color::Indexed(252))
                .add_modifier(Modifier::BOLD),
        ));
        frame.render_widget(header, label_rows[0]);

        for (i, name) in ROLE_NAMES.iter().enumerate() {
            let label = Line::from(Span::styled(
                format!(" {name}"),
                Style::default().fg(Color::Indexed(245)),
            ));
            frame.render_widget(label, label_rows[i + 1]);
        }

        // Render each palette column
        for (col_idx, (name, palette)) in presets.iter().enumerate() {
            let col_area = columns[col_idx + 1];
            let rows = Layout::vertical(&row_constraints).split(col_area);

            let header = Line::from(Span::styled(
                format!(" {name}"),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ));
            frame.render_widget(header, rows[0]);

            for (i, (_role, color)) in roles(palette).iter().enumerate() {
                let swatch = ColorBlock { color: *color };
                frame.render_widget(swatch, rows[i + 1]);
            }
        }
    }
}

struct ColorBlock {
    color: Color,
}

impl Widget for ColorBlock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().bg(self.color);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_style(style);
            }
        }
    }
}
