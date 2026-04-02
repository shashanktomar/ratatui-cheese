use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui_cheese::list::{
    DefaultHeader, List, ListHeader, ListHeaderContext, ListItem, ListItemContext, ListState,
};
use ratatui_cheese::theme::Palette;

use super::Component;

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

/// Two-line item: title + description (the default Bubbles style).
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

/// Detail item: title · description on line 1, metadata on line 2.
struct DetailItem {
    title: String,
    description: String,
    category: String,
    designation: String,
    distance: String,
    favorited: bool,
}

impl DetailItem {
    fn new(title: &str, desc: &str, category: &str, designation: &str, distance: &str) -> Self {
        Self {
            title: title.to_string(),
            description: desc.to_string(),
            category: category.to_string(),
            designation: designation.to_string(),
            distance: distance.to_string(),
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

        // Line 2: category · designation · distance  (+ hint when selected)
        if area.height > 1 {
            let meta = format!(
                "{}  ·  {}  ·  {}",
                self.category, self.designation, self.distance
            );
            buf.set_string(area.x, area.y + 1, &meta, Style::default().fg(p.faint));

            if ctx.selected {
                let hint = "[f]avorite";
                let hint_x = (area.x + area.width).saturating_sub(hint.len() as u16);
                buf.set_string(hint_x, area.y + 1, hint, Style::default().fg(p.faint));
            }
        }
    }
}

/// Single-line item with separator: content + right-aligned meta + ─ divider.
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

/// Dense single-line item: title · description left, metadata right.
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
        let right_x = (area.x + area.width).saturating_sub(right.len() as u16);
        buf.set_string(right_x, area.y, &right, Style::default().fg(meta_fg));
    }
}

/// Single-line item.
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

/// Numbered single-line item with index prefix.
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

// ---------------------------------------------------------------------------
// Demo configurations
// ---------------------------------------------------------------------------

const EXAMPLE_NAMES: &[&str] = &[
    "Styled Header",
    "Detail Cards",
    "With Line Separator",
    "Dense",
    "Simple Items",
    "Numbered",
];

fn two_line_items() -> Vec<TwoLineItem> {
    vec![
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
    ]
}

fn celestial_items() -> Vec<DetailItem> {
    vec![
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
    ]
}

fn simple_items() -> Vec<SimpleItem> {
    vec![
        SimpleItem::new("Rust"),
        SimpleItem::new("Go"),
        SimpleItem::new("Python"),
        SimpleItem::new("TypeScript"),
        SimpleItem::new("Zig"),
        SimpleItem::new("Elixir"),
        SimpleItem::new("Haskell"),
        SimpleItem::new("OCaml"),
        SimpleItem::new("Swift"),
        SimpleItem::new("Kotlin"),
        SimpleItem::new("Ruby"),
        SimpleItem::new("C"),
        SimpleItem::new("Lua"),
        SimpleItem::new("Julia"),
    ]
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Wraps the different demo types so we can cycle between them.
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
    Numbered {
        items: Vec<NumberedItem>,
        state: ListState,
    },
}

pub struct ListComponent {
    demos: Vec<DemoKind>,
    current: usize,
}

impl ListComponent {
    pub fn new() -> Self {
        let two_line = two_line_items();
        let detail = celestial_items();
        let separated: Vec<SeparatedDetailItem> = detail
            .iter()
            .map(|d| {
                SeparatedDetailItem::new(
                    &d.title,
                    &d.description,
                    &d.category,
                    &d.designation,
                    &d.distance,
                )
            })
            .collect();
        let dense: Vec<DenseItem> = detail
            .iter()
            .map(|d| {
                DenseItem::new(
                    &d.title,
                    &d.description,
                    &d.category,
                    &d.designation,
                    &d.distance,
                )
            })
            .collect();
        let simple = simple_items();
        let numbered: Vec<NumberedItem> = simple.iter().map(|s| NumberedItem::new(&s.0)).collect();

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
            DemoKind::Numbered {
                state: ListState::new(numbered.len()),
                items: numbered,
            },
        ];

        Self { demos, current: 0 }
    }

    fn next_demo(&mut self) {
        self.current = (self.current + 1) % self.demos.len();
    }

    fn item_count(&self) -> usize {
        match &self.demos[self.current] {
            DemoKind::TwoLine { items, .. } => items.len(),
            DemoKind::Detail { items, .. } => items.len(),
            DemoKind::Separated { items, .. } => items.len(),
            DemoKind::Dense { items, .. } => items.len(),
            DemoKind::Simple { items, .. } => items.len(),
            DemoKind::Numbered { items, .. } => items.len(),
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
            | DemoKind::Numbered { state, .. } => state,
        }
    }
}

impl Component for ListComponent {
    fn name(&self) -> &str {
        "List"
    }

    fn handle_key(&mut self, key: KeyCode) {
        let count = self.item_count();
        match key {
            KeyCode::Char('c') => self.next_demo(),
            KeyCode::Char('j') | KeyCode::Down => self.state_mut().select_next(count, false),
            KeyCode::Char('k') | KeyCode::Up => self.state_mut().select_prev(count, false),
            KeyCode::Char('l') | KeyCode::Right => self.state_mut().next_page(count),
            KeyCode::Char('h') | KeyCode::Left => self.state_mut().prev_page(count),
            KeyCode::Char('f') => self.toggle_favorite(),
            KeyCode::Char('g') => self.state_mut().select_first_on_page(),
            KeyCode::Char('G') => self.state_mut().select_last_on_page(count),
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, palette: &Palette, area: Rect, focused: bool) {
        let example_name = EXAMPLE_NAMES[self.current];
        let border_style = if focused {
            Style::default().fg(palette.foreground)
        } else {
            Style::default().fg(palette.faint)
        };
        let block = Block::bordered()
            .title(format!(" List: {example_name} "))
            .border_style(border_style)
            .padding(Padding::new(2, 2, 1, 0));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        // Cap list to roughly half so pagination is always visible
        let max_list_height = inner.height / 2;

        let (list_area, help_area) = if inner.height > 3 {
            let [la, _, ha] = Layout::vertical([
                Constraint::Max(max_list_height),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .areas(inner);
            (la, Some(ha))
        } else {
            (inner, None)
        };

        // Render the current demo
        match &mut self.demos[self.current] {
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
            DemoKind::Numbered { items, state } => {
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
        if let Some(help_area) = help_area {
            let help = Line::from(Span::styled(
                "j/k: select • g/G: top/bottom • h/l: page • c: cycle",
                Style::default().fg(palette.faint),
            ));
            help.render(help_area, frame.buffer_mut());
        }
    }
}
