//! Input example — cycles through different input configurations.
//!
//! Demonstrates title, description, placeholder, password mode, custom prompt,
//! validation error, and validation success.
//!
//! Run with: cargo run --example input

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::help::{Binding, Help};
use ratatui_cheese::input::{Input, InputState};
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorStyles};
use ratatui_cheese::theme::Palette;

struct Variant {
    name: &'static str,
    setup: fn(&Palette) -> (Input<'static>, InputState),
    live_validate: bool,
}

const VARIANTS: &[Variant] = &[
    Variant {
        name: "Basic",

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
        live_validate: false,
        setup: |p| {
            let input = Input::new("PIN")
                .description("Enter your 5-digit PIN.")
                .palette(p);
            let state = InputState::new().char_limit(5).validator(|v| {
                if v.len() != 5 { Err("PIN must be exactly 5 digits".into()) } else { Ok(None) }
            });
            (input, state)
        },
    },
];

struct Model {
    variant_index: usize,
    paginator_state: PaginatorState,
    input: Input<'static>,
    input_state: InputState,
    palette: Palette,
}

impl Model {
    fn new() -> Self {
        let palette = Palette::charm();
        let (input, mut input_state) = (VARIANTS[0].setup)(&palette);
        input_state.set_focused(true);
        Self {
            variant_index: 0,
            paginator_state: PaginatorState::new(VARIANTS.len(), 1),
            input,
            input_state,
            palette,
        }
    }

    fn variant(&self) -> &Variant {
        &VARIANTS[self.variant_index]
    }

    fn switch_to(&mut self, index: usize) {
        self.variant_index = index;
        let (input, mut input_state) = (VARIANTS[index].setup)(&self.palette);
        input_state.set_focused(true);
        self.input = input;
        self.input_state = input_state;
        self.paginator_state = PaginatorState::new(VARIANTS.len(), 1);
        for _ in 0..index {
            self.paginator_state.next_page();
        }
    }

    fn next(&mut self) {
        self.switch_to((self.variant_index + 1) % VARIANTS.len());
    }

    fn prev(&mut self) {
        self.switch_to(if self.variant_index == 0 {
            VARIANTS.len() - 1
        } else {
            self.variant_index - 1
        });
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
                KeyCode::Tab | KeyCode::Right => m.next(),
                KeyCode::BackTab | KeyCode::Left => m.prev(),
                KeyCode::Enter => {
                    m.input_state.validate();
                }
                KeyCode::Backspace => {
                    m.input_state.delete_before();
                    if m.variant().live_validate {
                        m.input_state.validate();
                    }
                }
                KeyCode::Delete => {
                    m.input_state.delete_at();
                    if m.variant().live_validate {
                        m.input_state.validate();
                    }
                }
                KeyCode::Char(c) => {
                    m.input_state.insert_char(c);
                    if m.variant().live_validate {
                        m.input_state.validate();
                    }
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
            Binding::new("←/→", "variant"),
            Binding::new("enter", "validate"),
            Binding::new("esc", "quit"),
        ])
        .styles(ratatui_cheese::help::HelpStyles::from_palette(palette));
    let help_height = help.required_height();

    let variant = &VARIANTS[m.variant_index];

    let [title_area, _, input_area, _, paginator_area, _, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(help_height),
    ])
    .areas(content_area);

    // Variant name as heading
    let heading = ratatui::text::Line::styled(
        variant.name,
        ratatui::style::Style::default()
            .fg(palette.foreground)
            .add_modifier(ratatui::style::Modifier::BOLD),
    );
    Widget::render(heading, title_area, frame.buffer_mut());

    // Input
    StatefulWidget::render(&m.input, input_area, frame.buffer_mut(), &mut m.input_state);

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
