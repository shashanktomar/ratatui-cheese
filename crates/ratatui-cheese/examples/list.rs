use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer};
use ratatui_cheese::help::{Binding, Help};
use ratatui_cheese::list::{List, ListItem, ListItemContext, ListState};
use ratatui_cheese::theme::Palette;

// ---------------------------------------------------------------------------
// Item types
// ---------------------------------------------------------------------------

struct TwoLineItem {
    title: String,
    description: String,
}

impl TwoLineItem {
    fn new(title: &str, desc: &str) -> Self {
        Self {
            title: title.to_string(),
            description: desc.to_string(),
        }
    }
}

impl ListItem for TwoLineItem {
    fn height(&self) -> u16 {
        2
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let p = &ctx.palette;
        let (title_style, desc_style) = if ctx.selected {
            (
                Style::default().fg(p.primary),
                Style::default().fg(p.muted),
            )
        } else {
            (
                Style::default().fg(p.foreground),
                Style::default().fg(p.faint),
            )
        };

        buf.set_string(area.x, area.y, &self.title, title_style);
        if area.height > 1 {
            buf.set_string(area.x, area.y + 1, &self.description, desc_style);
        }
    }
}

struct DetailItem {
    title: String,
    description: String,
    category: String,
    year: String,
    origin: String,
}

impl DetailItem {
    fn new(title: &str, desc: &str, category: &str, year: &str, origin: &str) -> Self {
        Self {
            title: title.to_string(),
            description: desc.to_string(),
            category: category.to_string(),
            year: year.to_string(),
            origin: origin.to_string(),
        }
    }
}

impl ListItem for DetailItem {
    fn height(&self) -> u16 {
        2
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let p = &ctx.palette;
        let title_fg = if ctx.selected { p.primary } else { p.foreground };

        // Line 1: title · description
        buf.set_string(area.x, area.y, &self.title, Style::default().fg(title_fg));
        let sep_x = area.x + self.title.len() as u16;
        buf.set_string(sep_x, area.y, " · ", Style::default().fg(p.faint));
        buf.set_string(sep_x + 3, area.y, &self.description, Style::default().fg(p.muted));

        // Line 2: category  ·  year  ·  origin
        if area.height > 1 {
            let meta = format!("{}  ·  {}  ·  {}", self.category, self.year, self.origin);
            buf.set_string(area.x, area.y + 1, &meta, Style::default().fg(p.faint));
        }
    }
}

struct SeparatedDetailItem {
    title: String,
    description: String,
    category: String,
    designation: String,
    distance: String,
}

impl SeparatedDetailItem {
    fn new(title: &str, desc: &str, category: &str, designation: &str, distance: &str) -> Self {
        Self {
            title: title.to_string(),
            description: desc.to_string(),
            category: category.to_string(),
            designation: designation.to_string(),
            distance: distance.to_string(),
        }
    }
}

impl ListItem for SeparatedDetailItem {
    fn height(&self) -> u16 {
        2
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let p = &ctx.palette;
        let (title_fg, desc_fg, meta_fg) = if ctx.selected {
            (p.primary, p.muted, p.muted)
        } else {
            (p.foreground, p.faint, p.faint)
        };

        // Left side: title · description
        buf.set_string(area.x, area.y, &self.title, Style::default().fg(title_fg));
        let sep_x = area.x + self.title.len() as u16;
        buf.set_string(sep_x, area.y, " · ", Style::default().fg(p.faint));
        buf.set_string(sep_x + 3, area.y, &self.description, Style::default().fg(desc_fg));

        // Right side: category · designation · distance
        let right = format!("{} · {} · {}", self.category, self.designation, self.distance);
        let right_x = (area.x + area.width).saturating_sub(right.len() as u16);
        buf.set_string(right_x, area.y, &right, Style::default().fg(meta_fg));

        // Line 2: separator
        if area.height > 1 {
            let line = "─".repeat(area.width as usize);
            buf.set_string(area.x, area.y + 1, &line, Style::default().fg(p.faint));
        }
    }
}

struct DenseItem {
    title: String,
    description: String,
    category: String,
    designation: String,
    distance: String,
}

impl DenseItem {
    fn new(title: &str, desc: &str, category: &str, designation: &str, distance: &str) -> Self {
        Self {
            title: title.to_string(),
            description: desc.to_string(),
            category: category.to_string(),
            designation: designation.to_string(),
            distance: distance.to_string(),
        }
    }
}

impl ListItem for DenseItem {
    fn height(&self) -> u16 {
        1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let p = &ctx.palette;
        let (title_fg, desc_fg, meta_fg) = if ctx.selected {
            (p.primary, p.muted, p.muted)
        } else {
            (p.foreground, p.faint, p.faint)
        };

        // Left: title · description
        buf.set_string(area.x, area.y, &self.title, Style::default().fg(title_fg));
        let sep_x = area.x + self.title.len() as u16;
        buf.set_string(sep_x, area.y, " · ", Style::default().fg(p.faint));
        buf.set_string(sep_x + 3, area.y, &self.description, Style::default().fg(desc_fg));

        // Right: category · designation · distance
        let right = format!("{} · {} · {}", self.category, self.designation, self.distance);
        let right_x = (area.x + area.width).saturating_sub(right.len() as u16);
        buf.set_string(right_x, area.y, &right, Style::default().fg(meta_fg));
    }
}

