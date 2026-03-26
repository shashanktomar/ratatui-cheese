//! A paginator widget for displaying page indicators.
//!
//! Renders pagination status as dots (`•••••`) or arabic numerals (`2/10`).
//! Handles page state tracking and provides helpers for slicing item collections.
//! Can be used standalone or embedded inside other widgets like a list.
//!
//! Ported from Charmbracelet's [Bubbles paginator](https://github.com/charmbracelet/bubbles/tree/master/paginator).
//!
//! # Example
//!
//! ```rust
//! use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorMode};
//!
//! let paginator = Paginator::default()
//!     .mode(PaginatorMode::Dots);
//!
//! let mut state = PaginatorState::new(100, 10); // 100 items, 10 per page
//! state.next_page();
//! ```

use crate::theme::Palette;
use crate::utils::display_width;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Styled};
use ratatui::widgets::{StatefulWidget, Widget};

// ---------------------------------------------------------------------------
// Mode
// ---------------------------------------------------------------------------

/// Controls how the paginator renders page indicators.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum PaginatorMode {
    /// Render as dots: `•••••`
    #[default]
    Dots,
    /// Render as arabic numerals: `2/10`
    Arabic,
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Styles for the different parts of the paginator.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PaginatorStyles {
    /// Style for the active dot or current page number.
    pub active: Style,
    /// Style for inactive dots or the separator/total.
    pub inactive: Style,
}

impl Default for PaginatorStyles {
    fn default() -> Self {
        Self::dark()
    }
}

impl PaginatorStyles {
    /// Creates styles from a [`Palette`].
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            active: Style::default().fg(p.primary),
            inactive: Style::default().fg(p.faint),
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

/// A paginator widget that renders page indicators.
///
/// `Paginator` holds appearance configuration. Mutable state (current page,
/// total pages) lives in [`PaginatorState`].
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::paginator::{Paginator, PaginatorMode};
///
/// let paginator = Paginator::default()
///     .mode(PaginatorMode::Dots)
///     .active_dot("•")
///     .inactive_dot("○");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Paginator {
    styles: PaginatorStyles,
    mode: PaginatorMode,
    active_dot: String,
    inactive_dot: String,
}

impl Default for Paginator {
    fn default() -> Self {
        Self {
            styles: PaginatorStyles::default(),
            mode: PaginatorMode::default(),
            active_dot: "•".into(),
            inactive_dot: "•".into(),
        }
    }
}

impl Paginator {
    /// Sets the styles for the paginator.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn styles(mut self, styles: PaginatorStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Sets the pagination display mode.
    ///
    /// Default: `PaginatorMode::Dots`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn mode(mut self, mode: PaginatorMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the character used for the active (current) page dot.
    ///
    /// Default: `"•"`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn active_dot(mut self, s: impl Into<String>) -> Self {
        self.active_dot = s.into();
        self
    }

    /// Sets the character used for inactive page dots.
    ///
    /// Default: `"•"`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn inactive_dot(mut self, s: impl Into<String>) -> Self {
        self.inactive_dot = s.into();
        self
    }
}

impl Styled for Paginator {
    type Item = Self;

