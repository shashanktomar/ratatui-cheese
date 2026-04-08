use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorStyles};
use ratatui_cheese::select::{Select, SelectOption, SelectState, SelectStyles};
use ratatui_cheese::theme::Palette;

use super::Component;

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

pub struct SelectComponent {
    variants: Vec<VariantData>,
    variant_index: usize,
    paginator_state: PaginatorState,
}

impl SelectComponent {
    pub fn new() -> Self {
        let mut variants = make_variants();
        let count = variants.len();
        let v = &mut variants[0];
        v.state.set_cursor(v.initial_cursor);
        v.state.set_focused(true);
        Self {
            variants,
            variant_index: 0,
            paginator_state: PaginatorState::new(count, 1),
        }
    }

    fn reset_variant(&mut self, index: usize) {
        let mut new_variants = make_variants();
        let v = &mut new_variants[index];
        v.state.set_cursor(v.initial_cursor);
        v.state.set_focused(true);
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

impl Component for SelectComponent {
    fn name(&self) -> &str {
        "Select"
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
                let live = self.variants[self.variant_index].live_validate;
                self.variants[self.variant_index].state.next();
                if live {
                    self.variants[self.variant_index].state.validate();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let live = self.variants[self.variant_index].live_validate;
                self.variants[self.variant_index].state.prev();
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
            .title(" Select ")
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
        let cursor_ind = self.variants[idx].cursor_indicator;
        let desc = self.variants[idx].description;

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
                "j/k: navigate • h/l: variant • enter: validate",
                Style::default().fg(palette.faint),
            ));
            help.render(help_area, frame.buffer_mut());
        }
    }
}
