//! A tree widget that displays expandable parent/child groups.
//!
//! Renders a vertically scrollable list of collapsible groups, each with a header
//! and optional children. Supports right-aligned counts, text truncation, icons,
//! and customizable chevron indicators.
//!
//! # Example
//!
//! ```rust
//! use ratatui_cheese::tree::{Tree, TreeGroup, TreeItem, TreeState};
//!
//! let tree = Tree::default()
//!     .groups(vec![
//!         TreeGroup::new(TreeItem::new("Fruits"))
//!             .children(vec![
//!                 TreeItem::new("Apple"),
//!                 TreeItem::new("Banana"),
//!             ]),
//!         TreeGroup::new(TreeItem::new("Vegetables").count(42))
//!             .children(vec![
//!                 TreeItem::new("Carrot"),
//!             ]),
//!     ]);
//!
//! let mut state = TreeState::new(2);
//! ```

use std::fmt;

use crate::theme::Palette;
use crate::utils::display_width;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Styled};
use ratatui::widgets::{StatefulWidget, Widget};

// ---------------------------------------------------------------------------
// Count mode
// ---------------------------------------------------------------------------

/// Controls how counts are displayed next to items.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Mode {
    /// No counts shown.
    None,
    /// Only show counts that were explicitly set via `.count()`.
    Explicit,
    /// Show explicit counts, and for parents without an explicit count,
    /// fall back to the number of children.
    #[default]
    ParentFallback,
}

impl Mode {
    /// Returns the next mode in the cycle: None → Explicit → ParentFallback → None.
    #[must_use]
    pub fn cycle(self) -> Self {
        match self {
            Self::None => Self::Explicit,
            Self::Explicit => Self::ParentFallback,
            Self::ParentFallback => Self::None,
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "Simple"),
            Self::Explicit => write!(f, "Explicit Count"),
            Self::ParentFallback => write!(f, "Parent Fallback"),
        }
    }
}

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A single item in a tree — used for both parent headers and children.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TreeItem {
    text: String,
    icon: Option<String>,
    count: Option<usize>,
}

impl TreeItem {
    /// Creates a new tree item with the given text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            count: None,
        }
    }

    /// Sets an icon to display before the text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets a right-aligned count number.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Returns the display text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the icon, if set.
    pub fn get_icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Returns the explicit count, if set.
    pub fn get_count(&self) -> Option<usize> {
        self.count
    }
}

/// A parent group with a header item and optional children.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TreeGroup {
    header: TreeItem,
    children: Vec<TreeItem>,
}

impl TreeGroup {
    /// Creates a new group with the given header item and no children.
    #[must_use]
    pub fn new(header: TreeItem) -> Self {
        Self {
            header,
            children: Vec::new(),
        }
    }

    /// Sets the children for this group.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn children(mut self, children: Vec<TreeItem>) -> Self {
        self.children = children;
        self
    }

    /// Returns the header item.
    pub fn header(&self) -> &TreeItem {
        &self.header
    }

    /// Returns the children.
    pub fn children_slice(&self) -> &[TreeItem] {
        &self.children
    }

    /// Returns the effective count: explicit count if set, otherwise `children.len()`.
    pub fn effective_count(&self) -> usize {
        self.header.count.unwrap_or(self.children.len())
    }
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Styles for the different parts of the tree widget.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TreeStyles {
    /// Style for unselected parent/header rows.
    pub parent: Style,
    /// Style for unselected child rows.
    pub child: Style,
    /// Style for the currently selected/highlighted row.
    pub selected: Style,
    /// Style for the chevron indicator.
    pub chevron: Style,
    /// Style for the chevron when the parent is selected or expanded.
    pub chevron_active: Style,
    /// Style for the chevron when the parent has no children.
    pub chevron_dim: Style,
    /// Style for the right-aligned count numbers.
    pub count: Style,
    /// Style for child icons.
    pub icon: Style,
}

impl Default for TreeStyles {
    fn default() -> Self {
        Self::dark()
    }
}

