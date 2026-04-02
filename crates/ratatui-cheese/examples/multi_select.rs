//! Multi-select example — cycles through different multi-select configurations.
//!
//! Demonstrates title, description, limits, pre-selection, disabled options,
//! and validation.
//!
//! Run with: cargo run --example multi_select

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::help::{Binding, Help, HelpStyles};
use ratatui_cheese::multi_select::{
    MultiSelect, MultiSelectOption, MultiSelectState, MultiSelectStyles,
};
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorStyles};
use ratatui_cheese::theme::Palette;

struct VariantData {
    name: &'static str,
    title: &'static str,
    description: Option<&'static str>,
    options: Vec<MultiSelectOption<'static>>,
    limit: Option<usize>,
    initial_cursor: usize,
    initial_selected: Vec<usize>,
    live_validate: bool,
    state: MultiSelectState,
}

fn make_variants() -> Vec<VariantData> {
    vec![
        VariantData {
            name: "Basic",
            title: "Instruments",
            description: Some("Select instruments for your mission."),
            options: vec![
                "Spectrometer".into(),
                "Magnetometer".into(),
                "Gravimeter".into(),
                "Radiometer".into(),
                "Altimeter".into(),
            ],
            limit: None,
            initial_cursor: 0,
            initial_selected: vec![],
            live_validate: false,
            state: MultiSelectState::new(5),
        },
        VariantData {
            name: "With limit",
            title: "Experiments",
            description: Some("Choose up to 3 experiments."),
            options: vec![
                "Dark matter detection".into(),
                "Cosmic ray analysis".into(),
                "Gravitational lensing".into(),
                "Neutrino oscillation".into(),
                "Pulsar timing".into(),
            ],
            limit: Some(3),
            initial_cursor: 0,
            initial_selected: vec![],
            live_validate: false,
            state: MultiSelectState::new(5),
        },
        VariantData {
            name: "Pre-selected",
            title: "Systems",
            description: Some("Review pre-configured systems."),
            options: vec![
                "Navigation".into(),
                "Life support".into(),
                "Communications".into(),
                "Propulsion".into(),
                "Shields".into(),
            ],
            limit: None,
            initial_cursor: 0,
            initial_selected: vec![0, 1, 2],
            live_validate: false,
            state: MultiSelectState::new(5),
        },
        VariantData {
            name: "Disabled options",
            title: "Destinations",
            description: Some("Some destinations are restricted."),
            options: vec![
                "Mars".into(),
                MultiSelectOption::new("Europa").enabled(false),
                "Titan".into(),
                MultiSelectOption::new("Enceladus").enabled(false),
                "Ganymede".into(),
            ],
            limit: None,
            initial_cursor: 0,
            initial_selected: vec![],
            live_validate: false,
            state: MultiSelectState::new(5),
        },
        VariantData {
            name: "Validation",
            title: "Crew roles",
            description: Some("Assign at least one role."),
            options: vec![
                "Commander".into(),
                "Pilot".into(),
                "Engineer".into(),
                "Scientist".into(),
                "Medic".into(),
            ],
            limit: None,
            initial_cursor: 0,
            initial_selected: vec![],
            live_validate: true,
            state: MultiSelectState::new(5).validator(|sel| {
                let count = sel.iter().filter(|&&s| s).count();
                if count == 0 {
                    Err("Select at least one role".into())
                } else {
                    Ok(Some(format!(
                        "{count} role{} assigned",
                        if count == 1 { "" } else { "s" }
                    )))
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
        Self::init_variant(&mut variants[0]);
        Self {
            variants,
            variant_index: 0,
            paginator_state: PaginatorState::new(count, 1),
            palette: Palette::charm(),
        }
    }

    fn init_variant(v: &mut VariantData) {
        v.state.set_cursor(v.initial_cursor);
        for &idx in &v.initial_selected {
            v.state.set_selected(idx, true);
        }
        v.state.set_focused(true);
    }

    fn switch_to(&mut self, index: usize) {
        self.variant_index = index;
        let mut new_variants = make_variants();
        let v = &mut new_variants[index];
        Self::init_variant(v);
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
                    let v = &m.variants[m.variant_index];
                    let opts = v.options.clone();
                    let live = v.live_validate;
                    m.variants[m.variant_index].state.next(&opts);
                    if live {
                        m.variants[m.variant_index].state.validate();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let v = &m.variants[m.variant_index];
                    let opts = v.options.clone();
                    let live = v.live_validate;
                    m.variants[m.variant_index].state.prev(&opts);
                    if live {
                        m.variants[m.variant_index].state.validate();
                    }
                }
                KeyCode::Char(' ') => {
                    let limit = m.variants[m.variant_index].limit;
                    let live = m.variants[m.variant_index].live_validate;
                    m.variants[m.variant_index].state.toggle_current(limit);
                    if live {
                        m.variants[m.variant_index].state.validate();
                    }
                }
                KeyCode::Enter => {
                    m.variants[m.variant_index].state.validate();
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
            Binding::new("space", "toggle"),
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
    let desc = m.variants[idx].description;
    let limit = m.variants[idx].limit;

    let mut multi = MultiSelect::new(title, &opts).styles(MultiSelectStyles::from_palette(palette));
    if let Some(d) = desc {
        multi = multi.description(d);
    }
    if let Some(l) = limit {
        multi = multi.limit(l);
    }
    StatefulWidget::render(
        &multi,
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
