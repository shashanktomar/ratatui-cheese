use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::help::{Binding, Help};
use ratatui_cheese::paginator::{Paginator, PaginatorMode, PaginatorState, PaginatorStyles};
use ratatui_cheese::theme::Palette;

const PER_PAGE: usize = 5;

struct App {
    items: Vec<String>,
    paginator_state: PaginatorState,
}

impl App {
    fn new() -> Self {
        let items: Vec<String> = (1..=100).map(|i| format!("Item {i}")).collect();
        let state = PaginatorState::new(items.len(), PER_PAGE);
        Self {
            items,
            paginator_state: state,
        }
    }
}

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
                KeyCode::Char('l') | KeyCode::Right => app.paginator_state.next_page(),
                KeyCode::Char('h') | KeyCode::Left => app.paginator_state.prev_page(),
                _ => {}
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let palette = Palette::charm();
    let area = frame.area();

    // Indent the content area
    let content_area = ratatui::layout::Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );

    let help = Help::default().bindings(vec![
        Binding::new("h/l", "←/→ page"),
        Binding::new("q", "quit"),
    ]);
    let help_height = help.required_height();

    let [title_area, _, items_area, _, paginator_area, _, help_area] = Layout::vertical([
        Constraint::Length(1),                   // title
        Constraint::Length(1),                   // blank
        Constraint::Length(PER_PAGE as u16 * 2), // items (2 lines each: text + blank)
        Constraint::Length(1),                   // blank
        Constraint::Length(1),                   // paginator
        Constraint::Length(1),                   // blank
        Constraint::Length(help_height),         // help
    ])
    .areas(content_area);

    // Title
    frame.buffer_mut().set_string(
        title_area.x,
        title_area.y,
        "Paginator Example",
        ratatui::style::Style::default()
            .fg(palette.foreground)
            .bold(),
    );

    // Items
    let (start, end) = app.paginator_state.get_slice_bounds(app.items.len());
    for (i, item) in app.items[start..end].iter().enumerate() {
        let y = items_area.y + (i as u16 * 2);
        if y >= items_area.y + items_area.height {
            break;
        }
        frame.buffer_mut().set_string(
            items_area.x,
            y,
            format!("• {item}"),
            ratatui::style::Style::default().fg(palette.foreground),
        );
    }

    // Paginator
    let paginator = Paginator::default()
        .mode(PaginatorMode::Dots)
        .styles(PaginatorStyles::from_palette(&palette));

    StatefulWidget::render(
        &paginator,
        paginator_area,
        frame.buffer_mut(),
        &mut app.paginator_state,
    );

    // Help
    Widget::render(&help, help_area, frame.buffer_mut());
}