impl TreeStyles {
    /// Creates styles from a [`Palette`].
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            parent: Style::default().fg(p.foreground),
            child: Style::default().fg(p.muted),
            selected: Style::default().fg(p.primary).bg(p.surface),
            chevron: Style::default().fg(p.muted),
            chevron_active: Style::default().fg(p.primary),
            chevron_dim: Style::default().fg(p.faint),
            count: Style::default().fg(p.muted),
            icon: Style::default(),
        }
    }

    /// Creates styles for dark backgrounds.
    #[must_use]
    pub fn dark() -> Self {
        Self::from_palette(&Palette::dark())
    }

    /// Creates styles for light backgrounds.
    #[must_use]
    pub fn light() -> Self {
        Self::from_palette(&Palette::light())
    }
}

// ---------------------------------------------------------------------------
// Widget struct
// ---------------------------------------------------------------------------

/// A tree widget that renders expandable parent/child groups.
///
/// `Tree` holds the data and appearance configuration. Mutable state (selection,
/// scroll offset, expanded groups) lives in [`TreeState`].
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::tree::{Tree, TreeGroup, TreeItem, TreeState};
///
/// let tree = Tree::default()
///     .groups(vec![
///         TreeGroup::new(TreeItem::new("Programming"))
///             .children(vec![TreeItem::new("Rust"), TreeItem::new("Go")]),
///     ]);
///
/// let mut state = TreeState::new(1);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Tree {
    groups: Vec<TreeGroup>,
    styles: TreeStyles,
    chevron_collapsed: String,
    chevron_expanded: String,
    indent: u16,
    mode: Mode,
    ellipsis: String,
    highlight_full_row: bool,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            styles: TreeStyles::default(),
            chevron_collapsed: ">".into(),
            chevron_expanded: "v".into(),
            indent: 4,
            mode: Mode::default(),
            ellipsis: "…".into(),
            highlight_full_row: true,
        }
    }
}

impl Tree {
    /// Sets the groups to display.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn groups(mut self, groups: Vec<TreeGroup>) -> Self {
        self.groups = groups;
        self
    }

    /// Sets the styles for the tree widget.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn styles(mut self, styles: TreeStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Sets the chevron string for collapsed groups.
    ///
    /// Default: `">"`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn chevron_collapsed(mut self, s: impl Into<String>) -> Self {
        self.chevron_collapsed = s.into();
        self
    }

    /// Sets the chevron string for expanded groups.
    ///
    /// Default: `"v"`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn chevron_expanded(mut self, s: impl Into<String>) -> Self {
        self.chevron_expanded = s.into();
        self
    }

    /// Sets the indentation width for children (in terminal cells).
    ///
    /// Default: `4`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn indent(mut self, indent: u16) -> Self {
        self.indent = indent;
        self
    }

    /// Sets the count display mode.
    ///
    /// Default: `Mode::ParentFallback`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the truncation ellipsis string.
    ///
    /// Default: `"…"`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn ellipsis(mut self, s: impl Into<String>) -> Self {
        self.ellipsis = s.into();
        self
    }

    /// Sets whether to highlight the full row or just the text area.
    ///
    /// Default: `true`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn highlight_full_row(mut self, full: bool) -> Self {
        self.highlight_full_row = full;
        self
    }
}

impl Styled for Tree {
    type Item = Self;

    fn style(&self) -> Style {
        self.styles.parent
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.styles.parent = style.into();
        self
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Mutable state for a [`Tree`] widget.
///
/// Tracks which groups are expanded, the currently selected item, and the
/// vertical scroll offset. Navigation methods take `&[TreeGroup]` to know
/// the tree structure without owning the data.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TreeState {
    expanded: Vec<bool>,
    selected: (usize, Option<usize>),
    offset: usize,
}

impl TreeState {
    /// Creates state for the given number of groups.
    ///
    /// All groups start collapsed with the first parent selected.
    pub fn new(num_groups: usize) -> Self {
        Self {
            expanded: vec![false; num_groups],
            selected: (0, None),
            offset: 0,
        }
    }

    /// Creates state with all groups expanded.
    pub fn all_expanded(num_groups: usize) -> Self {
        Self {
            expanded: vec![true; num_groups],
            selected: (0, None),
            offset: 0,
        }
    }

    /// Returns the currently selected position as `(group_index, child_index)`.
    ///
    /// `None` for the child index means the parent header is selected.
    pub fn selected(&self) -> (usize, Option<usize>) {
        self.selected
    }

