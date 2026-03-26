use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::help::{Binding, Help};
use ratatui_cheese::theme::Palette;
use ratatui_cheese::tree::{Mode, Tree, TreeGroup, TreeItem, TreeState, TreeStyles};

fn sample_groups() -> Vec<TreeGroup> {
    vec![
        TreeGroup::new(TreeItem::new("Inbox")).children(vec![
            TreeItem::new("Welcome to the app"),
            TreeItem::new("Your weekly digest"),
        ]),
        TreeGroup::new(TreeItem::new("Projects").count(12)).children(vec![
            TreeItem::new("Website Redesign").count(4),
            TreeItem::new("Mobile App v2").count(3),
            TreeItem::new("API Migration").count(5),
        ]),
        TreeGroup::new(TreeItem::new("Documents").count(87)).children(vec![
            TreeItem::new("Q4 Planning Document"),
            TreeItem::new("Architecture Decision Records"),
            TreeItem::new("Team Onboarding Guide"),
            TreeItem::new("Infrastructure Runbook and Procedures"),
            TreeItem::new("Quarterly Performance Review Template"),
            TreeItem::new("Release Notes - Version History"),
            TreeItem::new("Security Compliance Checklist"),
            TreeItem::new("Budget Forecast and Allocation"),
        ]),
        TreeGroup::new(TreeItem::new("Media").count(243)).children(vec![
            TreeItem::new("Screenshots").count(58),
            TreeItem::new("Recordings").count(12),
            TreeItem::new("Exports").count(34),
            TreeItem::new("Thumbnails").count(139),
        ]),
        TreeGroup::new(TreeItem::new("Bookmarks").count(31)),
        TreeGroup::new(TreeItem::new("Notes").count(56)).children(vec![
            TreeItem::new("Meeting Notes"),
            TreeItem::new("Ideas"),
            TreeItem::new("Reading List"),
        ]),
        TreeGroup::new(TreeItem::new("Archive").count(412)),
        TreeGroup::new(TreeItem::new("Trash")),
        TreeGroup::new(TreeItem::new("Shared with Me").count(7)),
        TreeGroup::new(TreeItem::new("Favorites").count(15)),
    ]
}

struct App {
    tree_state: TreeState,
    groups: Vec<TreeGroup>,
    mode: Mode,
    help_show_all: bool,
}

impl App {
    fn new() -> Self {
        let groups = sample_groups();
        let state = TreeState::new(groups.len());
        Self {
            tree_state: state,
            groups,
            mode: Mode::default(),
            help_show_all: false,
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
                KeyCode::Char('j') | KeyCode::Down => app.tree_state.select_next(&app.groups),
                KeyCode::Char('k') | KeyCode::Up => app.tree_state.select_prev(&app.groups),
                KeyCode::Char('l') | KeyCode::Right => {
                    app.tree_state.expand(app.tree_state.selected().0);
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    app.tree_state.collapse(app.tree_state.selected().0);
                }
                KeyCode::Char('L') => app.tree_state.expand_all(),
                KeyCode::Char('H') => app.tree_state.collapse_all(),
                KeyCode::Char('c') => app.mode = app.mode.cycle(),
                KeyCode::Char('?') => app.help_show_all = !app.help_show_all,
                KeyCode::Enter | KeyCode::Char(' ') => app.tree_state.toggle_selected(),
                _ => {}
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let [left, _] = Layout::horizontal([Constraint::Max(40), Constraint::Fill(1)]).areas(area);

    let help = Help::default()
        .bindings(vec![
            Binding::new("?", "toggle help"),
            Binding::new("q", "quit"),
        ])
        .binding_groups(vec![vec![
            Binding::new("j/k", "navigate"),
            Binding::new("h/l", "fold/unfold"),
            Binding::new("H/L", "expand/collapse all"),
            Binding::new("enter", "toggle"),
            Binding::new("c", "cycle count mode"),
            Binding::new("?", "toggle help"),
            Binding::new("q", "quit"),
        ]])
        .show_all(app.help_show_all);

    let help_height = help.required_height();
    let [tree_area, _, help_area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(help_height),
        Constraint::Length(1),
    ])
    .areas(left);

    let block = Block::bordered()
        .title(format!(" Tree: {} ", app.mode))
        .padding(Padding::horizontal(1));
    let inner = block.inner(tree_area);
    frame.render_widget(block, tree_area);

    let tree = Tree::default()
        .groups(app.groups.clone())
        .styles(TreeStyles::from_palette(&Palette::charm()))
        .mode(app.mode);

    StatefulWidget::render(&tree, inner, frame.buffer_mut(), &mut app.tree_state);

    // Indent help by 2 to align with tree content
    let help_area = ratatui::layout::Rect::new(
        help_area.x + 2,
        help_area.y,
        help_area.width.saturating_sub(2),
        help_area.height,
    );
    Widget::render(&help, help_area, frame.buffer_mut());
}
