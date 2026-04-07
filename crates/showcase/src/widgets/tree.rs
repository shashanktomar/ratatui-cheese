use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Widget};
use ratatui_cheese::theme::Palette;
use ratatui_cheese::tree::{Mode, Tree, TreeGroup, TreeItem, TreeState, TreeStyles};

use super::Component;

pub struct TreeComponent {
    tree_state: TreeState,
    groups: Vec<TreeGroup>,
    mode: Mode,
}

impl TreeComponent {
    pub fn new() -> Self {
        let groups = sample_groups();
        let tree_state = TreeState::new(groups.len());
        Self {
            tree_state,
            groups,
            mode: Mode::default(),
        }
    }
}

impl Component for TreeComponent {
    fn name(&self) -> &str {
        "Tree"
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('j') | KeyCode::Down => self.tree_state.select_next(&self.groups),
            KeyCode::Char('k') | KeyCode::Up => self.tree_state.select_prev(&self.groups),
            KeyCode::Char('l') | KeyCode::Right => {
                self.tree_state.expand(self.tree_state.selected().0);
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.tree_state.collapse(self.tree_state.selected().0);
            }
            KeyCode::Char('L') => self.tree_state.expand_all(),
            KeyCode::Char('H') => self.tree_state.collapse_all(),
            KeyCode::Char('c') => self.mode = self.mode.cycle(),
            KeyCode::Enter | KeyCode::Char(' ') => self.tree_state.toggle_selected(),
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
            .title(format!(" Tree: {} ", self.mode))
            .border_style(border_style)
            .padding(Padding::new(2, 2, 1, 0));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let (tree_area, hint_area) = if inner.height > 3 {
            let [ta, _, ha] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(inner);
            (ta, Some(ha))
        } else {
            (inner, None)
        };

        let tree = Tree::default()
            .groups(self.groups.clone())
            .styles(TreeStyles::from_palette(palette))
            .mode(self.mode);

        frame.render_stateful_widget(&tree, tree_area, &mut self.tree_state);

        if let Some(hint_area) = hint_area {
            let hint = Line::from(Span::styled(
                "j/k: navigate  h/l: fold/unfold  H/L: all  enter: toggle  c: mode",
                Style::default().fg(palette.faint),
            ));
            hint.render(hint_area, frame.buffer_mut());
        }
    }
}

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
        ]),
        TreeGroup::new(TreeItem::new("Media").count(243)).children(vec![
            TreeItem::new("Screenshots").count(58),
            TreeItem::new("Recordings").count(12),
            TreeItem::new("Exports").count(34),
        ]),
        TreeGroup::new(TreeItem::new("Bookmarks").count(31)),
        TreeGroup::new(TreeItem::new("Notes").count(56)),
        TreeGroup::new(TreeItem::new("Archive").count(412)),
        TreeGroup::new(TreeItem::new("Trash")),
    ]
}
