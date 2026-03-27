//! A paginated list widget with item delegation.
//!
//! Renders a vertically scrollable, paginated list of arbitrary items. Each item
//! implements the [`ListItem`] trait to control its own height and rendering.
//! Uses the [`Paginator`](crate::paginator::Paginator) widget internally for
//! page indicators.
//!
//! Ported from Charmbracelet's [Bubbles list](https://github.com/charmbracelet/bubbles/tree/master/list).
//!
//! Pagination assumes all items have uniform height. If items return
//! different values from [`ListItem::height()`], page boundaries may be
//! unpredictable.
//!
//! # Example
//!
//! ```rust
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::Rect;
//! use ratatui::prelude::*;
//! use ratatui_cheese::list::{List, ListItem, ListItemContext, ListState};
//!
//! struct MyItem { title: String }
//!
//! impl ListItem for MyItem {
//!     fn height(&self) -> u16 { 1 }
//!     fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
//!         let p = &ctx.palette;
//!         let style = if ctx.selected {
//!             Style::default().fg(p.primary)
//!         } else {
//!             Style::default().fg(p.foreground)
//!         };
//!         buf.set_string(area.x, area.y, &self.title, style);
//!     }
//! }
//!
//! let items = vec![MyItem { title: "Apple".into() }, MyItem { title: "Banana".into() }];
//! let list = List::new(&items);
//! let mut state = ListState::new(items.len());
//! ```

use crate::paginator::{Paginator, PaginatorState};
use crate::theme::Palette;
use crate::utils::{display_width, truncate_with_ellipsis};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Styled};
use ratatui::widgets::{StatefulWidget, Widget};

// ---------------------------------------------------------------------------
// ListItem trait
// ---------------------------------------------------------------------------

/// Context passed to each item during rendering.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ListItemContext {
    /// The item's index in the full list.
    pub index: usize,
    /// Whether the item is currently selected.
    pub selected: bool,
    /// The current page number (0-indexed).
    pub page: usize,
    /// The palette from the parent list, so items can respect theming.
    pub palette: Palette,
}

/// A trait for items that can be rendered inside a [`List`].
///
/// Implement this to control how each item appears and how tall it is.
///
/// **Note:** All items in a list should return the same value from
/// [`height()`](Self::height). Pagination assumes uniform item heights;
/// if items have different heights, page boundaries may be unpredictable.
pub trait ListItem {
    /// Height in rows this item occupies.
    ///
    /// All items in a list should return the same height. Mixed heights
    /// may cause pagination to over- or under-fill pages.
    fn height(&self) -> u16;

    /// Render this item into the given area.
    fn render(&self, area: Rect, buf: &mut Buffer, context: &ListItemContext);
}

// ---------------------------------------------------------------------------
// ListHeader trait
// ---------------------------------------------------------------------------

/// Context passed to the header during rendering.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ListHeaderContext {
    /// Total number of items in the list.
    pub total_items: usize,
    /// Current page (0-indexed).
    pub page: usize,
    /// Total number of pages.
    pub total_pages: usize,
    /// The palette from the parent list.
    pub palette: Palette,
}

/// A trait for custom headers rendered above list items.
///
/// Implement this to control the header area of a [`List`]. The header
/// receives context about the list state (total items, pagination) and
/// the palette for consistent theming.
pub trait ListHeader {
    /// Total height in rows this header occupies.
    fn height(&self) -> u16;

    /// Render the header into the given area.
    fn render(&self, area: Rect, buf: &mut Buffer, context: &ListHeaderContext);
}

// ---------------------------------------------------------------------------
// DefaultHeader
// ---------------------------------------------------------------------------

/// A simple header that renders a title and optional item count.
///
/// This covers the common case. For full control, implement [`ListHeader`]
/// directly.
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::list::DefaultHeader;
///
/// let header = DefaultHeader::new("My List").show_count(true);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DefaultHeader {
    title: String,
    show_count: bool,
}

impl DefaultHeader {
    /// Creates a new header with the given title.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            show_count: false,
        }
    }

    /// Sets whether to show the item count below the title.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn show_count(mut self, show: bool) -> Self {
        self.show_count = show;
        self
    }
}