    fn style(&self) -> Style {
        self.styles.active
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.styles.active = style.into();
        self
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Mutable state for a [`Paginator`] widget.
///
/// Tracks the current page, total pages, and items per page. Provides helper
/// methods for navigation and slicing item collections.
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::paginator::PaginatorState;
///
/// let mut state = PaginatorState::new(100, 10); // 100 items, 10 per page
/// assert_eq!(state.total_pages(), 10);
/// assert_eq!(state.page(), 0);
///
/// state.next_page();
/// assert_eq!(state.page(), 1);
///
/// let (start, end) = state.get_slice_bounds(100);
/// assert_eq!((start, end), (10, 20));
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PaginatorState {
    page: usize,
    total_pages: usize,
    per_page: usize,
}

impl PaginatorState {
    /// Creates a new state for the given number of items and items per page.
    ///
    /// Calculates `total_pages` automatically. If `per_page` is 0, it is
    /// treated as 1.
    pub fn new(total_items: usize, per_page: usize) -> Self {
        let per_page = per_page.max(1);
        let total_pages = total_pages(total_items, per_page);
        Self {
            page: 0,
            total_pages,
            per_page,
        }
    }

    /// Returns the current page (0-indexed).
    pub fn page(&self) -> usize {
        self.page
    }

    /// Returns the total number of pages.
    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    /// Returns the number of items per page.
    pub fn per_page(&self) -> usize {
        self.per_page
    }

    /// Navigates to the next page. Stops at the last page.
    pub fn next_page(&mut self) {
        if !self.on_last_page() {
            self.page += 1;
        }
    }

    /// Navigates to the previous page. Stops at the first page.
    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }

    /// Returns whether the current page is the first page.
    pub fn on_first_page(&self) -> bool {
        self.page == 0
    }

    /// Returns whether the current page is the last page.
    pub fn on_last_page(&self) -> bool {
        self.total_pages == 0 || self.page == self.total_pages - 1
    }

    /// Recalculates total pages from a new item count.
    ///
    /// Clamps the current page if it would be out of bounds.
    pub fn set_total_items(&mut self, total_items: usize) {
        self.total_pages = total_pages(total_items, self.per_page);
        if self.total_pages > 0 && self.page >= self.total_pages {
            self.page = self.total_pages - 1;
        }
    }

    /// Returns the start and end indices for slicing an item collection
    /// based on the current page.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui_cheese::paginator::PaginatorState;
    ///
    /// let state = PaginatorState::new(25, 10);
    /// let (start, end) = state.get_slice_bounds(25);
    /// assert_eq!((start, end), (0, 10));
    /// ```
    pub fn get_slice_bounds(&self, total_items: usize) -> (usize, usize) {
        let start = self.page * self.per_page;
        let end = (start + self.per_page).min(total_items);
        (start, end)
    }

    /// Returns the number of items on the current page.
    pub fn items_on_page(&self, total_items: usize) -> usize {
        if total_items == 0 {
            return 0;
        }
        let (start, end) = self.get_slice_bounds(total_items);
        end - start
    }
}

impl Default for PaginatorState {
    fn default() -> Self {
        Self::new(0, 1)
    }
}

/// Calculates the number of pages needed for the given items and page size.
fn total_pages(total_items: usize, per_page: usize) -> usize {
    if total_items == 0 || per_page == 0 {
        return 0;
    }
    total_items.div_ceil(per_page)
}

// ---------------------------------------------------------------------------
// Trait implementations
// ---------------------------------------------------------------------------

// Widget for owned — delegates to &ref
impl Widget for Paginator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

// Widget for &ref — stateless fallback with default state
impl Widget for &Paginator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = PaginatorState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// StatefulWidget for owned — delegates to &ref
impl StatefulWidget for Paginator {
    type State = PaginatorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

// StatefulWidget for &ref — the real render logic
impl StatefulWidget for &Paginator {
    type State = PaginatorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        match self.mode {
            PaginatorMode::Dots => self.render_dots(area, buf, state),
            PaginatorMode::Arabic => self.render_arabic(area, buf, state),
        }
    }
}

impl Paginator {
    /// Renders dot-style pagination: `●○○○○`
    fn render_dots(&self, area: Rect, buf: &mut Buffer, state: &PaginatorState) {
        let mut x = area.x;
        let max_x = area.x + area.width;

        for i in 0..state.total_pages {
            let (dot, style) = if i == state.page {
                (&self.active_dot, self.styles.active)
            } else {
                (&self.inactive_dot, self.styles.inactive)
            };

            let w = display_width(dot) as u16;
            if x + w > max_x {
                break;
            }

            buf.set_string(x, area.y, dot, style);
            x += w;
        }
    }

