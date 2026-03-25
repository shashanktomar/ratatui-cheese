use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Widget};
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::help::{Binding, Help};
use ratatui_cheese::spinner::{Spinner, SpinnerState, SpinnerType};
use ratatui_cheese::tree::{Mode, Tree, TreeGroup, TreeItem, TreeState, TreeStyles};

// -- Colors --

const COLOR_ACCENT: Color = Color::Indexed(69);
const COLOR_TEXT: Color = Color::Indexed(252);
const COLOR_HELP: Color = Color::Indexed(241);
const COLOR_CUSTOM_ACCENT: Color = Color::Indexed(212);

// -- Widget registry --
// Each widget in the showcase gets an entry here. As new widgets are added,
// just append to this list.

const WIDGETS: &[&str] = &["Spinner", "Help", "Tree"];

// -- Spinner config --

struct SpinnerEntry {
    name: &'static str,
    state: SpinnerState,
    spinner: Spinner,
}

fn spinner_entries() -> Vec<SpinnerEntry> {
    let style = Style::default().fg(COLOR_ACCENT);

    let presets: &[(SpinnerType, &str)] = &[
        (SpinnerType::Line, "Line"),
        (SpinnerType::Dot, "Dot"),
        (SpinnerType::MiniDot, "MiniDot"),
        (SpinnerType::Jump, "Jump"),
        (SpinnerType::Pulse, "Pulse"),
        (SpinnerType::Points, "Points"),
        (SpinnerType::Globe, "Globe"),
        (SpinnerType::Moon, "Moon"),
        (SpinnerType::Monkey, "Monkey"),
        (SpinnerType::Meter, "Meter"),
        (SpinnerType::Hamburger, "Hamburger"),
        (SpinnerType::Ellipsis, "Ellipsis"),
    ];

    let mut entries: Vec<SpinnerEntry> = presets
        .iter()
        .map(|(t, name)| SpinnerEntry {
            name,
            state: SpinnerState::new(*t),
            spinner: Spinner::default().style(style),
        })
        .collect();

    entries.push(SpinnerEntry {
        name: "Custom",
        state: SpinnerState::custom(vec!["⠇".into(), "⠸".into()], Duration::from_millis(300)),
        spinner: Spinner::default().style(Style::default().fg(COLOR_CUSTOM_ACCENT)),
    });

    entries
}

// -- App state --

// -- Help config --

fn help_short_bindings() -> Vec<Binding> {
    vec![Binding::new("?", "toggle help"), Binding::new("q", "quit")]
}

fn help_full_bindings() -> Vec<Vec<Binding>> {
    vec![
        vec![
            Binding::new("↑/k", "move up"),
            Binding::new("↓/j", "move down"),
            Binding::new("←/h", "move left"),
            Binding::new("→/l", "move right"),
        ],
        vec![Binding::new("?", "toggle help"), Binding::new("q", "quit")],
    ]
}

// -- App state --

struct App {
    // Widget list selection
    selected_widget: usize,

    // Spinner state
    spinner_index: usize,
    entries: Vec<SpinnerEntry>,
    last_tick: Instant,

    // Help state
    help_show_all: bool,

    // Tree state
    tree_state: TreeState,
    tree_groups: Vec<TreeGroup>,
    tree_mode: Mode,
}

impl App {
    fn new() -> Self {
        let tree_groups = showcase_tree_groups();
        let tree_state = TreeState::new(tree_groups.len());
        Self {
            selected_widget: 0,
            spinner_index: 0,
            entries: spinner_entries(),
            last_tick: Instant::now(),
            help_show_all: false,
            tree_state,
            tree_groups,
            tree_mode: Mode::default(),
        }
    }

    fn select_widget_next(&mut self) {
        self.selected_widget = (self.selected_widget + 1) % WIDGETS.len();
    }

    fn select_widget_prev(&mut self) {
        self.selected_widget = if self.selected_widget == 0 {
            WIDGETS.len() - 1
        } else {
            self.selected_widget - 1
        };
    }