    /// Returns the current scroll offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns whether the given group is expanded.
    pub fn is_expanded(&self, group_index: usize) -> bool {
        self.expanded.get(group_index).copied().unwrap_or(false)
    }

    /// Selects a specific item.
    pub fn select(&mut self, group: usize, child: Option<usize>) {
        self.selected = (group, child);
    }

    /// Moves selection to the next visible item. Stops at the last item.
    pub fn select_next(&mut self, groups: &[TreeGroup]) {
        let rows = build_visible_rows(groups, &self.expanded);
        let current_flat = flat_index_of(&rows, self.selected);
        if current_flat + 1 < rows.len() {
            let next = &rows[current_flat + 1];
            self.selected = (next.group_index(), next.child_index());
        }
    }

    /// Moves selection to the previous visible item. Stops at the first item.
    pub fn select_prev(&mut self, groups: &[TreeGroup]) {
        let rows = build_visible_rows(groups, &self.expanded);
        let current_flat = flat_index_of(&rows, self.selected);
        if current_flat > 0 {
            let prev = &rows[current_flat - 1];
            self.selected = (prev.group_index(), prev.child_index());
        }
    }

    /// Toggles expand/collapse of the group that the selection is in.
    pub fn toggle_selected(&mut self) {
        let group = self.selected.0;
        self.toggle(group);
    }

    /// Toggles a specific group.
    pub fn toggle(&mut self, group_index: usize) {
        if let Some(expanded) = self.expanded.get_mut(group_index) {
            if *expanded {
                self.collapse(group_index);
            } else {
                self.expand(group_index);
            }
        }
    }

    /// Expands a specific group.
    pub fn expand(&mut self, group_index: usize) {
        if let Some(expanded) = self.expanded.get_mut(group_index) {
            *expanded = true;
        }
    }

    /// Collapses a specific group.
    ///
    /// If a child of this group was selected, selection moves to the parent.
    pub fn collapse(&mut self, group_index: usize) {
        if let Some(expanded) = self.expanded.get_mut(group_index) {
            *expanded = false;
        }
        if self.selected.0 == group_index && self.selected.1.is_some() {
            self.selected = (group_index, None);
        }
    }

    /// Expands all groups.
    pub fn expand_all(&mut self) {
        self.expanded.fill(true);
    }

    /// Collapses all groups.
    ///
    /// If a child was selected, selection moves to its parent.
    pub fn collapse_all(&mut self) {
        self.expanded.fill(false);
        if self.selected.1.is_some() {
            self.selected = (self.selected.0, None);
        }
    }

    /// Adjusts the scroll offset so the selected item is visible in the viewport.
    fn ensure_visible(&mut self, viewport_height: usize, groups: &[TreeGroup]) {
        if viewport_height == 0 {
            return;
        }
        let rows = build_visible_rows(groups, &self.expanded);
        let selected_flat = flat_index_of(&rows, self.selected);

        if selected_flat < self.offset {
            self.offset = selected_flat;
        } else if selected_flat >= self.offset + viewport_height {
            self.offset = selected_flat - viewport_height + 1;
        }
    }
}

impl Default for TreeState {
    fn default() -> Self {
        Self::new(0)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// A flattened visible row for rendering purposes.
#[derive(Debug)]
enum VisibleRow<'a> {
    Parent {
        group_index: usize,
        item: &'a TreeItem,
        is_expanded: bool,
        effective_count: usize,
        has_children: bool,
    },
    Child {
        group_index: usize,
        child_index: usize,
        item: &'a TreeItem,
    },
}

impl VisibleRow<'_> {
    fn group_index(&self) -> usize {
        match self {
            Self::Parent { group_index, .. } | Self::Child { group_index, .. } => *group_index,
        }
    }

    fn child_index(&self) -> Option<usize> {
        match self {
            Self::Parent { .. } => None,
            Self::Child { child_index, .. } => Some(*child_index),
        }
    }
}

fn build_visible_rows<'a>(groups: &'a [TreeGroup], expanded: &[bool]) -> Vec<VisibleRow<'a>> {
    let mut rows = Vec::new();
    for (gi, group) in groups.iter().enumerate() {
        let is_expanded = expanded.get(gi).copied().unwrap_or(false);
        rows.push(VisibleRow::Parent {
            group_index: gi,
            item: &group.header,
            is_expanded,
            effective_count: group.effective_count(),
            has_children: !group.children.is_empty(),
        });
        if is_expanded {
            for (ci, child) in group.children.iter().enumerate() {
                rows.push(VisibleRow::Child {
                    group_index: gi,
                    child_index: ci,
                    item: child,
                });
            }
        }
    }
    rows
}