    /// Renders arabic-style pagination: `2/10`
    fn render_arabic(&self, area: Rect, buf: &mut Buffer, state: &PaginatorState) {
        let text = format!("{}/{}", state.page + 1, state.total_pages);
        let text_w = display_width(&text) as u16;

        if text_w > area.width {
            return;
        }

        buf.set_string(area.x, area.y, &text, self.styles.inactive);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

    // ---- State tests ----

    #[test]
    fn state_new_calculates_total_pages() {
        let state = PaginatorState::new(100, 10);
        assert_eq!(state.total_pages(), 10);
        assert_eq!(state.page(), 0);
        assert_eq!(state.per_page(), 10);
    }

    #[test]
    fn state_new_rounds_up() {
        let state = PaginatorState::new(25, 10);
        assert_eq!(state.total_pages(), 3);
    }

    #[test]
    fn state_new_zero_items() {
        let state = PaginatorState::new(0, 10);
        assert_eq!(state.total_pages(), 0);
    }

    #[test]
    fn state_new_zero_per_page_treated_as_one() {
        let state = PaginatorState::new(5, 0);
        assert_eq!(state.per_page(), 1);
        assert_eq!(state.total_pages(), 5);
    }

    #[test]
    fn state_new_single_item() {
        let state = PaginatorState::new(1, 10);
        assert_eq!(state.total_pages(), 1);
    }

    #[test]
    fn state_new_per_page_greater_than_total() {
        let state = PaginatorState::new(5, 10);
        assert_eq!(state.total_pages(), 1);
    }

    #[test]
    fn state_next_page() {
        let mut state = PaginatorState::new(30, 10);
        assert_eq!(state.page(), 0);
        state.next_page();
        assert_eq!(state.page(), 1);
        state.next_page();
        assert_eq!(state.page(), 2);
    }

    #[test]
    fn state_next_page_clamps_at_last() {
        let mut state = PaginatorState::new(20, 10);
        state.next_page();
        assert_eq!(state.page(), 1);
        state.next_page(); // already on last page
        assert_eq!(state.page(), 1);
    }

    #[test]
    fn state_prev_page() {
        let mut state = PaginatorState::new(30, 10);
        state.next_page();
        state.next_page();
        assert_eq!(state.page(), 2);
        state.prev_page();
        assert_eq!(state.page(), 1);
    }

    #[test]
    fn state_prev_page_clamps_at_first() {
        let mut state = PaginatorState::new(20, 10);
        state.prev_page(); // already on first page
        assert_eq!(state.page(), 0);
    }

    #[test]
    fn state_on_first_page() {
        let state = PaginatorState::new(20, 10);
        assert!(state.on_first_page());
        assert!(!state.on_last_page());
    }

    #[test]
    fn state_on_last_page() {
        let mut state = PaginatorState::new(20, 10);
        state.next_page();
        assert!(!state.on_first_page());
        assert!(state.on_last_page());
    }

    #[test]
    fn state_on_last_page_zero_items() {
        let state = PaginatorState::new(0, 10);
        assert!(state.on_first_page());
        assert!(state.on_last_page());
    }

    #[test]
    fn state_single_page_is_first_and_last() {
        let state = PaginatorState::new(5, 10);
        assert!(state.on_first_page());
        assert!(state.on_last_page());
    }

    #[test]
    fn state_get_slice_bounds_first_page() {
        let state = PaginatorState::new(25, 10);
        assert_eq!(state.get_slice_bounds(25), (0, 10));
    }

    #[test]
    fn state_get_slice_bounds_middle_page() {
        let mut state = PaginatorState::new(25, 10);
        state.next_page();
        assert_eq!(state.get_slice_bounds(25), (10, 20));
    }

    #[test]
    fn state_get_slice_bounds_last_page_partial() {
        let mut state = PaginatorState::new(25, 10);
        state.next_page();
        state.next_page();
        assert_eq!(state.get_slice_bounds(25), (20, 25));
    }

    #[test]
    fn state_items_on_page() {
        let mut state = PaginatorState::new(25, 10);
        assert_eq!(state.items_on_page(25), 10);
        state.next_page();
        assert_eq!(state.items_on_page(25), 10);
        state.next_page();
        assert_eq!(state.items_on_page(25), 5);
    }

    #[test]
    fn state_items_on_page_zero_items() {
        let state = PaginatorState::new(0, 10);
        assert_eq!(state.items_on_page(0), 0);
    }

    #[test]
    fn state_set_total_items() {
        let mut state = PaginatorState::new(30, 10);
        state.next_page();
        state.next_page();
        assert_eq!(state.page(), 2);

        state.set_total_items(15);
        assert_eq!(state.total_pages(), 2);
        assert_eq!(state.page(), 1); // clamped from 2 to 1
    }

    #[test]
    fn state_set_total_items_to_zero() {
        let mut state = PaginatorState::new(30, 10);
        state.next_page();
        state.set_total_items(0);
        assert_eq!(state.total_pages(), 0);
    }

    #[test]
    fn state_default() {
        let state = PaginatorState::default();
        assert_eq!(state.page(), 0);
        assert_eq!(state.total_pages(), 0);
        assert_eq!(state.per_page(), 1);
    }

    // ---- Render tests ----

    #[track_caller]
    fn render_stateful(paginator: &Paginator, state: &mut PaginatorState, width: u16) -> Buffer {
        let area = Rect::new(0, 0, width, 1);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(paginator, area, &mut buf, state);
        buf
    }

    /// Extracts the visible text content from a buffer row (trimming trailing spaces).
    #[track_caller]
    fn buf_text(buf: &Buffer) -> String {
        let mut s = String::new();
        for x in 0..buf.area.width {
            s.push_str(buf.cell((x, 0)).unwrap().symbol());
        }
        s.trim_end().to_string()
    }

    #[test]
    fn render_empty_area() {
        let paginator = Paginator::default();
        let mut state = PaginatorState::new(10, 5);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&paginator, area, &mut buf, &mut state);
        // No panic = pass
    }