struct SimpleItem(String);

impl SimpleItem {
    fn new(text: &str) -> Self {
        Self(text.to_string())
    }
}

impl ListItem for SimpleItem {
    fn height(&self) -> u16 {
        1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let p = &ctx.palette;
        let style = if ctx.selected {
            Style::default().fg(p.primary)
        } else {
            Style::default().fg(p.foreground)
        };
        buf.set_string(area.x, area.y, &self.0, style);
    }
}

struct NumberedItem(String);

impl NumberedItem {
    fn new(text: &str) -> Self {
        Self(text.to_string())
    }
}

impl ListItem for NumberedItem {
    fn height(&self) -> u16 {
        1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let p = &ctx.palette;
        let style = if ctx.selected {
            Style::default().fg(p.primary)
        } else {
            Style::default().fg(p.foreground)
        };
        let text = format!("{}. {}", ctx.index + 1, self.0);
        buf.set_string(area.x, area.y, &text, style);
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

const EXAMPLE_NAMES: &[&str] = &[
    "Two-Line Items",
    "Detail Cards",
    "With Line Separator",
    "Dense",
    "Simple Items",
    "Numbered",
];

enum DemoKind {
    TwoLine {
        items: Vec<TwoLineItem>,
        state: ListState,
    },
    Detail {
        items: Vec<DetailItem>,
        state: ListState,
    },
    Separated {
        items: Vec<SeparatedDetailItem>,
        state: ListState,
    },
    Dense {
        items: Vec<DenseItem>,
        state: ListState,
    },
    Simple {
        items: Vec<SimpleItem>,
        state: ListState,
    },
    Compact {
        items: Vec<NumberedItem>,
        state: ListState,
    },
}

struct App {
    demos: Vec<DemoKind>,
    current: usize,
}

impl App {
    fn new() -> Self {
        let two_line = vec![
            TwoLineItem::new("Raspberry Pi", "A small, affordable computer"),
            TwoLineItem::new("Arduino", "Open-source electronics platform"),
            TwoLineItem::new("ESP32", "Low-cost Wi-Fi microcontroller"),
            TwoLineItem::new("BeagleBone", "Open-source hardware computer"),
            TwoLineItem::new("Teensy", "USB-based microcontroller board"),
            TwoLineItem::new("Adafruit Feather", "Lightweight development boards"),
            TwoLineItem::new("STM32", "ARM Cortex-M microcontroller"),
            TwoLineItem::new("BBC micro:bit", "Pocket-sized computer for education"),
            TwoLineItem::new("Particle Photon", "Wi-Fi connected microcontroller"),
            TwoLineItem::new("RISC-V Board", "Open instruction set architecture"),
            TwoLineItem::new("NVIDIA Jetson", "AI computing on a small board"),
            TwoLineItem::new("Pine64", "Open-source ARM single board computer"),
        ];
        let detail = vec![
            DetailItem::new("Andromeda", "Nearest spiral galaxy, on collision course with the Milky Way", "Galaxy", "M31", "2.537 Mly"),
            DetailItem::new("Crab Nebula", "Supernova remnant with a rapidly spinning pulsar at its core", "Nebula", "M1", "6,500 ly"),
            DetailItem::new("Pillars of Creation", "Towering columns of gas and dust in the Eagle Nebula", "Nebula", "M16", "7,000 ly"),
            DetailItem::new("Sagittarius A*", "Supermassive black hole at the center of our galaxy", "Black Hole", "Sgr A*", "26,000 ly"),
            DetailItem::new("Europa", "Icy moon of Jupiter with a subsurface ocean twice Earth's volume", "Moon", "Jupiter II", "628.3 Mkm"),
            DetailItem::new("Olympus Mons", "Largest volcano in the solar system, 2.5× Everest's height", "Volcano", "Mars", "225 Mkm"),
            DetailItem::new("Titan", "Saturn's largest moon with dense atmosphere and methane lakes", "Moon", "Saturn VI", "1.2 Bkm"),
            DetailItem::new("Whirlpool Galaxy", "Grand-design spiral interacting with a smaller companion", "Galaxy", "M51", "23 Mly"),
            DetailItem::new("Io", "Most volcanically active body in the solar system", "Moon", "Jupiter I", "628.3 Mkm"),
            DetailItem::new("Horsehead Nebula", "Dark nebula silhouetted against glowing hydrogen gas", "Nebula", "B33", "1,375 ly"),
            DetailItem::new("Enceladus", "Tiny moon venting water ice geysers from its south pole", "Moon", "Saturn II", "1.27 Bkm"),
            DetailItem::new("Sombrero Galaxy", "Bright nucleus with an unusually large central bulge and dust lane", "Galaxy", "M104", "31.1 Mly"),
        ];
        let separated: Vec<SeparatedDetailItem> = detail
            .iter()
            .map(|d| {
                SeparatedDetailItem::new(
                    &d.title,
                    &d.description,
                    &d.category,
                    &d.year,
                    &d.origin,
                )
            })
            .collect();
        let dense: Vec<DenseItem> = detail
            .iter()
            .map(|d| DenseItem::new(&d.title, &d.description, &d.category, &d.year, &d.origin))
            .collect();
        let simple: Vec<SimpleItem> = [
            "Rust", "Go", "Python", "TypeScript", "Zig", "Elixir", "Haskell", "OCaml", "Swift",
            "Kotlin", "Ruby", "C", "Lua", "Julia",
        ]
        .iter()
        .map(|s| SimpleItem::new(s))
        .collect();
        let compact: Vec<NumberedItem> = simple.iter().map(|s| NumberedItem::new(&s.0)).collect();

        let demos = vec![
            DemoKind::TwoLine {
                state: ListState::new(two_line.len()),
                items: two_line,
            },
            DemoKind::Detail {
                state: ListState::new(detail.len()),
                items: detail,
            },
            DemoKind::Separated {
                state: ListState::new(separated.len()),
                items: separated,
            },
            DemoKind::Dense {
                state: ListState::new(dense.len()),
                items: dense,
            },
            DemoKind::Simple {
                state: ListState::new(simple.len()),
                items: simple,
            },
            DemoKind::Compact {
                state: ListState::new(compact.len()),
                items: compact,
            },
        ];

        Self { demos, current: 0 }
    }

    fn next_demo(&mut self) {
        self.current = (self.current + 1) % self.demos.len();
    }

    fn prev_demo(&mut self) {
        self.current = (self.current + self.demos.len() - 1) % self.demos.len();
    }

    fn item_count(&self) -> usize {
        match &self.demos[self.current] {
            DemoKind::TwoLine { items, .. } => items.len(),
            DemoKind::Detail { items, .. } => items.len(),
            DemoKind::Separated { items, .. } => items.len(),
            DemoKind::Dense { items, .. } => items.len(),
            DemoKind::Simple { items, .. } => items.len(),
            DemoKind::Compact { items, .. } => items.len(),
        }
    }

    fn state_mut(&mut self) -> &mut ListState {
        match &mut self.demos[self.current] {
            DemoKind::TwoLine { state, .. }
            | DemoKind::Detail { state, .. }
            | DemoKind::Separated { state, .. }
            | DemoKind::Dense { state, .. }
            | DemoKind::Simple { state, .. }
            | DemoKind::Compact { state, .. } => state,
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| draw(frame, &mut app))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('n') => app.next_demo(),
                KeyCode::Char('p') => app.prev_demo(),
                KeyCode::Char('j') | KeyCode::Down => {
                    let count = app.item_count();
                    app.state_mut().select_next(count, false);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let count = app.item_count();
                    app.state_mut().select_prev(count, false);
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    let count = app.item_count();
                    app.state_mut().next_page(count);
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    let count = app.item_count();
                    app.state_mut().prev_page(count);
                }
                KeyCode::Char('g') => {
                    app.state_mut().select_first_on_page();
                }
                KeyCode::Char('G') => {
                    let count = app.item_count();
                    app.state_mut().select_last_on_page(count);
                }
                _ => {}
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let palette = Palette::charm();
    let area = frame.area();

    // Indent the content area
    let content_area = Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );

    let help = Help::default().bindings(vec![
        Binding::new("j/k", "select"),
        Binding::new("g/G", "top/bottom"),
        Binding::new("h/l", "page"),
        Binding::new("n/p", "next/prev example"),
        Binding::new("q", "quit"),
    ]);
    let help_height = help.required_height();

    let example_name = EXAMPLE_NAMES[app.current];

    // Cap the list to roughly half the terminal so pagination is always visible
    let max_list_height = content_area.height / 2;

    let [title_area, _, list_area, _, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Max(max_list_height),
        Constraint::Min(1),
        Constraint::Length(help_height),
    ])
    .areas(content_area);

    // Title
    frame.buffer_mut().set_string(
        title_area.x,
        title_area.y,
        format!("List Example: {example_name}"),
        Style::default().fg(palette.foreground).bold(),
    );

    // Render the current demo
    match &mut app.demos[app.current] {
        DemoKind::TwoLine { items, state } => {
            let list = List::new(items.as_slice())
                .title("Development Boards")
                .show_count(true)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Detail { items, state } => {
            let list = List::new(items.as_slice())
                .title("Celestial Objects")
                .show_count(true)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Separated { items, state } => {
            let list = List::new(items.as_slice())
                .title("Celestial Objects")
                .show_count(true)
                .item_spacing(0)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Dense { items, state } => {
            let list = List::new(items.as_slice())
                .title("Celestial Objects")
                .show_count(true)
                .item_spacing(0)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Simple { items, state } => {
            let list = List::new(items.as_slice())
                .title("Programming Languages")
                .show_count(true)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Compact { items, state } => {
            let list = List::new(items.as_slice())
                .title("Programming Languages")
                .show_count(true)
                .item_spacing(0)
                .selection_indicator(">")
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
    }

    // Help
    Widget::render(&help, help_area, frame.buffer_mut());
}