fn flat_index_of(rows: &[VisibleRow<'_>], selected: (usize, Option<usize>)) -> usize {
    rows.iter()
        .position(|r| r.group_index() == selected.0 && r.child_index() == selected.1)
        .unwrap_or(0)
}

/// Truncates text to fit within `max_width` terminal cells, appending ellipsis if needed.
fn truncate_with_ellipsis(text: &str, max_width: usize, ellipsis: &str) -> String {
    let text_w = display_width(text);
    if text_w <= max_width {
        return text.to_string();
    }
    let ellipsis_w = display_width(ellipsis);
    if max_width <= ellipsis_w {
        // Not enough space even for the ellipsis — just return what fits.
        return take_width(text, max_width);
    }
    let target = max_width - ellipsis_w;
    let mut result = take_width(text, target);
    result.push_str(ellipsis);
    result
}

/// Takes characters from `text` up to `max_width` terminal cells.
fn take_width(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut w = 0;
    for ch in text.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > max_width {
            break;
        }
        result.push(ch);
        w += cw;
    }
    result
}

// ---------------------------------------------------------------------------
// Trait implementations
// ---------------------------------------------------------------------------

// Widget for owned — delegates to &ref
impl Widget for Tree {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

// Widget for &ref — creates default state, delegates to StatefulWidget
impl Widget for &Tree {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TreeState::new(self.groups.len());
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// StatefulWidget for owned — delegates to &ref
impl StatefulWidget for Tree {
    type State = TreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

// StatefulWidget for &ref — the real render logic
impl StatefulWidget for &Tree {
    type State = TreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = area.intersection(buf.area);
        if area.is_empty() || self.groups.is_empty() {
            return;
        }

        let viewport_height = area.height as usize;
        state.ensure_visible(viewport_height, &self.groups);

        let rows = build_visible_rows(&self.groups, &state.expanded);
        let end = (state.offset + viewport_height).min(rows.len());