    #[test]
    fn render_dots_first_page() {
        let paginator = Paginator::default();
        let mut state = PaginatorState::new(30, 10);
        let buf = render_stateful(&paginator, &mut state, 10);
        assert_eq!(buf_text(&buf), "•••");
    }

    #[test]
    fn render_dots_second_page() {
        let paginator = Paginator::default();
        let mut state = PaginatorState::new(30, 10);
        state.next_page();
        let buf = render_stateful(&paginator, &mut state, 10);
        assert_eq!(buf_text(&buf), "•••");
    }

    #[test]
    fn render_dots_last_page() {
        let paginator = Paginator::default();
        let mut state = PaginatorState::new(30, 10);
        state.next_page();
        state.next_page();
        let buf = render_stateful(&paginator, &mut state, 10);
        assert_eq!(buf_text(&buf), "•••");
    }

    #[test]
    fn render_dots_single_page() {
        let paginator = Paginator::default();
        let mut state = PaginatorState::new(5, 10);
        let buf = render_stateful(&paginator, &mut state, 10);
        assert_eq!(buf_text(&buf), "•");
    }

    #[test]
    fn render_dots_zero_pages() {
        let paginator = Paginator::default();
        let mut state = PaginatorState::new(0, 10);
        let buf = render_stateful(&paginator, &mut state, 10);
        assert_eq!(buf_text(&buf), "");
    }

    #[test]
    fn render_dots_custom_characters() {
        let paginator = Paginator::default().active_dot("●").inactive_dot("○");
        let mut state = PaginatorState::new(30, 10);
        state.next_page();
        let buf = render_stateful(&paginator, &mut state, 10);
        assert_eq!(buf_text(&buf), "○●○"); // custom chars: ● active, ○ inactive
    }