    fn set_spinner(&mut self, index: usize) {
        self.spinner_index = index;
        self.last_tick = Instant::now();
    }

    fn next_spinner(&mut self) {
        let index = (self.spinner_index + 1) % self.entries.len();
        self.set_spinner(index);
    }

    fn prev_spinner(&mut self) {
        let index = if self.spinner_index == 0 {
            self.entries.len() - 1
        } else {
            self.spinner_index - 1
        };
        self.set_spinner(index);
    }

    fn current_entry(&mut self) -> &mut SpinnerEntry {
        &mut self.entries[self.spinner_index]
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_tick;
        self.last_tick = now;
        self.entries[self.spinner_index].state.tick(dt);
    }
}

// -- Main --

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

        app.tick();

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                // Widget list navigation
                KeyCode::Tab => app.select_widget_next(),
                KeyCode::BackTab => app.select_widget_prev(),
                KeyCode::Char('j') | KeyCode::Down => match WIDGETS[app.selected_widget] {
                    "Tree" => app.tree_state.select_next(&app.tree_groups),
                    _ => app.select_widget_next(),
                },
                KeyCode::Char('k') | KeyCode::Up => match WIDGETS[app.selected_widget] {
                    "Tree" => app.tree_state.select_prev(&app.tree_groups),
                    _ => app.select_widget_prev(),
                },
                // Widget-specific controls
                KeyCode::Char('l') | KeyCode::Right => match WIDGETS[app.selected_widget] {
                    "Spinner" => app.next_spinner(),
                    "Tree" => app.tree_state.expand(app.tree_state.selected().0),
                    _ => {}
                },
                KeyCode::Char('h') | KeyCode::Left => match WIDGETS[app.selected_widget] {
                    "Spinner" => app.prev_spinner(),
                    "Tree" => app.tree_state.collapse(app.tree_state.selected().0),
                    _ => {}
                },
                KeyCode::Char('L') => {
                    if WIDGETS[app.selected_widget] == "Tree" {
                        app.tree_state.expand_all();
                    }
                }
                KeyCode::Char('H') => {
                    if WIDGETS[app.selected_widget] == "Tree" {
                        app.tree_state.collapse_all();
                    }
                }
                KeyCode::Char('?') => {
                    if WIDGETS[app.selected_widget] == "Help" {
                        app.help_show_all = !app.help_show_all;
                    }
                }
                KeyCode::Char('c') => {
                    if WIDGETS[app.selected_widget] == "Tree" {
                        app.tree_mode = app.tree_mode.cycle();
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if WIDGETS[app.selected_widget] == "Tree" {
                        app.tree_state.toggle_selected();
                    }
                }
                _ => {}
            }
        }
    }
}

// -- Drawing --

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Two-column layout: sidebar (fixed) | detail (fill)
    let [sidebar_area, detail_area] =
        Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(area);

    draw_sidebar(frame, app, sidebar_area);
    draw_detail(frame, app, detail_area);
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .title(" Widgets ")
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    for (i, name) in WIDGETS.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let style = if i == app.selected_widget {
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(COLOR_TEXT)
        };
        let prefix = if i == app.selected_widget { "▸ " } else { "  " };
        let line = Line::from(Span::styled(format!("{prefix}{name}"), style));
        let line_area = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        line.render(line_area, frame.buffer_mut());
    }
}

fn draw_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    match WIDGETS[app.selected_widget] {
        "Spinner" => draw_spinner_detail(frame, app, area),
        "Help" => draw_help_detail(frame, app, area),
        "Tree" => draw_tree_detail(frame, app, area),
        _ => {}
    }
}

