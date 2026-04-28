//! Palette example — shows all theme palettes as color swatches.
//!
//! A single-page reference displaying every palette's semantic color roles
//! side by side. Inspired by Charmbracelet's lipgloss color examples.
//!
//! Run with: cargo run --example palette

use std::io;

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::theme::Palette;

/// Returns (role_name, color) pairs for the given palette.
fn roles(p: &Palette) -> Vec<(&'static str, ratatui::style::Color)> {
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

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    terminal.draw(view)?;

    // Wait for any key to quit
    loop {
        if let crossterm::event::Event::Key(_) = crossterm::event::read()? {
            return Ok(());
        }
    }
}

fn view(frame: &mut Frame) {
    let area = frame.area();

    let presets = Palette::presets();

    // Layout: label column + one column per palette
    let label_width = 14u16;
    let swatch_width = 8u16;
    let mut constraints = vec![Constraint::Length(label_width)];
    for _ in &presets {
        constraints.push(Constraint::Length(swatch_width));
    }
    constraints.push(Constraint::Fill(1)); // absorb remaining space
    let columns = Layout::horizontal(constraints).split(area);

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
            .fg(ratatui::style::Color::Indexed(252))
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(header, label_rows[0]);

    for (i, name) in ROLE_NAMES.iter().enumerate() {
        let label = Line::from(Span::styled(
            format!(" {name}"),
            Style::default().fg(ratatui::style::Color::Indexed(245)),
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

/// Fills the entire area with a solid background color.
struct ColorBlock {
    color: ratatui::style::Color,
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