impl ListHeader for DefaultHeader {
    fn height(&self) -> u16 {
        let mut h: u16 = 2; // title + blank line
        if self.show_count {
            h += 2; // count + blank line
        }
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListHeaderContext) {
        let p = &ctx.palette;
        let indent: u16 = 2;
        let x = area.x + indent;
        let max_w = area.width.saturating_sub(indent) as usize;

        let truncated = truncate_with_ellipsis(&self.title, max_w, "…");
        buf.set_string(x, area.y, &truncated, Style::default().fg(p.foreground));

        if self.show_count && area.height > 2 {
            let count_text = format!("{} items", ctx.total_items);
            let truncated = truncate_with_ellipsis(&count_text, max_w, "…");
            buf.set_string(x, area.y + 2, &truncated, Style::default().fg(p.muted));
        }
    }
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Style for the selection indicator.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ListStyles {
    /// Style for the selection indicator.
    pub selected: Style,
}

impl Default for ListStyles {
    fn default() -> Self {
        Self::dark()
    }
}

impl ListStyles {
    /// Creates styles from a [`Palette`].
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            selected: Style::default().fg(p.primary),
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

/// A paginated list widget that renders items implementing [`ListItem`].
///
/// `List` holds appearance configuration and a borrowed slice of items.
/// Mutable state (selection, pagination) lives in [`ListState`].
///
/// # Example
///
/// ```rust
/// use ratatui::buffer::Buffer;
/// use ratatui::layout::Rect;
/// use ratatui::prelude::*;
/// use ratatui_cheese::list::{List, ListItem, ListItemContext, ListState};
///
/// struct Item(String);
/// impl ListItem for Item {
///     fn height(&self) -> u16 { 1 }
///     fn render(&self, area: Rect, buf: &mut Buffer, _ctx: &ListItemContext) {
///         buf.set_string(area.x, area.y, &self.0, Style::default());
///     }
/// }
///
/// let items = vec![Item("one".into()), Item("two".into())];
/// let list = List::new(&items);
/// let mut state = ListState::new(items.len());
/// ```
pub struct List<'a, T: ListItem> {
    items: &'a [T],
    header: Option<&'a dyn ListHeader>,
    styles: ListStyles,
    palette: Palette,
    show_paginator: bool,
    paginator: Paginator,
    styles_overridden: bool,
    selection_indicator: String,
    infinite_scrolling: bool,
    item_spacing: u16,
}

impl<'a, T: ListItem> List<'a, T> {
    /// Creates a new list with the given items.
    #[must_use]
    pub fn new(items: &'a [T]) -> Self {
        Self {
            items,
            header: None,
            styles: ListStyles::default(),
            palette: Palette::default(),
            show_paginator: true,
            paginator: Paginator::default(),
            styles_overridden: false,
            selection_indicator: "│".into(),
            infinite_scrolling: false,
            item_spacing: 1,
        }
    }

    /// Sets the items to display.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn items(mut self, items: &'a [T]) -> Self {
        self.items = items;
        self
    }

    /// Sets a custom header that controls its own rendering.
    ///
    /// When set, the custom header replaces the built-in title and count.
    /// The header receives a [`ListHeaderContext`] with item count,
    /// pagination info, and the palette.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn header(mut self, header: &'a dyn ListHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// Sets the styles for the list.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn styles(mut self, styles: ListStyles) -> Self {
        self.styles = styles;
        self.styles_overridden = true;
        self
    }

    /// Sets the palette, which is passed to items and headers via
    /// [`ListItemContext`] and [`ListHeaderContext`] for consistent theming.
    ///
    /// Also sets the list's styles from this palette via
    /// [`ListStyles::from_palette`], unless styles were already set
    /// explicitly with [`styles()`](Self::styles).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn palette(mut self, palette: Palette) -> Self {
        if !self.styles_overridden {
            self.styles = ListStyles::from_palette(&palette);
        }
        self.palette = palette;
        self
    }

    /// Sets whether to show the paginator at the bottom.
    ///
    /// Default: `true`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn show_paginator(mut self, show: bool) -> Self {
        self.show_paginator = show;
        self
    }

    /// Sets the paginator configuration for rendering page indicators.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn paginator(mut self, paginator: Paginator) -> Self {
        self.paginator = paginator;
        self
    }

    /// Sets the selection indicator string displayed on the left edge of the
    /// selected item.
    ///
    /// Default: `"│"`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn selection_indicator(mut self, s: impl Into<String>) -> Self {
        self.selection_indicator = s.into();
        self
    }

    /// Sets whether scrolling wraps around at boundaries.
    ///
    /// Default: `false`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn infinite_scrolling(mut self, infinite: bool) -> Self {
        self.infinite_scrolling = infinite;
        self
    }

    /// Sets the number of blank rows between items.
    ///
    /// Default: `1` (matching Bubbles' default spacing)
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn item_spacing(mut self, spacing: u16) -> Self {
        self.item_spacing = spacing;
        self
    }
}