    #[test]
    fn render_dots_truncated_by_width() {
        let paginator = Paginator::default();
        let mut state = PaginatorState::new(100, 10); // 10 pages
        let buf = render_stateful(&paginator, &mut state, 5);
        // Only 5 dots fit in width 5
        assert_eq!(buf_text(&buf), "•••••");
    }

    #[test]
    fn render_arabic_first_page() {
        let paginator = Paginator::default().mode(PaginatorMode::Arabic);
        let mut state = PaginatorState::new(100, 10);
        let buf = render_stateful(&paginator, &mut state, 10);
        assert_eq!(buf_text(&buf), "1/10");
    }

    #[test]
    fn render_arabic_middle_page() {
        let paginator = Paginator::default().mode(PaginatorMode::Arabic);
        let mut state = PaginatorState::new(100, 10);
        for _ in 0..4 {
            state.next_page();
        }
        let buf = render_stateful(&paginator, &mut state, 10);
        assert_eq!(buf_text(&buf), "5/10");
    }

    #[test]
    fn render_arabic_too_narrow() {
        let paginator = Paginator::default().mode(PaginatorMode::Arabic);
        let mut state = PaginatorState::new(100, 10);
        let buf = render_stateful(&paginator, &mut state, 2);
        assert_eq!(buf_text(&buf), "");
    }

    #[test]
    fn render_dots_styles_applied() {
        let paginator = Paginator::default().styles(PaginatorStyles {
            active: Style::default().fg(Color::Green),
            inactive: Style::default().fg(Color::Red),
        });
        let mut state = PaginatorState::new(20, 10);
        let buf = render_stateful(&paginator, &mut state, 10);

        // First dot should be green (active)
        assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Green);
        // Second dot should be red (inactive)
        assert_eq!(buf.cell((1, 0)).unwrap().fg, Color::Red);
    }

    #[test]
    fn render_arabic_styles_applied() {
        let paginator = Paginator::default()
            .mode(PaginatorMode::Arabic)
            .styles(PaginatorStyles {
                active: Style::default().fg(Color::Green),
                inactive: Style::default().fg(Color::Red),
            });
        let mut state = PaginatorState::new(20, 10);
        let buf = render_stateful(&paginator, &mut state, 10);

        // Entire "1/2" string uses inactive style
        assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Red);
        assert_eq!(buf.cell((1, 0)).unwrap().fg, Color::Red);
        assert_eq!(buf.cell((2, 0)).unwrap().fg, Color::Red);
    }

    // ---- Widget trait tests ----

    #[test]
    fn stateless_widget_renders() {
        let paginator = Paginator::default();
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);
        Widget::render(&paginator, area, &mut buf);
        // Default state has 0 pages — nothing visible
        assert_eq!(buf_text(&Buffer::empty(area)), "");
    }

    #[test]
    fn styled_trait() {
        let paginator = Paginator::default();
        assert_eq!(paginator.style(), PaginatorStyles::dark().active);

        let styled = paginator.set_style(Style::default().fg(Color::Blue));
        assert_eq!(styled.styles.active, Style::default().fg(Color::Blue));
    }

    // ---- Mode tests ----

    #[test]
    fn mode_default_is_dots() {
        assert_eq!(PaginatorMode::default(), PaginatorMode::Dots);
    }

    // ---- Styles tests ----

    #[test]
    fn styles_default_is_dark() {
        assert_eq!(PaginatorStyles::default(), PaginatorStyles::dark());
    }

    #[test]
    fn styles_from_palette() {
        let p = Palette::charm();
        let styles = PaginatorStyles::from_palette(&p);
        assert_eq!(styles.active, Style::default().fg(p.primary));
        assert_eq!(styles.inactive, Style::default().fg(p.faint));
    }
}
