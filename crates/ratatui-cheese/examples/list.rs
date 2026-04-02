use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer};
use ratatui_cheese::help::{Binding, Help};
use ratatui_cheese::list::{
    DefaultHeader, List, ListHeader, ListHeaderContext, ListItem, ListItemContext, ListState,
};
use ratatui_cheese::theme::Palette;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn truncate_desc(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        return text.to_string();
    }
    if max_width <= 1 {
        return "…".to_string();
    }
    let mut result: String = text.chars().take(max_width - 1).collect();
    result.push('…');
    result
}

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
            (Style::default().fg(p.primary), Style::default().fg(p.muted))
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
    favorited: bool,
}

impl DetailItem {
    fn new(title: &str, desc: &str, category: &str, year: &str, origin: &str) -> Self {
        Self {
            title: title.to_string(),
            description: desc.to_string(),
            category: category.to_string(),
            year: year.to_string(),
            origin: origin.to_string(),
            favorited: false,
        }
    }

    fn toggle_favorite(&mut self) {
        self.favorited = !self.favorited;
    }
}

impl ListItem for DetailItem {
    fn height(&self) -> u16 {
        2
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let p = &ctx.palette;
        let title_fg = if ctx.selected { p.primary } else { p.foreground };

        // Line 1: title · description  ★
        buf.set_string(area.x, area.y, &self.title, Style::default().fg(title_fg));
        let sep_x = area.x + self.title.len() as u16;
        buf.set_string(sep_x, area.y, " · ", Style::default().fg(p.faint));
        let desc_x = sep_x + 3;
        buf.set_string(
            desc_x,
            area.y,
            &self.description,
            Style::default().fg(p.muted),
        );
        if self.favorited {
            let star_x = (area.x + area.width).saturating_sub(1);
            buf.set_string(star_x, area.y, "★", Style::default().fg(p.primary));
        }

        // Line 2: category · year · origin  (+ hint when selected)
        if area.height > 1 {
            let meta = format!("{}  ·  {}  ·  {}", self.category, self.year, self.origin);
            buf.set_string(area.x, area.y + 1, &meta, Style::default().fg(p.faint));

            if ctx.selected {
                let hint = "[f]avorite";
                let hint_x = (area.x + area.width).saturating_sub(hint.len() as u16);
                buf.set_string(hint_x, area.y + 1, hint, Style::default().fg(p.faint));
            }
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

        let right = format!(
            "{} · {} · {}",
            self.category, self.designation, self.distance
        );
        let gap = 3u16;

        // Left side: title · description (truncated to fit before right side)
        buf.set_string(area.x, area.y, &self.title, Style::default().fg(title_fg));
        let sep_x = area.x + self.title.len() as u16;
        buf.set_string(sep_x, area.y, " · ", Style::default().fg(p.faint));
        let desc_x = sep_x + 3;
        let max_desc =
            (area.x + area.width).saturating_sub(desc_x + right.len() as u16 + gap) as usize;
        let desc = truncate_desc(&self.description, max_desc);
        buf.set_string(desc_x, area.y, &desc, Style::default().fg(desc_fg));

        // Right side: category · designation · distance
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

        let right = format!(
            "{} · {} · {}",
            self.category, self.designation, self.distance
        );
        let gap = 3u16;

        // Left: title · description (truncated to fit before right side)
        buf.set_string(area.x, area.y, &self.title, Style::default().fg(title_fg));
        let sep_x = area.x + self.title.len() as u16;
        buf.set_string(sep_x, area.y, " · ", Style::default().fg(p.faint));
        let desc_x = sep_x + 3;
        let max_desc =
            (area.x + area.width).saturating_sub(desc_x + right.len() as u16 + gap) as usize;
        let desc = truncate_desc(&self.description, max_desc);
        buf.set_string(desc_x, area.y, &desc, Style::default().fg(desc_fg));

        // Right: category · designation · distance
        let right = format!(
            "{} · {} · {}",
            self.category, self.designation, self.distance
        );
        let right_x = (area.x + area.width).saturating_sub(right.len() as u16);
        buf.set_string(right_x, area.y, &right, Style::default().fg(meta_fg));
    }
}

/// Custom header with a highlighted title bar and item count.
struct StyledHeader {
    title: String,
}

impl StyledHeader {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl ListHeader for StyledHeader {
    fn height(&self) -> u16 {
        4 // title + blank + count + blank
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListHeaderContext) {
        let p = &ctx.palette;

        // Title with background highlight
        let title_style = Style::default().fg(p.on_highlight).bg(p.highlight);
        let padded = format!(" {} ", self.title);
        buf.set_string(area.x + 2, area.y, &padded, title_style);

        // Count
        if area.height > 2 {
            let count_text = format!("{} items", ctx.total_items);
            buf.set_string(
                area.x + 2,
                area.y + 2,
                &count_text,
                Style::default().fg(p.muted),
            );
        }
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
    "Styled Header",
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
            DetailItem::new(
                "Andromeda",
                "Nearest spiral galaxy, on collision course with the Milky Way",
                "Galaxy",
                "M31",
                "2.537 Mly",
            ),
            DetailItem::new(
                "Crab Nebula",
                "Supernova remnant with a rapidly spinning pulsar at its core",
                "Nebula",
                "M1",
                "6,500 ly",
            ),
            DetailItem::new(
                "Pillars of Creation",
                "Towering columns of gas and dust in the Eagle Nebula",
                "Nebula",
                "M16",
                "7,000 ly",
            ),
            DetailItem::new(
                "Sagittarius A*",
                "Supermassive black hole at the center of our galaxy",
                "Black Hole",
                "Sgr A*",
                "26,000 ly",
            ),
            DetailItem::new(
                "Europa",
                "Icy moon of Jupiter with a subsurface ocean twice Earth's volume",
                "Moon",
                "Jupiter II",
                "628.3 Mkm",
            ),
            DetailItem::new(
                "Olympus Mons",
                "Largest volcano in the solar system, 2.5× Everest's height",
                "Volcano",
                "Mars",
                "225 Mkm",
            ),
            DetailItem::new(
                "Titan",
                "Saturn's largest moon with dense atmosphere and methane lakes",
                "Moon",
                "Saturn VI",
                "1.2 Bkm",
            ),
            DetailItem::new(
                "Whirlpool Galaxy",
                "Grand-design spiral interacting with a smaller companion",
                "Galaxy",
                "M51",
                "23 Mly",
            ),
            DetailItem::new(
                "Io",
                "Most volcanically active body in the solar system",
                "Moon",
                "Jupiter I",
                "628.3 Mkm",
            ),
            DetailItem::new(
                "Horsehead Nebula",
                "Dark nebula silhouetted against glowing hydrogen gas",
                "Nebula",
                "B33",
                "1,375 ly",
            ),
            DetailItem::new(
                "Enceladus",
                "Tiny moon venting water ice geysers from its south pole",
                "Moon",
                "Saturn II",
                "1.27 Bkm",
            ),
            DetailItem::new(
                "Sombrero Galaxy",
                "Bright nucleus with an unusually large central bulge and dust lane",
                "Galaxy",
                "M104",
                "31.1 Mly",
            ),
        ];
        let separated: Vec<SeparatedDetailItem> = detail
            .iter()
            .map(|d| {
                SeparatedDetailItem::new(&d.title, &d.description, &d.category, &d.year, &d.origin)
            })
            .collect();
        let dense: Vec<DenseItem> = detail
            .iter()
            .map(|d| DenseItem::new(&d.title, &d.description, &d.category, &d.year, &d.origin))
            .collect();
        let simple: Vec<SimpleItem> = [
            "Rust",
            "Go",
            "Python",
            "TypeScript",
            "Zig",
            "Elixir",
            "Haskell",
            "OCaml",
            "Swift",
            "Kotlin",
            "Ruby",
            "C",
            "Lua",
            "Julia",
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

    fn toggle_favorite(&mut self) {
        let selected = self.state_mut().selected();
        if let DemoKind::Detail { items, .. } = &mut self.demos[self.current]
            && selected < items.len()
        {
            items[selected].toggle_favorite();
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
                KeyCode::Char('f') => app.toggle_favorite(),
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
        area.width.saturating_sub(4).min(120),
        area.height.saturating_sub(2),
    );

    let help = Help::default().bindings(vec![
        Binding::new("j/k", "select"),
        Binding::new("g/G", "top/bottom"),
        Binding::new("h/l", "page"),
        Binding::new("f", "favorite"),
        Binding::new("n/p", "next/prev example"),
        Binding::new("q", "quit"),
    ]);
    let help_height = help.required_height();

    let example_name = EXAMPLE_NAMES[app.current];

    // Cap the list so pagination is always visible
    let max_list_height = content_area.height * 4 / 5;

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
            let header = StyledHeader::new("Development Boards");
            let list = List::new(items.as_slice())
                .header(&header)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Detail { items, state } => {
            let header = DefaultHeader::new("Celestial Objects").show_count(true);
            let list = List::new(items.as_slice())
                .header(&header)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Separated { items, state } => {
            let header = DefaultHeader::new("Celestial Objects").show_count(true);
            let list = List::new(items.as_slice())
                .header(&header)
                .item_spacing(0)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Dense { items, state } => {
            let header = DefaultHeader::new("Celestial Objects").show_count(true);
            let list = List::new(items.as_slice())
                .header(&header)
                .item_spacing(0)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Simple { items, state } => {
            let header = DefaultHeader::new("Programming Languages").show_count(true);
            let list = List::new(items.as_slice())
                .header(&header)
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
        DemoKind::Compact { items, state } => {
            let header = DefaultHeader::new("Programming Languages").show_count(true);
            let list = List::new(items.as_slice())
                .header(&header)
                .item_spacing(0)
                .selection_indicator(">")
                .palette(palette.clone());
            StatefulWidget::render(&list, list_area, frame.buffer_mut(), state);
        }
    }

    // Help
    Widget::render(&help, help_area, frame.buffer_mut());
}
