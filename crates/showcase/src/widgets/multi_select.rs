use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui_cheese::multi_select::{
    MultiSelect, MultiSelectOption, MultiSelectState, MultiSelectStyles,
};
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorStyles};
use ratatui_cheese::theme::Palette;

use super::Component;

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

fn init_variant(v: &mut VariantData) {
    v.state.set_cursor(v.initial_cursor);
    for &idx in &v.initial_selected {
        v.state.set_selected(idx, true);
    }
    v.state.set_focused(true);
}

pub struct MultiSelectComponent {
    variants: Vec<VariantData>,
    variant_index: usize,
    paginator_state: PaginatorState,
}

impl MultiSelectComponent {
    pub fn new() -> Self {
        let mut variants = make_variants();
        let count = variants.len();
        init_variant(&mut variants[0]);
        Self {
            variants,
            variant_index: 0,
            paginator_state: PaginatorState::new(count, 1),
        }
    }

    fn reset_variant(&mut self, index: usize) {
        let mut new_variants = make_variants();
        let v = &mut new_variants[index];
        init_variant(v);
        if v.name == "Validation" {
            v.state.validate();
        }
        self.variants[index] = new_variants.swap_remove(index);
    }

    fn switch_to(&mut self, index: usize) {
        self.variant_index = index;
        self.reset_variant(index);
        self.paginator_state = PaginatorState::new(self.variants.len(), 1);
        for _ in 0..index {
            self.paginator_state.next_page();
        }
    }
}

impl Component for MultiSelectComponent {
    fn name(&self) -> &str {
        "Multi Select"
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('l') | KeyCode::Right => {
                self.switch_to((self.variant_index + 1) % self.variants.len());
            }
            KeyCode::Char('h') | KeyCode::Left => {
                let len = self.variants.len();
                self.switch_to(if self.variant_index == 0 {
                    len - 1
                } else {
                    self.variant_index - 1
                });
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let opts = self.variants[self.variant_index].options.clone();
                let live = self.variants[self.variant_index].live_validate;
                self.variants[self.variant_index].state.next(&opts);
                if live {
                    self.variants[self.variant_index].state.validate();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let opts = self.variants[self.variant_index].options.clone();
                let live = self.variants[self.variant_index].live_validate;
                self.variants[self.variant_index].state.prev(&opts);
                if live {
                    self.variants[self.variant_index].state.validate();
                }
            }
            KeyCode::Char(' ') => {
                let limit = self.variants[self.variant_index].limit;
                let live = self.variants[self.variant_index].live_validate;
                self.variants[self.variant_index]
                    .state
                    .toggle_current(limit);
                if live {
                    self.variants[self.variant_index].state.validate();
                }
            }
            KeyCode::Enter => {
                self.variants[self.variant_index].state.validate();
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, palette: &Palette, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(palette.foreground)
        } else {
            Style::default().fg(palette.faint)
        };
        let block = Block::bordered()
            .title(" Multi Select ")
            .border_style(border_style)
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let idx = self.variant_index;

        let [title_area, _, select_area, _, paginator_area, _, help_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);

        // Variant name
        let heading = Line::styled(
            self.variants[idx].name,
            Style::default()
                .fg(palette.foreground)
                .add_modifier(Modifier::BOLD),
        );
        heading.render(title_area, frame.buffer_mut());

        // Clone options to avoid borrow conflict with state
        let opts = self.variants[idx].options.clone();
        let title = self.variants[idx].title;
        let desc = self.variants[idx].description;
        let limit = self.variants[idx].limit;

        let mut multi =
            MultiSelect::new(title, &opts).styles(MultiSelectStyles::from_palette(palette));
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
            &mut self.variants[idx].state,
        );

        // Paginator
        let paginator = Paginator::default().styles(PaginatorStyles::from_palette(palette));
        StatefulWidget::render(
            &paginator,
            paginator_area,
            frame.buffer_mut(),
            &mut self.paginator_state,
        );

        // Help
        if help_area.height > 0 {
            let help = Line::from(Span::styled(
                "j/k: navigate • space: toggle • h/l: variant • enter: validate",
                Style::default().fg(palette.faint),
            ));
            help.render(help_area, frame.buffer_mut());
        }
    }
}