        for (vi, row) in rows[state.offset..end].iter().enumerate() {
            let y = area.y + vi as u16;
            let row_width = area.width as usize;

            let is_selected =
                row.group_index() == state.selected.0 && row.child_index() == state.selected.1;

            // Fill the row background if selected and highlight_full_row
            if is_selected && self.highlight_full_row {
                for x in area.x..area.x + area.width {
                    buf[(x, y)].set_style(self.styles.selected);
                }
            }

            match row {
                VisibleRow::Parent {
                    item,
                    is_expanded,
                    effective_count,
                    has_children,
                    ..
                } => {
                    let base_style =
                        if is_selected { self.styles.selected } else { self.styles.parent };
                    let chevron_style = if is_selected || *is_expanded {
                        let mut style = self.styles.chevron_active;
                        if is_selected {
                            style = style.bg(self.styles.selected.bg.unwrap_or_default());
                        }
                        style
                    } else if *has_children {
                        self.styles.chevron
                    } else {
                        self.styles.chevron_dim
                    };
                    let count_style =
                        if is_selected { self.styles.selected } else { self.styles.count };

                    let chevron =
                        if *is_expanded { &self.chevron_expanded } else { &self.chevron_collapsed };
                    let chevron_w = display_width(chevron);
                    // Layout: [chevron] [space] [text...] [space] [count]
                    let prefix_w = chevron_w + 1; // chevron + space

                    // Calculate count portion
                    let count_str = match self.mode {
                        Mode::None => String::new(),
                        Mode::Explicit => {
                            item.get_count().map(|c| format!("{c}")).unwrap_or_default()
                        }
                        Mode::ParentFallback => format!("{effective_count}"),
                    };
                    let count_gap = 2; // space between text and count
                    let count_w = if count_str.is_empty() {
                        0
                    } else {
                        display_width(&count_str) + count_gap
                    };

                    // Text available width
                    let text_max = row_width.saturating_sub(prefix_w + count_w);

                    // Render chevron
                    let mut x = area.x;
                    buf.set_string(x, y, chevron, chevron_style);
                    x += chevron_w as u16;
                    buf.set_string(x, y, " ", base_style);
                    x += 1;

                    // Render text (possibly truncated)
                    let display_text =
                        truncate_with_ellipsis(item.text(), text_max, &self.ellipsis);
                    buf.set_string(x, y, &display_text, base_style);

                    // Render count right-aligned
                    if !count_str.is_empty() {
                        let count_x = area.x + area.width - display_width(&count_str) as u16;
                        buf.set_string(count_x, y, &count_str, count_style);
                    }
                }
                VisibleRow::Child { item, .. } => {
                    let base_style =
                        if is_selected { self.styles.selected } else { self.styles.child };
                    let icon_style =
                        if is_selected { self.styles.selected } else { self.styles.icon };
                    let count_style =
                        if is_selected { self.styles.selected } else { self.styles.count };

                    let indent_w = self.indent as usize;

                    // Icon width
                    let icon_str = item.get_icon().unwrap_or("");
                    let icon_w = if icon_str.is_empty() {
                        0
                    } else {
                        display_width(icon_str) + 1 // icon + space
                    };

                    let prefix_w = indent_w + icon_w;

                    // Count portion (only if explicitly set on child, and mode allows it)
                    let count_str = match self.mode {
                        Mode::None => String::new(),
                        Mode::Explicit | Mode::ParentFallback => {
                            item.get_count().map(|c| format!("{c}")).unwrap_or_default()
                        }
                    };
                    let count_gap = 2;
                    let count_w = if count_str.is_empty() {
                        0
                    } else {
                        display_width(&count_str) + count_gap
                    };

                    let text_max = row_width.saturating_sub(prefix_w + count_w);

                    // Render indent
                    let mut x = area.x;
                    for _ in 0..self.indent.min(area.width) {
                        buf.set_string(x, y, " ", base_style);
                        x += 1;
                    }

                    // Render icon
                    if !icon_str.is_empty() {
                        buf.set_string(x, y, icon_str, icon_style);
                        x += display_width(icon_str) as u16;
                        buf.set_string(x, y, " ", base_style);
                        x += 1;
                    }

                    // Render text
                    let display_text =
                        truncate_with_ellipsis(item.text(), text_max, &self.ellipsis);
                    buf.set_string(x, y, &display_text, base_style);

                    // Render count right-aligned
                    if !count_str.is_empty() {
                        let count_x = area.x + area.width - display_width(&count_str) as u16;
                        buf.set_string(count_x, y, &count_str, count_style);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

    fn test_groups() -> Vec<TreeGroup> {
        vec![
            TreeGroup::new(TreeItem::new("Fruits"))
                .children(vec![TreeItem::new("Apple"), TreeItem::new("Banana")]),
            TreeGroup::new(TreeItem::new("Vegetables").count(42))
                .children(vec![TreeItem::new("Carrot")]),
            TreeGroup::new(TreeItem::new("Grains")),
        ]
    }

    #[track_caller]
    fn render_tree(tree: &Tree, state: &mut TreeState, area: Rect) -> Buffer {
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(tree, area, &mut buf, state);
        buf
    }

    // ---- Data model ----

    #[test]
    fn tree_item_new() {
        let item = TreeItem::new("Hello");
        assert_eq!(item.text(), "Hello");
        assert_eq!(item.get_icon(), None);
        assert_eq!(item.get_count(), None);
    }

    #[test]
    fn tree_item_with_icon_and_count() {
        let item = TreeItem::new("Test").icon("🍎").count(10);
        assert_eq!(item.text(), "Test");
        assert_eq!(item.get_icon(), Some("🍎"));
        assert_eq!(item.get_count(), Some(10));
    }

    #[test]
    fn tree_group_effective_count_uses_explicit() {
        let group = TreeGroup::new(TreeItem::new("G").count(99)).children(vec![TreeItem::new("a")]);
        assert_eq!(group.effective_count(), 99);
    }

    #[test]
    fn tree_group_effective_count_falls_back_to_children_len() {
        let group = TreeGroup::new(TreeItem::new("G"))
            .children(vec![TreeItem::new("a"), TreeItem::new("b")]);
        assert_eq!(group.effective_count(), 2);
    }

    #[test]
    fn tree_group_no_children_no_count() {
        let group = TreeGroup::new(TreeItem::new("G"));
        assert_eq!(group.effective_count(), 0);
    }

    // ---- State ----

    #[test]
    fn state_new_all_collapsed() {
        let state = TreeState::new(3);
        assert!(!state.is_expanded(0));
        assert!(!state.is_expanded(1));
        assert!(!state.is_expanded(2));
        assert_eq!(state.selected(), (0, None));
    }

    #[test]
    fn state_all_expanded() {
        let state = TreeState::all_expanded(3);
        assert!(state.is_expanded(0));
        assert!(state.is_expanded(1));
        assert!(state.is_expanded(2));
    }

    #[test]
    fn toggle_expands_collapsed_group() {
        let mut state = TreeState::new(2);
        state.toggle(0);
        assert!(state.is_expanded(0));
    }

    #[test]
    fn toggle_collapses_expanded_group() {
        let mut state = TreeState::all_expanded(2);
        state.toggle(0);
        assert!(!state.is_expanded(0));
    }

    #[test]
    fn collapse_moves_selection_from_child_to_parent() {
        let groups = test_groups();
        let mut state = TreeState::new(groups.len());
        state.expand(0);
        state.select(0, Some(1)); // select Banana
        state.collapse(0);
        assert_eq!(state.selected(), (0, None)); // moved to Fruits
    }

    #[test]
    fn select_next_all_collapsed() {
        let groups = test_groups();
        let mut state = TreeState::new(groups.len());
        // Fruits -> Vegetables -> Grains
        assert_eq!(state.selected(), (0, None));
        state.select_next(&groups);
        assert_eq!(state.selected(), (1, None));
        state.select_next(&groups);
        assert_eq!(state.selected(), (2, None));
    }

    #[test]
    fn select_next_stops_at_end() {
        let groups = test_groups();
        let mut state = TreeState::new(groups.len());
        state.select(2, None); // Grains (last)
        state.select_next(&groups);
        assert_eq!(state.selected(), (2, None)); // stays
    }

    #[test]
    fn select_prev_stops_at_start() {
        let groups = test_groups();
        let mut state = TreeState::new(groups.len());
        state.select_prev(&groups);
        assert_eq!(state.selected(), (0, None)); // stays
    }

    #[test]
    fn select_next_enters_expanded_children() {
        let groups = test_groups();
        let mut state = TreeState::new(groups.len());
        state.expand(0);
        // Fruits -> Apple -> Banana -> Vegetables -> ...
        state.select_next(&groups);
        assert_eq!(state.selected(), (0, Some(0))); // Apple
        state.select_next(&groups);
        assert_eq!(state.selected(), (0, Some(1))); // Banana
        state.select_next(&groups);
        assert_eq!(state.selected(), (1, None)); // Vegetables
    }

    #[test]
    fn select_next_skips_collapsed_children() {
        let groups = test_groups();
        let mut state = TreeState::new(groups.len());
        // All collapsed: Fruits -> Vegetables (skips Apple, Banana)
        state.select_next(&groups);
        assert_eq!(state.selected(), (1, None));
    }

    #[test]
    fn select_prev_navigates_into_expanded_children() {
        let groups = test_groups();
        let mut state = TreeState::new(groups.len());
        state.expand(0);
        state.select(1, None); // Vegetables
        state.select_prev(&groups);
        assert_eq!(state.selected(), (0, Some(1))); // Banana (last child of expanded Fruits)
    }

    // ---- Rendering ----

    #[test]
    fn render_empty_area() {
        let tree = Tree::default().groups(test_groups());
        let mut state = TreeState::new(3);
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 0, 0));
        assert_eq!(buf, Buffer::empty(Rect::new(0, 0, 0, 0)));
    }

    #[test]
    fn render_empty_groups() {
        let tree = Tree::default();
        let mut state = TreeState::new(0);
        let area = Rect::new(0, 0, 20, 5);
        let buf = render_tree(&tree, &mut state, area);
        assert_eq!(buf, Buffer::empty(area));
    }

    #[test]
    fn render_collapsed_parents_with_counts() {
        let tree = Tree::default()
            .groups(vec![
                TreeGroup::new(TreeItem::new("Fruits"))
                    .children(vec![TreeItem::new("Apple"), TreeItem::new("Banana")]),
                TreeGroup::new(TreeItem::new("Vegs").count(42)),
            ])
            .styles(TreeStyles {
                parent: Style::default(),
                child: Style::default(),
                selected: Style::default(),
                chevron: Style::default(),
                chevron_active: Style::default(),
                chevron_dim: Style::default(),
                count: Style::default(),
                icon: Style::default(),
            })
            .highlight_full_row(false);
        let mut state = TreeState::new(2);
        state.select(99, None); // deselect by pointing nowhere visible
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 20, 3));
        let expected = Buffer::with_lines(vec![
            "> Fruits           2",
            "> Vegs            42",
            "                    ",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn render_expanded_with_children() {
        let tree = Tree::default()
            .groups(vec![TreeGroup::new(TreeItem::new("Fruits")).children(
                vec![TreeItem::new("Apple"), TreeItem::new("Banana")],
            )])
            .styles(TreeStyles {
                parent: Style::default(),
                child: Style::default(),
                selected: Style::default(),
                chevron: Style::default(),
                chevron_active: Style::default(),
                chevron_dim: Style::default(),
                count: Style::default(),
                icon: Style::default(),
            })
            .highlight_full_row(false);
        let mut state = TreeState::new(1);
        state.expand(0);
        state.select(99, None); // deselect
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 20, 4));
        let expected = Buffer::with_lines(vec![
            "v Fruits           2",
            "    Apple           ",
            "    Banana          ",
            "                    ",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn render_no_counts() {
        let tree = Tree::default()
            .groups(vec![
                TreeGroup::new(TreeItem::new("Fruits")).children(vec![TreeItem::new("Apple")]),
            ])
            .mode(Mode::None)
            .styles(TreeStyles {
                parent: Style::default(),
                child: Style::default(),
                selected: Style::default(),
                chevron: Style::default(),
                chevron_active: Style::default(),
                chevron_dim: Style::default(),
                count: Style::default(),
                icon: Style::default(),
            })
            .highlight_full_row(false);
        let mut state = TreeState::new(1);
        state.expand(0);
        state.select(99, None);
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 20, 3));
        let expected = Buffer::with_lines(vec![
            "v Fruits            ",
            "    Apple           ",
            "                    ",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn render_text_truncation_with_count() {
        let tree = Tree::default()
            .groups(vec![TreeGroup::new(
                TreeItem::new("Very Long Name Here").count(42),
            )])
            .styles(TreeStyles {
                parent: Style::default(),
                child: Style::default(),
                selected: Style::default(),
                chevron: Style::default(),
                chevron_active: Style::default(),
                chevron_dim: Style::default(),
                count: Style::default(),
                icon: Style::default(),
            })
            .highlight_full_row(false);
        let mut state = TreeState::new(1);
        state.select(99, None);
        // Width 18: "> " (2) + text + " " + "42" (3) = 15 chars for text
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 18, 1));
        // prefix=2 ("> "), count_w=3 (" 42"), text_max=13
        // "Very Long Name Here" (19) truncated to 13: "Very Long Na…" (12+1=13)
        // Count "42" right-aligned: positions 16,17 (width 18 - 2 = 16)
        assert_eq!(buf[(16, 0)].symbol(), "4");
        assert_eq!(buf[(17, 0)].symbol(), "2");
        assert_eq!(buf[(0, 0)].symbol(), ">");
    }

    #[test]
    fn render_children_with_icons() {
        let tree = Tree::default()
            .groups(vec![TreeGroup::new(TreeItem::new("Fruits")).children(
                vec![TreeItem::new("Apple").icon("A"), TreeItem::new("Banana")],
            )])
            .styles(TreeStyles {
                parent: Style::default(),
                child: Style::default(),
                selected: Style::default(),
                chevron: Style::default(),
                chevron_active: Style::default(),
                chevron_dim: Style::default(),
                count: Style::default(),
                icon: Style::default(),
            })
            .highlight_full_row(false);
        let mut state = TreeState::new(1);
        state.expand(0);
        state.select(99, None);
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 20, 3));
        let expected = Buffer::with_lines(vec![
            "v Fruits           2",
            "    A Apple         ",
            "    Banana          ",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn render_scrolling() {
        let tree = Tree::default()
            .groups(vec![
                TreeGroup::new(TreeItem::new("A")),
                TreeGroup::new(TreeItem::new("B")),
                TreeGroup::new(TreeItem::new("C")),
                TreeGroup::new(TreeItem::new("D")),
            ])
            .mode(Mode::None)
            .styles(TreeStyles {
                parent: Style::default(),
                child: Style::default(),
                selected: Style::default(),
                chevron: Style::default(),
                chevron_active: Style::default(),
                chevron_dim: Style::default(),
                count: Style::default(),
                icon: Style::default(),
            })
            .highlight_full_row(false);
        let mut state = TreeState::new(4);
        state.select(2, None); // select C
        // Viewport height 2 — ensure_visible scrolls so C is visible.
        // selected flat index=2, offset adjusts to 1 (2 - 2 + 1), showing B and C.
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 10, 2));
        let expected = Buffer::with_lines(vec!["> B       ", "> C       "]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn render_child_with_explicit_count() {
        let tree = Tree::default()
            .groups(vec![TreeGroup::new(TreeItem::new("Group")).children(vec![
                TreeItem::new("Item A").count(5),
                TreeItem::new("Item B"),
            ])])
            .styles(TreeStyles {
                parent: Style::default(),
                child: Style::default(),
                selected: Style::default(),
                chevron: Style::default(),
                chevron_active: Style::default(),
                chevron_dim: Style::default(),
                count: Style::default(),
                icon: Style::default(),
            })
            .highlight_full_row(false);
        let mut state = TreeState::new(1);
        state.expand(0);
        state.select(99, None);
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 20, 3));
        let expected = Buffer::with_lines(vec![
            "v Group            2",
            "    Item A         5",
            "    Item B          ",
        ]);
        assert_eq!(buf, expected);
    }

    // ---- Truncation helper ----

    #[test]
    fn truncate_no_op_when_fits() {
        assert_eq!(truncate_with_ellipsis("Hello", 10, "…"), "Hello");
    }

    #[test]
    fn truncate_adds_ellipsis() {
        assert_eq!(truncate_with_ellipsis("Hello World", 6, "…"), "Hello…");
    }

    #[test]
    fn truncate_very_narrow() {
        assert_eq!(truncate_with_ellipsis("Hello", 1, "…"), "H");
    }

    #[test]
    fn truncate_zero_width() {
        assert_eq!(truncate_with_ellipsis("Hello", 0, "…"), "");
    }

    // ---- Builder / Styled ----

    #[test]
    fn builder_methods_chainable() {
        let tree = Tree::default()
            .groups(vec![])
            .styles(TreeStyles::dark())
            .chevron_collapsed("▶")
            .chevron_expanded("▼")
            .indent(2)
            .mode(Mode::None)
            .ellipsis("...")
            .highlight_full_row(false);
        assert_eq!(tree.chevron_collapsed, "▶");
        assert_eq!(tree.chevron_expanded, "▼");
        assert_eq!(tree.indent, 2);
        assert_eq!(tree.mode, Mode::None);
    }

    #[test]
    fn styled_trait() {
        let style = Style::default().fg(Color::Red);
        let tree = Tree::default().set_style(style);
        assert_eq!(tree.style(), style);
    }

    // ---- Selected styling ----

    #[test]
    fn render_selected_has_style() {
        let selected_style = Style::default().fg(Color::Green).bg(Color::Black);
        let tree = Tree::default()
            .groups(vec![TreeGroup::new(TreeItem::new("A"))])
            .mode(Mode::None)
            .styles(TreeStyles {
                parent: Style::default(),
                child: Style::default(),
                selected: selected_style,
                chevron: Style::default(),
                chevron_active: Style::default(),
                chevron_dim: Style::default(),
                count: Style::default(),
                icon: Style::default(),
            });
        let mut state = TreeState::new(1);
        let buf = render_tree(&tree, &mut state, Rect::new(0, 0, 10, 1));
        // The selected row should have the selected style applied
        assert_eq!(buf[(0, 0)].fg, Color::Green);
        assert_eq!(buf[(0, 0)].bg, Color::Black);
    }
}
