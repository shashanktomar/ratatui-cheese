//! Select example — cycles through different select configurations.
//!
//! Demonstrates title, description, custom cursor, pre-selection,
//! validation, and long lists.
//!
//! Run with: cargo run --example select

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::help::{Binding, Help, HelpStyles};
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorStyles};
use ratatui_cheese::select::{Select, SelectOption, SelectState, SelectStyles};
use ratatui_cheese::theme::Palette;

struct VariantData {
    name: &'static str,
    title: &'static str,
    description: Option<&'static str>,
    options: Vec<SelectOption<'static>>,
    cursor_indicator: &'static str,
    initial_cursor: usize,
    live_validate: bool,
    state: SelectState,
}

fn make_variants() -> Vec<VariantData> {
    vec![
        VariantData {
            name: "Basic",
            title: "Destination",
            description: Some("Where would you like to go?"),
            options: vec![
                "Mars".into(),
                "Europa".into(),
                "Titan".into(),
                "Enceladus".into(),
            ],
            cursor_indicator: ">",
            initial_cursor: 0,
            live_validate: false,
            state: SelectState::new(4),
        },
        VariantData {
            name: "Custom cursor",
            title: "Star class",
            description: None,
            options: vec![
                "O — Blue supergiant".into(),
                "B — Blue-white".into(),
                "G — Yellow dwarf".into(),
                "M — Red dwarf".into(),
            ],
            cursor_indicator: "→",
            initial_cursor: 0,
            live_validate: false,
            state: SelectState::new(4),
        },
        VariantData {
            name: "Pre-selected",
            title: "Orbital altitude",
            description: Some("Select your orbit."),
            options: vec![
                "LEO (400 km)".into(),
                "MEO (20,000 km)".into(),
                "GEO (35,786 km)".into(),
                "HEO (500–40,000 km)".into(),
            ],
            cursor_indicator: ">",
            initial_cursor: 2,
            live_validate: false,
            state: SelectState::new(4),
        },
        VariantData {
            name: "Disabled options",
            title: "Destination",
            description: Some("Some destinations are restricted."),
            options: vec![
                "Mars".into(),
                SelectOption::new("Europa").enabled(false),
                "Titan".into(),
                SelectOption::new("Enceladus").enabled(false),
            ],
            cursor_indicator: ">",
            initial_cursor: 0,
            live_validate: false,
            state: {
                let mut s = SelectState::new(4);
                s.set_enabled(1, false);
                s.set_enabled(3, false);
                s
            },
        },
        VariantData {
            name: "Validation",
            title: "Destination",
            description: Some("Where would you like to go?"),
            options: vec![
                "Mars".into(),
                "Europa".into(),
                "Titan".into(),
                "Enceladus".into(),
            ],
            cursor_indicator: ">",
            initial_cursor: 1,
            live_validate: true,
            state: SelectState::new(4).validator(|i| {
                if i == 1 {
                    Err("Europa is off limits by treaty".into())
                } else {
                    Ok(Some("Mission approved".into()))
                }
            }),
        },
    ]
}

struct Model {
    variants: Vec<VariantData>,
    variant_index: usize,
    paginator_state: PaginatorState,
    palette: Palette,
}

impl Model {
    fn new() -> Self {
        let mut variants = make_variants();
        let count = variants.len();
        let v = &mut variants[0];
        v.state.set_cursor(v.initial_cursor);
        v.state.set_focused(true);
        Self {
            variants,
            variant_index: 0,
            paginator_state: PaginatorState::new(count, 1),
            palette: Palette::charm(),
        }
    }

    fn current(&self) -> &VariantData {
        &self.variants[self.variant_index]
    }

    fn current_mut(&mut self) -> &mut VariantData {
        &mut self.variants[self.variant_index]
    }

    fn switch_to(&mut self, index: usize) {
        self.variant_index = index;
        let mut new_variants = make_variants();
        let v = &mut new_variants[index];
        v.state.set_cursor(v.initial_cursor);
        v.state.set_focused(true);
        if v.name == "Validation" {
            v.state.validate();
        }
        self.variants[index] = new_variants.swap_remove(index);
        self.paginator_state = PaginatorState::new(self.variants.len(), 1);
        for _ in 0..index {
            self.paginator_state.next_page();
        }
    }

    fn next_variant(&mut self) {
        self.switch_to((self.variant_index + 1) % self.variants.len());
    }

    fn prev_variant(&mut self) {
        let len = self.variants.len();
        self.switch_to(if self.variant_index == 0 { len - 1 } else { self.variant_index - 1 });
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
                KeyCode::Esc => return Ok(()),
                KeyCode::Char('h') | KeyCode::Left => m.prev_variant(),
                KeyCode::Char('l') | KeyCode::Right => m.next_variant(),
                KeyCode::Char('j') | KeyCode::Down => {
                    let live = m.current().live_validate;
                    m.current_mut().state.next();
                    if live {
                        m.current_mut().state.validate();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let live = m.current().live_validate;
                    m.current_mut().state.prev();
                    if live {
                        m.current_mut().state.validate();
                    }
                }
                KeyCode::Enter => {
                    m.current_mut().state.validate();
                }
                _ => {}
            }
        }
    }
}

fn view(frame: &mut Frame, m: &mut Model) {
    let palette = &m.palette;
    let area = frame.area();

    let content_area = Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );

    let help = Help::default()
        .bindings(vec![
            Binding::new("h/l", "variant"),
            Binding::new("j/k", "navigate"),
            Binding::new("enter", "validate"),
            Binding::new("esc", "quit"),
        ])
        .styles(HelpStyles::from_palette(palette));
    let help_height = help.required_height();

    let [title_area, _, select_area, _, paginator_area, _, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(help_height),
    ])
    .areas(content_area);

    let idx = m.variant_index;

    // Variant name
    let heading = Line::styled(
        m.variants[idx].name,
        Style::default()
            .fg(palette.foreground)
            .add_modifier(Modifier::BOLD),
    );
    Widget::render(heading, title_area, frame.buffer_mut());

    // Clone options to avoid borrow conflict with state
    let opts = m.variants[idx].options.clone();
    let title = m.variants[idx].title;
    let cursor_ind = m.variants[idx].cursor_indicator;
    let desc = m.variants[idx].description;

    let mut select = Select::new(title, &opts)
        .cursor_indicator(cursor_ind)
        .styles(SelectStyles::from_palette(palette));
    if let Some(d) = desc {
        select = select.description(d);
    }
    StatefulWidget::render(
        &select,
        select_area,
        frame.buffer_mut(),
        &mut m.variants[idx].state,
    );

    // Paginator
    let paginator = Paginator::default().styles(PaginatorStyles::from_palette(palette));
    StatefulWidget::render(
        &paginator,
        paginator_area,
        frame.buffer_mut(),
        &mut m.paginator_state,
    );

    // Help
    Widget::render(&help, help_area, frame.buffer_mut());
}
