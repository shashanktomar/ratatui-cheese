use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};
use ratatui_cheese::help::{Binding, Help};
use ratatui_cheese::tree::{Mode, Tree, TreeGroup, TreeItem, TreeState, TreeStyles};

fn sample_groups() -> Vec<TreeGroup> {
    vec![
        TreeGroup::new(TreeItem::new("Favorites (old)")).children(vec![
            TreeItem::new("HN Best"),
            TreeItem::new("Reddit Programming"),
        ]),
        TreeGroup::new(TreeItem::new("Programming").count(44)).children(vec![
            TreeItem::new("Hacker News"),
            TreeItem::new("Lobste.rs"),
            TreeItem::new("Reddit r/rust"),
        ]),
        TreeGroup::new(TreeItem::new("Engineering Blogs").count(123)).children(vec![
            TreeItem::new("Netflix Technology Blog"),
            TreeItem::new("Cloudflare Blog and Announcements"),
            TreeItem::new("Discord Engineering Blog"),
            TreeItem::new("Uber Engineering Blog"),
            TreeItem::new("Stripe Engineering Blog"),
            TreeItem::new("Airbnb Engineering and Data Science"),
            TreeItem::new("Meta Engineering Blog"),
            TreeItem::new("Google Developers Blog"),
        ]),
        TreeGroup::new(TreeItem::new("K8s Ecosystem").count(107)).children(vec![
            TreeItem::new("Blog - Cloud Native Computing Foundation").count(43),
            TreeItem::new("Cloud Native Now").count(46),
            TreeItem::new("Istio Blog and News").count(9),
            TreeItem::new("Release notes from kubernetes").count(9),
            TreeItem::new("Kubernetes Blog"),
            TreeItem::new("Sysdig Blog | Use Cases, Thought Leadership").count(12),
            TreeItem::new("Technology Archives - The New Stack").count(31),
            TreeItem::new("Weaveworks"),
            TreeItem::new("Container Journal - Features and Columns").count(7),
            TreeItem::new("Giant Swarm Blog and News"),
            TreeItem::new("Rancher Labs Blog and Announcements").count(18),
        ]),
        TreeGroup::new(TreeItem::new("Other Clouds").count(13)),
        TreeGroup::new(TreeItem::new("Tech Blogs").count(163)).children(vec![
            TreeItem::new("TechCrunch"),
            TreeItem::new("Ars Technica"),
            TreeItem::new("The Verge"),
        ]),
        TreeGroup::new(TreeItem::new("t.master").count(39)),
        TreeGroup::new(TreeItem::new("t.ai").count(110)),
        TreeGroup::new(TreeItem::new("t.lang").count(15)),
        TreeGroup::new(TreeItem::new("t.startups")),
        TreeGroup::new(TreeItem::new("t.startups.asia").count(142)),
        TreeGroup::new(TreeItem::new("t.startups.aus").count(106)),
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

    let [left, _] =
        Layout::horizontal([Constraint::Max(40), Constraint::Fill(1)]).areas(area);

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
        .styles(TreeStyles::dark())
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