fn draw_spinner_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let entry = app.current_entry();

    let block = Block::bordered()
        .title(format!(" Spinner: {} ", entry.name))
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.is_empty() {
        return;
    }

    // Spinner + "Spinning..." on the first line
    let gap = " ";

    let spinner_area = Rect::new(inner.x, inner.y, 10, 1);
    frame.render_stateful_widget(&entry.spinner, spinner_area, &mut entry.state);

    let frames = entry.state.frames();
    let spinner_width = frames
        .iter()
        .map(|f| unicode_display_width(f))
        .max()
        .unwrap_or(0);
    let text_x = inner.x + spinner_width + gap.len() as u16;
    let text = Span::styled("Spinning...", Style::default().fg(COLOR_TEXT));
    let text_area = Rect::new(
        text_x,
        inner.y,
        inner.width.saturating_sub(text_x - inner.x),
        1,
    );
    Widget::render(text, text_area, frame.buffer_mut());

    // Help text
    if inner.height > 2 {
        let help = Line::from(Span::styled(
            "h/l, ←/→: change spinner",
            Style::default().fg(COLOR_HELP),
        ));
        let help_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
        help.render(help_area, frame.buffer_mut());
    }
}

fn draw_help_detail(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.help_show_all { " Help: Full " } else { " Help: Short " };
    let block = Block::bordered()
        .title(title)
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.is_empty() {
        return;
    }

    // Help widget
    let help = Help::default()
        .bindings(help_short_bindings())
        .binding_groups(help_full_bindings())
        .show_all(app.help_show_all);
    let help_height = help.required_height();
    let help_area = Rect::new(inner.x, inner.y, inner.width, help_height.min(inner.height));
    Widget::render(&help, help_area, frame.buffer_mut());

    // Hint text
    let hint_y = inner.y + help_height + 1;
    if hint_y < inner.y + inner.height {
        let hint = Line::from(Span::styled(
            "?: toggle short/full help",
            Style::default().fg(COLOR_HELP),
        ));
        let hint_area = Rect::new(inner.x, hint_y, inner.width, 1);
        hint.render(hint_area, frame.buffer_mut());
    }
}

fn showcase_tree_groups() -> Vec<TreeGroup> {
    vec![
        TreeGroup::new(TreeItem::new("Favorites (old)")).children(vec![
            TreeItem::new("HN Best"),
            TreeItem::new("Reddit Programming"),
        ]),
        TreeGroup::new(TreeItem::new("Programming").count(44)).children(vec![
            TreeItem::new("Hacker News"),
            TreeItem::new("Lobste.rs"),
        ]),
        TreeGroup::new(TreeItem::new("Engineering Blogs").count(123)).children(vec![
            TreeItem::new("Netflix Tech Blog"),
            TreeItem::new("Cloudflare Blog"),
        ]),
        TreeGroup::new(TreeItem::new("K8s Ecosystem").count(107)).children(vec![
            TreeItem::new("Cloud Native Now").count(46),
            TreeItem::new("Istio Blog and News").count(9),
            TreeItem::new("Kubernetes Blog"),
        ]),
        TreeGroup::new(TreeItem::new("Other Clouds").count(13)),
        TreeGroup::new(TreeItem::new("Tech Blogs").count(163)),
        TreeGroup::new(TreeItem::new("t.ai").count(110)),
        TreeGroup::new(TreeItem::new("t.startups")),
    ]
}

fn draw_tree_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::bordered()
        .title(format!(" Tree: {} ", app.tree_mode))
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.is_empty() {
        return;
    }

    // Reserve last row for hint text
    let (tree_area, hint_area) = if inner.height > 2 {
        let [ta, ha] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);
        (ta, Some(ha))
    } else {
        (inner, None)
    };

    let tree = Tree::default()
        .groups(app.tree_groups.clone())
        .styles(TreeStyles::dark())
        .mode(app.tree_mode);

    frame.render_stateful_widget(&tree, tree_area, &mut app.tree_state);

    if let Some(hint_area) = hint_area {
        let hint = Line::from(Span::styled(
            "j/k: navigate  h/l: fold/unfold  H/L: all  tab: switch widget",
            Style::default().fg(COLOR_HELP),
        ));
        hint.render(hint_area, frame.buffer_mut());
    }
}

/// Returns the display width of a string in terminal cells.
fn unicode_display_width(s: &str) -> u16 {
    use ratatui::text::Line;
    Line::raw(s).width() as u16
}