impl<T: ListItem> Styled for List<'_, T> {
    type Item = Self;

    fn style(&self) -> Style {
        self.styles.selected
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.styles.selected = style.into();
        self
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Mutable state for a [`List`] widget.
///
/// Tracks the selected item index and pagination state. Provides navigation
/// methods that automatically handle page boundaries.
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::list::ListState;
///
/// let mut state = ListState::new(100);
/// assert_eq!(state.selected(), 0);
///
/// state.select_next(100, false);
/// assert_eq!(state.selected(), 1);
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct ListState {
    selected: usize,
    paginator: PaginatorState,
}

impl ListState {
    /// Creates a new state for a list with the given number of items.
    ///
    /// The items per page is initially set to 1 and will be recalculated
    /// during rendering based on the available area and item heights.
    pub fn new(total_items: usize) -> Self {
        Self {
            selected: 0,
            paginator: PaginatorState::new(total_items, 1),
        }
    }

    /// Returns the currently selected item index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Sets the selected item index.
    ///
    /// Clamps to `item_count - 1` if out of bounds.
    pub fn select(&mut self, index: usize, item_count: usize) {
        if item_count == 0 {
            self.selected = 0;
        } else {
            self.selected = index.min(item_count - 1);
        }
    }

    /// Selects the next item. If `infinite` is true, wraps to the first item
    /// when at the end.
    pub fn select_next(&mut self, item_count: usize, infinite: bool) {
        if item_count == 0 {
            return;
        }
        if self.selected + 1 < item_count {
            self.selected += 1;
        } else if infinite {
            self.selected = 0;
        }
        self.ensure_selected_visible(item_count);
    }

    /// Selects the previous item. If `infinite` is true, wraps to the last
    /// item when at the beginning.
    pub fn select_prev(&mut self, item_count: usize, infinite: bool) {
        if item_count == 0 {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        } else if infinite {
            self.selected = item_count - 1;
        }
        self.ensure_selected_visible(item_count);
    }

    /// Navigates to the next page, moving selection to the first item on the
    /// new page.
    pub fn next_page(&mut self, item_count: usize) {
        if item_count == 0 {
            return;
        }
        self.paginator.next_page();
        let (start, _) = self.paginator.get_slice_bounds(item_count);
        self.selected = start;
    }

    /// Navigates to the previous page, moving selection to the first item on
    /// the new page.
    pub fn prev_page(&mut self, item_count: usize) {
        if item_count == 0 {
            return;
        }
        self.paginator.prev_page();
        let (start, _) = self.paginator.get_slice_bounds(item_count);
        self.selected = start;
    }

    /// Selects the first item on the current page.
    pub fn select_first_on_page(&mut self) {
        let start = self.paginator.page() * self.paginator.per_page();
        self.selected = start;
    }

    /// Selects the last item on the current page.
    pub fn select_last_on_page(&mut self, item_count: usize) {
        if item_count == 0 {
            return;
        }
        let per_page = self.paginator.per_page();
        let last = ((self.paginator.page() + 1) * per_page)
            .min(item_count)
            .saturating_sub(1);
        self.selected = last;
    }

    /// Returns the current page (0-indexed).
    pub fn page(&self) -> usize {
        self.paginator.page()
    }

    /// Returns the total number of pages.
    pub fn total_pages(&self) -> usize {
        self.paginator.total_pages()
    }

    /// Returns a reference to the internal paginator state.
    pub fn paginator(&self) -> &PaginatorState {
        &self.paginator
    }

    /// Ensures the selected item is on the current page by adjusting the page
    /// if needed.
    fn ensure_selected_visible(&mut self, item_count: usize) {
        if item_count == 0 || self.paginator.per_page() == 0 {
            return;
        }
        let target_page = self.selected / self.paginator.per_page();
        while self.paginator.page() < target_page && !self.paginator.on_last_page() {
            self.paginator.next_page();
        }
        while self.paginator.page() > target_page && !self.paginator.on_first_page() {
            self.paginator.prev_page();
        }
    }
}

// ---------------------------------------------------------------------------
// Trait implementations
// ---------------------------------------------------------------------------

// Widget for owned — delegates to &ref
impl<T: ListItem> Widget for List<'_, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

// Widget for &ref — stateless fallback with default state
impl<T: ListItem> Widget for &List<'_, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = ListState::new(self.items.len());
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// StatefulWidget for owned — delegates to &ref
impl<T: ListItem> StatefulWidget for List<'_, T> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

// StatefulWidget for &ref — the real render logic
impl<T: ListItem> StatefulWidget for &List<'_, T> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        // 1. Calculate chrome height (excluding paginator — added only if needed)
        let chrome_height: u16 = self.header.map_or(0, |h| h.height());

        let paginator_chrome: u16 = 2; // blank line + paginator line

        // 2. Calculate per_page — first try without paginator
        let items_area_height = area.height.saturating_sub(chrome_height);
        if items_area_height == 0 {
            return;
        }

        let mut per_page = calculate_per_page(self.items, items_area_height, self.item_spacing);
        if per_page == 0 {
            return;
        }

        // If all items fit, no paginator needed
        let needs_paginator =
            self.show_paginator && !self.items.is_empty() && per_page < self.items.len();

        // If paginator is needed, recalculate with reduced height
        if needs_paginator {
            let reduced = items_area_height.saturating_sub(paginator_chrome);
            per_page = calculate_per_page(self.items, reduced, self.item_spacing);
            if per_page == 0 {
                return;
            }
        }

        // 3. Update paginator state
        state.paginator = PaginatorState::new(self.items.len(), per_page);
        // Restore the page for the selected item
        state.ensure_selected_visible(self.items.len());

        // Clamp selected
        if !self.items.is_empty() && state.selected >= self.items.len() {
            state.selected = self.items.len() - 1;
        }

        let mut y = area.y;

        // 4. Render header
        if let Some(header) = self.header {
            let header_h = header.height();
            if header_h > 0 {
                let header_area = Rect::new(area.x, y, area.width, header_h);
                let header_ctx = ListHeaderContext {
                    total_items: self.items.len(),
                    page: state.paginator.page(),
                    total_pages: state.paginator.total_pages(),
                    palette: self.palette.clone(),
                };
                header.render(header_area, buf, &header_ctx);
                y += header_h;
            }
        }

        // 6. Render items
        let (start, end) = state.paginator.get_slice_bounds(self.items.len());
        let indicator_w = display_width(&self.selection_indicator) as u16;

        for (i, item) in self.items[start..end].iter().enumerate() {
            let abs_index = start + i;
            let is_selected = abs_index == state.selected;
            let item_h = item.height();

            if y + item_h > area.y + area.height {
                break;
            }

            // Render selection indicator
            if is_selected {
                buf.set_string(area.x, y, &self.selection_indicator, self.styles.selected);
            }

            // Item area: offset by indicator width + 1 space
            let item_x = area.x + indicator_w + 1;
            let item_w = area.width.saturating_sub(indicator_w + 1);
            let item_area = Rect::new(item_x, y, item_w, item_h);

            let context = ListItemContext {
                index: abs_index,
                selected: is_selected,
                page: state.paginator.page(),
                palette: self.palette.clone(),
            };

            item.render(item_area, buf, &context);
            y += item_h;

            // Add spacing between items (not after the last one)
            if i + 1 < end - start {
                y += self.item_spacing;
            }
        }

        // 7. Render paginator
        if needs_paginator && state.paginator.total_pages() > 1 {
            y += 1; // blank line before paginator
            if y < area.y + area.height {
                let pag_indent: u16 = 2;
                let pag_x = area.x + pag_indent;
                let pag_w = area.width.saturating_sub(pag_indent);
                let pag_area = Rect::new(pag_x, y, pag_w, 1);
                StatefulWidget::render(&self.paginator, pag_area, buf, &mut state.paginator);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Calculate how many items fit in the given height, accounting for item
/// height and spacing between items. Uses the tallest item's height to
/// ensure every page can fit its items. Returns 0 if no items exist.
fn calculate_per_page<T: ListItem>(items: &[T], available_height: u16, spacing: u16) -> usize {
    if items.is_empty() {
        return 0;
    }
    // Use the tallest item's height so every page fits its items
    let max_h = items.iter().map(|i| i.height().max(1)).max().unwrap_or(1);
    let slot = max_h + spacing; // height + gap per item
    if available_height < max_h {
        return 0;
    }
    // n items need: n * max_h + (n-1) * spacing = n * slot - spacing
    // n <= (available_height + spacing) / slot
    ((available_height + spacing) / slot) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

    // -- Test item implementation --

    #[derive(Debug, Clone)]
    struct TestItem {
        text: String,
        height: u16,
    }

    impl TestItem {
        fn new(text: &str) -> Self {
            Self {
                text: text.to_string(),
                height: 1,
            }
        }
    }

    impl ListItem for TestItem {
        fn height(&self) -> u16 {
            self.height
        }

        fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
            let style = if ctx.selected {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default()
            };
            buf.set_string(area.x, area.y, &self.text, style);
        }
    }

    // -- Helpers --

    #[track_caller]
    fn render_list(
        list: &List<'_, TestItem>,
        state: &mut ListState,
        width: u16,
        height: u16,
    ) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(list, area, &mut buf, state);
        buf
    }

    /// Extracts visible text from a buffer row, trimming trailing spaces.
    #[track_caller]
    fn row_text(buf: &Buffer, row: u16) -> String {
        let mut s = String::new();
        for x in 0..buf.area.width {
            s.push_str(buf.cell((x, row)).unwrap().symbol());
        }
        s.trim_end().to_string()
    }

    // -- State tests --

    #[test]
    fn state_new() {
        let state = ListState::new(10);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn state_select() {
        let mut state = ListState::new(10);
        state.select(5, 10);
        assert_eq!(state.selected(), 5);
    }

    #[test]
    fn state_select_clamps() {
        let mut state = ListState::new(3);
        state.select(10, 3);
        assert_eq!(state.selected(), 2); // clamped to last
    }

    #[test]
    fn state_select_zero_items() {
        let mut state = ListState::new(0);
        state.select(5, 0);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn state_select_next() {
        let mut state = ListState::new(5);
        state.select_next(5, false);
        assert_eq!(state.selected(), 1);
        state.select_next(5, false);
        assert_eq!(state.selected(), 2);
    }

    #[test]
    fn state_select_next_clamps() {
        let mut state = ListState::new(3);
        state.select(2, 3);
        state.select_next(3, false);
        assert_eq!(state.selected(), 2); // stays at end
    }

    #[test]
    fn state_select_next_wraps() {
        let mut state = ListState::new(3);
        state.select(2, 3);
        state.select_next(3, true);
        assert_eq!(state.selected(), 0); // wraps
    }

    #[test]
    fn state_select_prev() {
        let mut state = ListState::new(5);
        state.select(2, 3);
        state.select_prev(5, false);
        assert_eq!(state.selected(), 1);
    }

    #[test]
    fn state_select_prev_clamps() {
        let mut state = ListState::new(3);
        state.select_prev(3, false);
        assert_eq!(state.selected(), 0); // stays at start
    }

    #[test]
    fn state_select_prev_wraps() {
        let mut state = ListState::new(3);
        state.select_prev(3, true);
        assert_eq!(state.selected(), 2); // wraps to end
    }

    #[test]
    fn state_next_page() {
        let mut state = ListState::new(20);
        // Simulate per_page being set by render
        state.paginator = PaginatorState::new(20, 5);
        state.next_page(20);
        assert_eq!(state.page(), 1);
        assert_eq!(state.selected(), 5); // first item on page 2
    }

    #[test]
    fn state_prev_page() {
        let mut state = ListState::new(20);
        state.paginator = PaginatorState::new(20, 5);
        state.next_page(20);
        state.prev_page(20);
        assert_eq!(state.page(), 0);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn state_select_first_on_page() {
        let mut state = ListState::new(20);
        state.paginator = PaginatorState::new(20, 5);
        state.next_page(20);
        state.select(7, 20); // middle of page 2
        state.select_first_on_page();
        assert_eq!(state.selected(), 5); // first item on page 2
    }

    #[test]
    fn state_select_last_on_page() {
        let mut state = ListState::new(20);
        state.paginator = PaginatorState::new(20, 5);
        state.next_page(20);
        state.select_last_on_page(20);
        assert_eq!(state.selected(), 9); // last item on page 2
    }

    #[test]
    fn state_select_last_on_page_partial() {
        // 7 items, 3 per page → page 0: [0,1,2], page 1: [3,4,5], page 2: [6]
        let mut state = ListState::new(7);
        state.paginator = PaginatorState::new(7, 3);
        state.next_page(7);
        state.next_page(7);
        assert_eq!(state.page(), 2);
        state.select_last_on_page(7);
        assert_eq!(state.selected(), 6); // only item on last page
    }

    #[test]
    fn state_select_first_last_on_page_zero_items() {
        let mut state = ListState::new(0);
        state.select_first_on_page();
        assert_eq!(state.selected(), 0);
        state.select_last_on_page(0);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn state_zero_items() {
        let mut state = ListState::new(0);
        state.select_next(0, false);
        assert_eq!(state.selected(), 0);
        state.select_prev(0, true);
        assert_eq!(state.selected(), 0);
    }

    // -- Render tests --

    #[test]
    fn render_empty_area() {
        let items: Vec<TestItem> = vec![];
        let list = List::new(&items);
        let mut state = ListState::new(0);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&list, area, &mut buf, &mut state);
        // No panic = pass
    }

    #[test]
    fn render_empty_items() {
        let items: Vec<TestItem> = vec![];
        let list = List::new(&items);
        let mut state = ListState::new(0);
        let buf = render_list(&list, &mut state, 20, 5);
        // All spaces
        assert_eq!(row_text(&buf, 0), "");
    }

    #[test]
    fn render_basic_items() {
        let items = vec![
            TestItem::new("Apple"),
            TestItem::new("Banana"),
            TestItem::new("Cherry"),
        ];
        let list = List::new(&items);
        let mut state = ListState::new(items.len());
        // 3 items + 2 gaps (spacing=1) = 5 rows
        let buf = render_list(&list, &mut state, 20, 5);

        assert_eq!(row_text(&buf, 0), "│ Apple");
        assert_eq!(row_text(&buf, 1), ""); // spacing
        assert_eq!(row_text(&buf, 2), "  Banana");
        assert_eq!(row_text(&buf, 3), ""); // spacing
        assert_eq!(row_text(&buf, 4), "  Cherry");
    }

    #[test]
    fn render_no_spacing() {
        let items = vec![
            TestItem::new("Apple"),
            TestItem::new("Banana"),
            TestItem::new("Cherry"),
        ];
        let list = List::new(&items).item_spacing(0);
        let mut state = ListState::new(items.len());
        let buf = render_list(&list, &mut state, 20, 3);

        assert_eq!(row_text(&buf, 0), "│ Apple");
        assert_eq!(row_text(&buf, 1), "  Banana");
        assert_eq!(row_text(&buf, 2), "  Cherry");
    }

    #[test]
    fn render_selection_moves() {
        let items = vec![
            TestItem::new("Apple"),
            TestItem::new("Banana"),
            TestItem::new("Cherry"),
        ];
        let list = List::new(&items);
        let mut state = ListState::new(items.len());
        state.select(1, items.len());
        let buf = render_list(&list, &mut state, 20, 5);

        assert_eq!(row_text(&buf, 0), "  Apple");
        assert_eq!(row_text(&buf, 1), ""); // spacing
        assert_eq!(row_text(&buf, 2), "│ Banana");
        assert_eq!(row_text(&buf, 3), ""); // spacing
        assert_eq!(row_text(&buf, 4), "  Cherry");
    }

    #[test]
    fn render_with_default_header() {
        let items = vec![TestItem::new("Apple"), TestItem::new("Banana")];
        let header = DefaultHeader::new("Fruits");
        let list = List::new(&items).header(&header);
        let mut state = ListState::new(items.len());
        // header(2) + 2 items + 1 gap = 5
        let buf = render_list(&list, &mut state, 20, 5);

        assert_eq!(row_text(&buf, 0), "  Fruits");
        assert_eq!(row_text(&buf, 1), ""); // blank line after title
        assert_eq!(row_text(&buf, 2), "│ Apple");
        assert_eq!(row_text(&buf, 3), ""); // spacing
        assert_eq!(row_text(&buf, 4), "  Banana");
    }

    #[test]
    fn render_with_default_header_and_count() {
        let items = vec![TestItem::new("Apple"), TestItem::new("Banana")];
        let header = DefaultHeader::new("Fruits").show_count(true);
        let list = List::new(&items).header(&header);
        let mut state = ListState::new(items.len());
        // header(4) + 2 items + 1 gap = 7
        let buf = render_list(&list, &mut state, 20, 7);

        assert_eq!(row_text(&buf, 0), "  Fruits");
        assert_eq!(row_text(&buf, 1), ""); // blank line after title
        assert_eq!(row_text(&buf, 2), "  2 items");
        assert_eq!(row_text(&buf, 3), ""); // blank line after count
        assert_eq!(row_text(&buf, 4), "│ Apple");
        assert_eq!(row_text(&buf, 5), ""); // spacing
        assert_eq!(row_text(&buf, 6), "  Banana");
    }

    #[test]
    fn render_pagination() {
        let items = vec![
            TestItem::new("A"),
            TestItem::new("B"),
            TestItem::new("C"),
            TestItem::new("D"),
        ];
        let list = List::new(&items);
        // items_area = 5 - 2 (paginator chrome) = 3 rows
        // With spacing=1: slot=2, fits (3+1)/2 = 2 items per page
        // 2 items + 1 gap = 3, then blank + paginator = 5
        let mut state = ListState::new(items.len());
        let buf = render_list(&list, &mut state, 20, 5);

        assert_eq!(row_text(&buf, 0), "│ A");
        assert_eq!(row_text(&buf, 1), ""); // spacing
        assert_eq!(row_text(&buf, 2), "  B");
        // row 3: blank line before paginator
        assert_eq!(row_text(&buf, 3), "");
        // row 4: paginator dots
        assert_eq!(row_text(&buf, 4), "  ••");
    }

    #[test]
    fn render_mixed_height_items() {
        // 3 items: heights 1, 2, 1. Total content = 1+2+1 = 4 rows (no spacing).
        // max_h=2, slot=2. per_page = (4+0)/2 = 2. All 3 items > 2 per_page
        // → paginator needed. Reduced height = 4-2 = 2, per_page = 1.
        // So at height 4 we get pagination with 1 item per page.
        //
        // Use height 6 instead: per_page = (6+0)/2 = 3 ≥ 3 items → no paginator.
        // Items render: "Short"(1) + "Tall"(2) + "Short2"(1) = 4 rows, fits in 6.
        let mut items = vec![
            TestItem::new("Short"),
            TestItem::new("Tall"),
            TestItem::new("Short2"),
        ];
        items[1].height = 2;
        let list = List::new(&items).item_spacing(0);
        let mut state = ListState::new(items.len());
        let buf = render_list(&list, &mut state, 20, 6);

        assert_eq!(row_text(&buf, 0), "│ Short");
        assert_eq!(row_text(&buf, 1), "  Tall");
        // row 2 is the second line of "Tall" (height=2), empty since render only writes line 0
        assert_eq!(row_text(&buf, 3), "  Short2");
    }

    #[test]
    fn render_selected_item_style() {
        let items = vec![TestItem::new("Apple")];
        let list = List::new(&items);
        let mut state = ListState::new(items.len());
        let buf = render_list(&list, &mut state, 20, 3);

        // The selection indicator should use the selected style
        let indicator_cell = buf.cell((0, 0)).unwrap();
        assert_eq!(indicator_cell.fg, ListStyles::dark().selected.fg.unwrap());
    }

    #[test]
    fn render_custom_selection_indicator() {
        let items = vec![TestItem::new("Apple"), TestItem::new("Banana")];
        let list = List::new(&items).selection_indicator(">");
        let mut state = ListState::new(items.len());
        // 2 items + 1 gap = 3
        let buf = render_list(&list, &mut state, 20, 3);

        assert_eq!(row_text(&buf, 0), "> Apple");
        assert_eq!(row_text(&buf, 1), ""); // spacing
        assert_eq!(row_text(&buf, 2), "  Banana");
    }

    #[test]
    fn render_no_paginator() {
        let items = vec![TestItem::new("A"), TestItem::new("B")];
        let list = List::new(&items).show_paginator(false);
        let mut state = ListState::new(items.len());
        // 2 items + 1 gap = 3
        let buf = render_list(&list, &mut state, 20, 3);

        assert_eq!(row_text(&buf, 0), "│ A");
        assert_eq!(row_text(&buf, 1), ""); // spacing
        assert_eq!(row_text(&buf, 2), "  B");
    }

    // -- Custom header tests --

    struct TestHeader;

    impl ListHeader for TestHeader {
        fn height(&self) -> u16 {
            2
        }

        fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListHeaderContext) {
            buf.set_string(area.x, area.y, "My List", Style::default());
            let count = format!("{} items", ctx.total_items);
            buf.set_string(area.x, area.y + 1, &count, Style::default());
        }
    }

    #[test]
    fn render_custom_header() {
        let items = vec![TestItem::new("Apple"), TestItem::new("Banana")];
        let header = TestHeader;
        let list = List::new(&items).header(&header);
        let mut state = ListState::new(items.len());
        // header(2) + 2 items + 1 gap = 5
        let buf = render_list(&list, &mut state, 20, 5);

        assert_eq!(row_text(&buf, 0), "My List");
        assert_eq!(row_text(&buf, 1), "2 items");
        assert_eq!(row_text(&buf, 2), "│ Apple");
        assert_eq!(row_text(&buf, 3), ""); // spacing
        assert_eq!(row_text(&buf, 4), "  Banana");
    }

    #[test]
    fn render_custom_header_context() {
        let items = vec![
            TestItem::new("A"),
            TestItem::new("B"),
            TestItem::new("C"),
            TestItem::new("D"),
        ];

        struct ContextCheckHeader;
        impl ListHeader for ContextCheckHeader {
            fn height(&self) -> u16 {
                1
            }
            fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListHeaderContext) {
                let text = format!(
                    "total={} page={} pages={}",
                    ctx.total_items, ctx.page, ctx.total_pages
                );
                buf.set_string(area.x, area.y, &text, Style::default());
            }
        }

        let header = ContextCheckHeader;
        let list = List::new(&items).header(&header);
        let mut state = ListState::new(items.len());
        // header(1) + items_area(4) = 5, but with spacing items need more
        // 4 items need 4+3=7 rows with spacing, only 4 available → paginated
        let buf = render_list(&list, &mut state, 40, 5);

        // Should show pagination info in header
        let header_text = row_text(&buf, 0);
        assert!(header_text.starts_with("total=4 page=0"));
    }

    // -- Styles tests --

    #[test]
    fn styles_default_is_dark() {
        assert_eq!(ListStyles::default(), ListStyles::dark());
    }

    #[test]
    fn styles_from_palette() {
        let p = Palette::charm();
        let styles = ListStyles::from_palette(&p);
        assert_eq!(styles.selected, Style::default().fg(p.primary));
    }

    // -- Styled trait --

    #[test]
    fn styled_trait() {
        let items: Vec<TestItem> = vec![];
        let list = List::new(&items);
        assert_eq!(list.style(), ListStyles::dark().selected);
    }
}
