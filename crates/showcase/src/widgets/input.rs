use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui_cheese::input::{Input, InputState};
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorStyles};
use ratatui_cheese::theme::Palette;

use super::Component;

struct Variant {
    name: &'static str,
    setup: fn(&Palette) -> (Input<'static>, InputState),
    char_limit: Option<usize>,
    live_validate: bool,
}

const VARIANTS: &[Variant] = &[
    Variant {
        name: "Basic",
        char_limit: None,
        live_validate: false,
        setup: |p| {
            let input = Input::new("What's your name?")
                .description("For when your order is ready.")
                .placeholder("Enter name...")
                .palette(p);
            let state = InputState::new().validator(|v| {
                if v.is_empty() { Err("Name is required".into()) } else { Ok(None) }
            });
            (input, state)
        },
    },
    Variant {
        name: "No description",
        char_limit: None,
        live_validate: false,
        setup: |p| {
            let input = Input::new("Email")
                .placeholder("you@example.com")
                .palette(p);
            let state = InputState::new().validator(|v| {
                if v.is_empty() {
                    Err("Email is required".into())
                } else if !v.contains('@') {
                    Err("Must be a valid email".into())
                } else {
                    Ok(None)
                }
            });
            (input, state)
        },
    },
    Variant {
        name: "Custom prompt",
        char_limit: None,
        live_validate: false,
        setup: |p| {
            let input = Input::new("Search")
                .description("Find anything.")
                .prompt("→")
                .placeholder("Type to search...")
                .palette(p);
            let state = InputState::new();
            (input, state)
        },
    },
    Variant {
        name: "Password mode",
        char_limit: None,
        live_validate: false,
        setup: |p| {
            let input = Input::new("Password")
                .description("Keep it secret.")
                .placeholder("Enter password...")
                .password_mode(true)
                .palette(p);
            let state = InputState::new().validator(|v| {
                if v.len() < 4 { Err("Must be at least 4 characters".into()) } else { Ok(None) }
            });
            (input, state)
        },
    },
    Variant {
        name: "With value",
        char_limit: None,
        live_validate: false,
        setup: |p| {
            let input = Input::new("Package")
                .description("Already filled in.")
                .palette(p);
            let mut state = InputState::new().validator(|v| {
                if v.is_empty() { Err("Required".into()) } else { Ok(None) }
            });
            state.set_value("ratatui-cheese".into());
            state.end();
            (input, state)
        },
    },
    Variant {
        name: "Validation",
        char_limit: None,
        live_validate: false,
        setup: |p| {
            let input = Input::new("Email")
                .description("We'll never share your email.")
                .placeholder("you@example.com")
                .palette(p);
            let mut state = InputState::new().validator(|v| {
                if v.is_empty() {
                    Err("Email is required".into())
                } else if !v.contains('@') {
                    Err("Must be a valid email address".into())
                } else {
                    Ok(Some("Valid email".into()))
                }
            });
            state.set_value("not-an-email".into());
            state.end();
            state.validate();
            (input, state)
        },
    },
    Variant {
        name: "Live validation",
        char_limit: None,
        live_validate: true,
        setup: |p| {
            let input = Input::new("ISBN")
                .description("Validates on every keystroke.")
                .placeholder("978...")
                .palette(p);
            let state = InputState::new().validator(|v| {
                if v.is_empty() {
                    Err("Enter an ISBN".into())
                } else if v.len() < 13 {
                    Err(format!("{} digits, need 13", v.len()))
                } else if v.len() > 13 {
                    Err("Too many digits".into())
                } else {
                    Ok(Some("Valid ISBN".into()))
                }
            });
            (input, state)
        },
    },
    Variant {
        name: "Char limit (5)",
        char_limit: Some(5),
        live_validate: false,
        setup: |p| {
            let input = Input::new("PIN")
                .description("Enter your 5-digit PIN.")
                .char_limit(5)
                .palette(p);
            let state = InputState::new().validator(|v| {
                if v.len() != 5 { Err("PIN must be exactly 5 digits".into()) } else { Ok(None) }
            });
            (input, state)
        },
    },
];

pub struct InputComponent {
    variant_index: usize,
    paginator_state: PaginatorState,
    input: Input<'static>,
    input_state: InputState,
    last_palette: Palette,
}

impl InputComponent {
    pub fn new() -> Self {
        let palette = Palette::dark();
        let (input, mut input_state) = (VARIANTS[0].setup)(&palette);
        input_state.set_focused(true);
        Self {
            variant_index: 0,
            paginator_state: PaginatorState::new(VARIANTS.len(), 1),
            input,
            input_state,
            last_palette: palette,
        }
    }

    fn variant(&self) -> &Variant {
        &VARIANTS[self.variant_index]
    }

    fn switch_to(&mut self, index: usize, palette: &Palette) {
        self.variant_index = index;
        let (input, mut input_state) = (VARIANTS[index].setup)(palette);
        input_state.set_focused(true);
        self.input = input;
        self.input_state = input_state;
        self.paginator_state = PaginatorState::new(VARIANTS.len(), 1);
        for _ in 0..index {
            self.paginator_state.next_page();
        }
    }

    fn next(&mut self, palette: &Palette) {
        self.switch_to((self.variant_index + 1) % VARIANTS.len(), palette);
    }

    fn prev(&mut self, palette: &Palette) {
        let index = if self.variant_index == 0 {
            VARIANTS.len() - 1
        } else {
            self.variant_index - 1
        };
        self.switch_to(index, palette);
    }
}

impl Component for InputComponent {
    fn name(&self) -> &str {
        "Input"
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Right => self.next(&self.last_palette.clone()),
            KeyCode::Left => self.prev(&self.last_palette.clone()),
            KeyCode::Enter => {
                self.input_state.validate();
            }
            KeyCode::Backspace => {
                self.input_state.delete_before();
                if self.variant().live_validate {
                    self.input_state.validate();
                }
            }
            KeyCode::Delete => {
                self.input_state.delete_at();
                if self.variant().live_validate {
                    self.input_state.validate();
                }
            }
            KeyCode::Char(c) => {
                self.input_state
                    .insert_char_limited(c, self.variant().char_limit);
                if self.variant().live_validate {
                    self.input_state.validate();
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, palette: &Palette, area: Rect, focused: bool) {
        // Refresh input styles if palette changed
        if *palette != self.last_palette {
            self.last_palette = palette.clone();
            self.switch_to(self.variant_index, palette);
        }

        let border_style = if focused {
            Style::default().fg(palette.foreground)
        } else {
            Style::default().fg(palette.faint)
        };
        let block = Block::bordered()
            .title(" Input ")
            .border_style(border_style)
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let variant = &VARIANTS[self.variant_index];

        let [title_area, _, input_area, _, paginator_area, _, help_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(inner);

        // Variant name
        let heading = Line::styled(
            variant.name,
            Style::default()
                .fg(palette.foreground)
                .add_modifier(Modifier::BOLD),
        );
        heading.render(title_area, frame.buffer_mut());

        // Input
        StatefulWidget::render(
            &self.input,
            input_area,
            frame.buffer_mut(),
            &mut self.input_state,
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
                "←/→: variant • enter: validate",
                Style::default().fg(palette.faint),
            ));
            help.render(help_area, frame.buffer_mut());
        }
    }
}
