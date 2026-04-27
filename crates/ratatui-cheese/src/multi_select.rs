//! Multiple-selection widget.
//!
//! A form field for toggling multiple options on/off from a vertical list.
//! Inspired by Charmbracelet's [huh](https://github.com/charmbracelet/huh)
//! multi-select field.
//!
//! # Example
//!
//! ```rust
//! use ratatui_cheese::multi_select::{MultiSelect, MultiSelectOption, MultiSelectState};
//!
//! let options: Vec<MultiSelectOption> = vec!["Apple".into(), "Banana".into(), "Cherry".into()];
//! let multi = MultiSelect::new("Pick fruits", &options);
//!
//! let mut state = MultiSelectState::from_options(&options);
//! state.toggle_current(None);
//! assert!(state.is_selected(0));
//! ```

use crate::field::{ValidationKind, ValidationResult};
use crate::theme::Palette;
use crate::utils::display_width;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{StatefulWidget, Widget};

// ---------------------------------------------------------------------------
// Option
// ---------------------------------------------------------------------------

/// A single option in a [`MultiSelect`] widget.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MultiSelectOption<'a> {
    /// Display label for this option.
    pub label: &'a str,
    /// Whether this option can be toggled. Disabled options are
    /// rendered in a faint style and skipped by cursor navigation.
    pub enabled: bool,
}

impl<'a> MultiSelectOption<'a> {
    /// Creates a new enabled option with the given label.
    #[must_use]
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            enabled: true,
        }
    }

    /// Sets whether this option is enabled.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<'a> From<&'a str> for MultiSelectOption<'a> {
    fn from(label: &'a str) -> Self {
        Self::new(label)
    }
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Visual styles for the [`MultiSelect`] widget.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MultiSelectStyles {
    /// Title text.
    pub title: Style,
    /// Description text below the title.
    pub description: Style,
    /// Cursor indicator character.
    pub cursor: Style,
    /// Checked indicator and label.
    pub checked: Style,
    /// Unchecked indicator and label.
    pub unchecked: Style,
    /// Disabled option text.
    pub disabled: Style,
    /// Validation error message.
    pub validation_error: Style,
    /// Validation success message.
    pub validation_success: Style,
}

impl Default for MultiSelectStyles {
    fn default() -> Self {
        Self::dark()
    }
}

impl MultiSelectStyles {
    /// Creates styles from a [`Palette`].
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            title: Style::default()
                .fg(p.secondary)
                .add_modifier(Modifier::BOLD),
            description: Style::default().fg(p.muted),
            cursor: Style::default().fg(p.primary),
            checked: Style::default().fg(p.primary),
            unchecked: Style::default().fg(p.foreground),
            disabled: Style::default().fg(p.faint),
            validation_error: Style::default().fg(p.error),
            validation_success: Style::default().fg(p.success),
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
// Widget
// ---------------------------------------------------------------------------

/// A multiple-selection widget.
///
/// `MultiSelect` holds appearance configuration (title, options, styles).
/// Mutable state (cursor position, selections, validation) lives in
/// [`MultiSelectState`].
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::multi_select::{MultiSelect, MultiSelectOption};
///
/// let options: Vec<MultiSelectOption> = vec!["Red".into(), "Green".into(), "Blue".into()];
/// let multi = MultiSelect::new("Colors", &options)
///     .description("Pick your favorites.");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MultiSelect<'a> {
    title: &'a str,
    description: Option<&'a str>,
    options: &'a [MultiSelectOption<'a>],
    limit: Option<usize>,
    cursor_indicator: &'a str,
    checked_indicator: &'a str,
    unchecked_indicator: &'a str,
    styles: MultiSelectStyles,
}

impl<'a> MultiSelect<'a> {
    /// Creates a new multi-select with the given title and options.
    #[must_use]
    pub fn new(title: &'a str, options: &'a [MultiSelectOption<'a>]) -> Self {
        Self {
            title,
            description: None,
            options,
            limit: None,
            cursor_indicator: ">",
            checked_indicator: "✓",
            unchecked_indicator: "•",
            styles: MultiSelectStyles::default(),
        }
    }

    /// Sets the description shown below the title.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the maximum number of selections allowed.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the cursor indicator (default: ">").
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn cursor_indicator(mut self, indicator: &'a str) -> Self {
        self.cursor_indicator = indicator;
        self
    }

    /// Sets the checked indicator (default: "✓").
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn checked_indicator(mut self, indicator: &'a str) -> Self {
        self.checked_indicator = indicator;
        self
    }

    /// Sets the unchecked indicator (default: "•").
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn unchecked_indicator(mut self, indicator: &'a str) -> Self {
        self.unchecked_indicator = indicator;
        self
    }

    /// Sets the styles for this multi-select.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn styles(mut self, styles: MultiSelectStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Sets styles from a [`Palette`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn palette(mut self, palette: &Palette) -> Self {
        self.styles = MultiSelectStyles::from_palette(palette);
        self
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Mutable state for a [`MultiSelect`] widget.
///
/// Holds cursor position, selection state, focus state, and optional
/// validation. The validator (if set) runs automatically on blur.
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::multi_select::{MultiSelectOption, MultiSelectState};
///
/// let options: Vec<MultiSelectOption> = vec!["A".into(), "B".into(), "C".into()];
/// let mut state = MultiSelectState::from_options(&options);
/// state.toggle_current(None);
/// assert!(state.is_selected(0));
/// assert_eq!(state.selected_count(), 1);
///
/// state.next();
/// state.toggle_current(None);
/// assert_eq!(state.selected_indices(), vec![0, 1]);
/// ```
type Validator = Box<dyn Fn(&[bool]) -> ValidationResult>;

pub struct MultiSelectState {
    cursor: usize,
    selected: Vec<bool>,
    enabled: Vec<bool>,
    focused: bool,
    validation_message: Option<(ValidationKind, String)>,
    validator: Option<Validator>,
}

impl std::fmt::Debug for MultiSelectState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiSelectState")
            .field("cursor", &self.cursor)
            .field("selected", &self.selected)
            .field("enabled", &self.enabled)
            .field("focused", &self.focused)
            .field("validation_message", &self.validation_message)
            .field("validator", &self.validator.as_ref().map(|_| ".."))
            .finish()
    }
}

impl Default for MultiSelectState {
    fn default() -> Self {
        Self::new(0)
    }
}

impl MultiSelectState {
    /// Creates a new state for a multi-select with the given number of
    /// options. All options start unselected and enabled. Use
    /// [`from_options`](Self::from_options) or [`sync_options`](Self::sync_options)
    /// to derive enabled/disabled flags from the option slice.
    pub fn new(count: usize) -> Self {
        Self {
            cursor: 0,
            selected: vec![false; count],
            enabled: vec![true; count],
            focused: false,
            validation_message: None,
            validator: None,
        }
    }

    /// Creates a new state from an option slice, picking up each option's
    /// `enabled` flag so navigation, toggling, and rendering stay coherent.
    pub fn from_options(options: &[MultiSelectOption]) -> Self {
        let mut state = Self {
            cursor: 0,
            selected: vec![false; options.len()],
            enabled: options.iter().map(|o| o.enabled).collect(),
            focused: false,
            validation_message: None,
            validator: None,
        };
        // Snap the initial cursor onto the first enabled option, if any.
        state.set_cursor(0);
        state
    }

    /// Synchronises the state with a (possibly changed) option slice.
    ///
    /// Resizes the selection vector (truncating or zero-extending as
    /// needed), refreshes the enabled mask, and clamps the cursor onto
    /// the nearest enabled option.
    pub fn sync_options(&mut self, options: &[MultiSelectOption]) {
        self.selected.resize(options.len(), false);
        self.enabled = options.iter().map(|o| o.enabled).collect();
        self.set_cursor(self.cursor);
    }

    /// Sets a validator function that runs on blur.
    ///
    /// The validator receives the selection state slice.
    /// See [`ValidationResult`] for return type semantics.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn validator(mut self, f: impl Fn(&[bool]) -> ValidationResult + 'static) -> Self {
        self.validator = Some(Box::new(f));
        self
    }

    /// Returns the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns whether the option at `index` is selected.
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.get(index).copied().unwrap_or(false)
    }

    /// Returns the indices of all selected options.
    pub fn selected_indices(&self) -> Vec<usize> {
        self.selected
            .iter()
            .enumerate()
            .filter_map(|(i, &s)| if s { Some(i) } else { None })
            .collect()
    }

    /// Returns the number of currently selected options.
    pub fn selected_count(&self) -> usize {
        self.selected.iter().filter(|&&s| s).count()
    }

    fn is_enabled(&self, index: usize) -> bool {
        self.enabled.get(index).copied().unwrap_or(true)
    }

    /// Moves the cursor to the next enabled option (wraps around).
    ///
    /// Skips disabled options. If all options are disabled, the cursor
    /// does not move.
    pub fn next(&mut self) {
        let total = self.selected.len();
        if total == 0 {
            return;
        }
        for _ in 0..total {
            self.cursor = (self.cursor + 1) % total;
            if self.is_enabled(self.cursor) {
                return;
            }
        }
    }

    /// Moves the cursor to the previous enabled option (wraps around).
    ///
    /// Skips disabled options. If all options are disabled, the cursor
    /// does not move.
    pub fn prev(&mut self) {
        let total = self.selected.len();
        if total == 0 {
            return;
        }
        for _ in 0..total {
            self.cursor = if self.cursor == 0 { total - 1 } else { self.cursor - 1 };
            if self.is_enabled(self.cursor) {
                return;
            }
        }
    }

    /// Toggles the option at the current cursor position.
    ///
    /// Disabled rows are never toggled. If `limit` is `Some(n)` and the
    /// option is currently unchecked, the toggle is only applied when
    /// fewer than `n` options are already selected. Deselecting always
    /// works regardless of limit.
    pub fn toggle_current(&mut self, limit: Option<usize>) {
        if self.cursor >= self.selected.len() || !self.is_enabled(self.cursor) {
            return;
        }
        if self.selected[self.cursor] {
            // Always allow deselect
            self.selected[self.cursor] = false;
        } else if let Some(max) = limit {
            if self.selected_count() < max {
                self.selected[self.cursor] = true;
            }
        } else {
            self.selected[self.cursor] = true;
        }
    }

    /// Selects all enabled options, capped by `limit` if provided.
    ///
    /// The limit counts options that were already selected, so calling
    /// `select_all(Some(n))` never leaves more than `n` items selected.
    pub fn select_all(&mut self, limit: Option<usize>) {
        let mut count = self.selected_count();
        for i in 0..self.selected.len() {
            if let Some(max) = limit
                && count >= max
            {
                break;
            }
            if self.is_enabled(i) && !self.selected[i] {
                self.selected[i] = true;
                count += 1;
            }
        }
    }

    /// Deselects all options.
    pub fn deselect_all(&mut self) {
        self.selected.fill(false);
    }

    /// Sets the cursor directly.
    ///
    /// Clamps to valid range and snaps forward to the nearest enabled
    /// option. If no enabled option exists, the cursor stays at the
    /// clamped position.
    pub fn set_cursor(&mut self, index: usize) {
        if self.selected.is_empty() {
            self.cursor = 0;
            return;
        }
        self.cursor = index.min(self.selected.len() - 1);
        if self.is_enabled(self.cursor) {
            return;
        }
        let total = self.selected.len();
        let start = self.cursor;
        for _ in 0..total {
            self.cursor = (self.cursor + 1) % total;
            if self.is_enabled(self.cursor) {
                return;
            }
        }
        // All disabled — stay at clamped position.
        self.cursor = start;
    }

    /// Sets the selection state for a specific option.
    ///
    /// Disabled options are not modified.
    pub fn set_selected(&mut self, index: usize, selected: bool) {
        if !self.is_enabled(index) {
            return;
        }
        if let Some(s) = self.selected.get_mut(index) {
            *s = selected;
        }
    }

    /// Returns whether the multi-select is focused.
    pub fn focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// When set to `false` (blur), auto-runs the validator if one is set.
    pub fn set_focused(&mut self, focused: bool) {
        let was_focused = self.focused;
        self.focused = focused;

        if was_focused && !focused {
            self.validate();
        }
    }

    /// Runs the validator against the current selections and updates
    /// the validation message.
    ///
    /// Returns `true` if validation passed (or no validator is set),
    /// `false` if validation failed.
    pub fn validate(&mut self) -> bool {
        if let Some(ref validator) = self.validator {
            match validator(&self.selected) {
                Ok(None) => {
                    self.validation_message = None;
                    true
                }
                Ok(Some(msg)) => {
                    self.validation_message = Some((ValidationKind::Success, msg));
                    true
                }
                Err(msg) => {
                    self.validation_message = Some((ValidationKind::Error, msg));
                    false
                }
            }
        } else {
            true
        }
    }

    /// Returns the current validation message, if any.
    pub fn validation(&self) -> Option<&(ValidationKind, String)> {
        self.validation_message.as_ref()
    }

    /// Manually sets or clears the validation message.
    pub fn set_validation(&mut self, validation: Option<(ValidationKind, String)>) {
        self.validation_message = validation;
    }
}

// ---------------------------------------------------------------------------
// Widget + StatefulWidget impls
// ---------------------------------------------------------------------------

impl Widget for MultiSelect<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &MultiSelect<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = MultiSelectState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl StatefulWidget for MultiSelect<'_> {
    type State = MultiSelectState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

impl StatefulWidget for &MultiSelect<'_> {
    type State = MultiSelectState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        let styles = &self.styles;
        let mut y = area.y;

        // 1. Title
        if !self.title.is_empty() {
            buf.set_string(area.x, y, self.title, styles.title);
            y += 1;
        }

        // 2. Description
        if let Some(desc) = self.description
            && y < area.bottom()
            && !desc.is_empty()
        {
            buf.set_string(area.x, y, desc, styles.description);
            y += 1;
        }

        // 3. Options
        let cursor_width = display_width(self.cursor_indicator);
        let indicator_width =
            display_width(self.checked_indicator).max(display_width(self.unchecked_indicator));
        // Layout: [cursor_indicator] [space] [check_indicator] [space] [label]
        let prefix_width = cursor_width + 1 + indicator_width + 1;

        for (i, option) in self.options.iter().enumerate() {
            if y >= area.bottom() {
                break;
            }

            let is_cursor = i == state.cursor;
            let is_checked = state.is_selected(i);
            let label_x = area.x + prefix_width as u16;
            let max_label_width = area.width.saturating_sub(prefix_width as u16) as usize;
            let label = truncate_to_width(option.label, max_label_width);

            if !option.enabled {
                // Disabled: show unchecked indicator + label, all in faint
                let indicator_x = area.x + (cursor_width + 1) as u16;
                buf.set_string(indicator_x, y, self.unchecked_indicator, styles.disabled);
                buf.set_string(label_x, y, &label, styles.disabled);
            } else {
                let indicator =
                    if is_checked { self.checked_indicator } else { self.unchecked_indicator };
                let indicator_x = area.x + (cursor_width + 1) as u16;
                let label_style = if is_checked { styles.checked } else { styles.unchecked };

                if is_cursor {
                    buf.set_string(area.x, y, self.cursor_indicator, styles.cursor);
                    buf.set_string(indicator_x, y, indicator, label_style);
                    buf.set_string(label_x, y, &label, label_style);
                } else {
                    buf.set_string(indicator_x, y, indicator, label_style);
                    buf.set_string(label_x, y, &label, label_style);
                }
            }

            y += 1;
        }

        // 4. Validation message
        if let Some((kind, msg)) = &state.validation_message
            && y < area.bottom()
        {
            let style = match kind {
                ValidationKind::Error => styles.validation_error,
                ValidationKind::Success => styles.validation_success,
            };
            let display = format!("* {msg}");
            buf.set_string(area.x, y, &display, style);
        }
    }
}

/// Truncates a string to fit within `max_width` terminal cells.
fn truncate_to_width(text: &str, max_width: usize) -> String {
    use unicode_width::UnicodeWidthChar;
    let mut result = String::new();
    let mut w = 0;
    for ch in text.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > max_width {
            break;
        }
        result.push(ch);
        w += cw;
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn options<'a>(labels: &'a [&'a str]) -> Vec<MultiSelectOption<'a>> {
        labels.iter().map(|&l| l.into()).collect()
    }

    /// Renders and compares text content only (ignores styles).
    #[track_caller]
    fn assert_renders_content(
        widget: &MultiSelect,
        state: &mut MultiSelectState,
        expected_lines: &[&str],
    ) {
        let width = expected_lines
            .iter()
            .map(|l| unicode_width::UnicodeWidthStr::width(*l))
            .max()
            .unwrap_or(0) as u16;
        let height = expected_lines.len() as u16;
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(widget, area, &mut buf, state);

        let actual_lines: Vec<String> = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| {
                        buf[ratatui::layout::Position::new(x, y)]
                            .symbol()
                            .to_string()
                    })
                    .collect::<String>()
            })
            .collect();

        for (row, (actual, expected)) in actual_lines.iter().zip(expected_lines.iter()).enumerate()
        {
            assert_eq!(
                actual, expected,
                "row {row} mismatch:\n  actual:   {actual:?}\n  expected: {expected:?}"
            );
        }
    }

    // ---- State methods ----

    #[test]
    fn new_state() {
        let state = MultiSelectState::new(5);
        assert_eq!(state.cursor(), 0);
        assert_eq!(state.selected_count(), 0);
        assert!(!state.focused());
        assert!(state.validation().is_none());
    }

    #[test]
    fn next_wraps() {
        let opts = options(&["A", "B", "C"]);
        let mut state = MultiSelectState::from_options(&opts);
        state.next();
        assert_eq!(state.cursor(), 1);
        state.next();
        assert_eq!(state.cursor(), 2);
        state.next();
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn prev_wraps() {
        let opts = options(&["A", "B", "C"]);
        let mut state = MultiSelectState::from_options(&opts);
        state.prev();
        assert_eq!(state.cursor(), 2);
        state.prev();
        assert_eq!(state.cursor(), 1);
        state.prev();
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn next_zero_options() {
        let mut state = MultiSelectState::new(0);
        state.next();
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn next_skips_disabled() {
        let opts = vec![
            MultiSelectOption::new("A"),
            MultiSelectOption::new("B").enabled(false),
            MultiSelectOption::new("C"),
        ];
        let mut state = MultiSelectState::from_options(&opts);
        state.next();
        assert_eq!(state.cursor(), 2);
    }

    #[test]
    fn prev_skips_disabled() {
        let opts = vec![
            MultiSelectOption::new("A"),
            MultiSelectOption::new("B").enabled(false),
            MultiSelectOption::new("C"),
        ];
        let mut state = MultiSelectState::from_options(&opts);
        state.set_cursor(2);
        state.prev();
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn all_disabled_no_move() {
        let opts = vec![
            MultiSelectOption::new("A").enabled(false),
            MultiSelectOption::new("B").enabled(false),
        ];
        let mut state = MultiSelectState::from_options(&opts);
        state.next();
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn toggle_current() {
        let mut state = MultiSelectState::new(3);
        assert!(!state.is_selected(0));
        state.toggle_current(None);
        assert!(state.is_selected(0));
        state.toggle_current(None);
        assert!(!state.is_selected(0));
    }

    #[test]
    fn toggle_respects_limit() {
        let mut state = MultiSelectState::new(3);
        state.toggle_current(Some(1)); // select 0
        assert!(state.is_selected(0));
        state.set_cursor(1);
        state.toggle_current(Some(1)); // can't select 1, limit reached
        assert!(!state.is_selected(1));
    }

    #[test]
    fn toggle_deselect_ignores_limit() {
        let mut state = MultiSelectState::new(3);
        state.toggle_current(Some(1)); // select 0
        state.toggle_current(Some(1)); // deselect 0 — should work
        assert!(!state.is_selected(0));
    }

    #[test]
    fn select_all_no_limit() {
        let opts = options(&["A", "B", "C"]);
        let mut state = MultiSelectState::from_options(&opts);
        state.select_all(None);
        assert_eq!(state.selected_count(), 3);
        assert_eq!(state.selected_indices(), vec![0, 1, 2]);
    }

    #[test]
    fn select_all_with_limit() {
        let opts = options(&["A", "B", "C"]);
        let mut state = MultiSelectState::from_options(&opts);
        state.select_all(Some(2));
        assert_eq!(state.selected_count(), 2);
        assert_eq!(state.selected_indices(), vec![0, 1]);
    }

    #[test]
    fn select_all_skips_disabled() {
        let opts = vec![
            MultiSelectOption::new("A"),
            MultiSelectOption::new("B").enabled(false),
            MultiSelectOption::new("C"),
        ];
        let mut state = MultiSelectState::from_options(&opts);
        state.select_all(None);
        assert!(state.is_selected(0));
        assert!(!state.is_selected(1));
        assert!(state.is_selected(2));
    }

    #[test]
    fn select_all_counts_existing_selections_against_limit() {
        let opts = options(&["A", "B", "C", "D"]);
        let mut state = MultiSelectState::from_options(&opts);
        state.set_selected(0, true);
        state.set_selected(2, true);
        state.select_all(Some(3));
        // Already had 2 selected; only one more slot allowed.
        assert_eq!(state.selected_count(), 3);
        assert_eq!(state.selected_indices(), vec![0, 1, 2]);
    }

    #[test]
    fn select_all_idempotent_when_already_at_limit() {
        let opts = options(&["A", "B", "C"]);
        let mut state = MultiSelectState::from_options(&opts);
        state.set_selected(0, true);
        state.set_selected(1, true);
        state.select_all(Some(2));
        assert_eq!(state.selected_count(), 2);
        assert_eq!(state.selected_indices(), vec![0, 1]);
    }

    #[test]
    fn deselect_all() {
        let mut state = MultiSelectState::new(3);
        state.toggle_current(None);
        state.set_cursor(1);
        state.toggle_current(None);
        assert_eq!(state.selected_count(), 2);
        state.deselect_all();
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn set_cursor_clamps() {
        let mut state = MultiSelectState::new(3);
        state.set_cursor(10);
        assert_eq!(state.cursor(), 2);
    }

    #[test]
    fn set_cursor_empty() {
        let mut state = MultiSelectState::new(0);
        state.set_cursor(5);
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn validator_runs_on_blur() {
        let mut state = MultiSelectState::new(3).validator(|sel| {
            if sel.iter().any(|&s| s) {
                Ok(None)
            } else {
                Err("Select at least one".into())
            }
        });

        state.set_focused(true);
        state.set_focused(false);

        let (kind, msg) = state.validation().unwrap();
        assert_eq!(*kind, ValidationKind::Error);
        assert_eq!(msg, "Select at least one");
    }

    #[test]
    fn validator_clears_on_success() {
        let mut state = MultiSelectState::new(3).validator(|sel| {
            if sel.iter().any(|&s| s) {
                Ok(Some("Looks good!".into()))
            } else {
                Err("Select at least one".into())
            }
        });

        // Fail first
        state.set_focused(true);
        state.set_focused(false);
        assert!(matches!(
            state.validation().unwrap().0,
            ValidationKind::Error
        ));

        // Now select and succeed
        state.toggle_current(None);
        state.set_focused(true);
        state.set_focused(false);
        let (kind, msg) = state.validation().unwrap();
        assert_eq!(*kind, ValidationKind::Success);
        assert_eq!(msg, "Looks good!");
    }

    #[test]
    fn manual_validation() {
        let mut state = MultiSelectState::new(3);
        state.set_validation(Some((ValidationKind::Success, "OK".into())));
        assert!(state.validation().is_some());
        state.set_validation(None);
        assert!(state.validation().is_none());
    }

    // ---- Rendering ----

    #[test]
    fn render_empty_area() {
        let opts = options(&["A", "B"]);
        let multi = MultiSelect::new("Title", &opts);
        let mut state = MultiSelectState::new(2);
        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        StatefulWidget::render(&multi, buf.area, &mut buf, &mut state);
    }

    #[test]
    fn render_basic_unchecked() {
        let opts = options(&["Apple", "Banana", "Cherry"]);
        let multi = MultiSelect::new("Fruit", &opts);
        let mut state = MultiSelectState::new(3);
        assert_renders_content(
            &multi,
            &mut state,
            &[
                "Fruit                    ",
                "> • Apple                ",
                "  • Banana               ",
                "  • Cherry               ",
            ],
        );
    }

    #[test]
    fn render_with_checked() {
        let opts = options(&["Apple", "Banana", "Cherry"]);
        let multi = MultiSelect::new("Fruit", &opts);
        let mut state = MultiSelectState::new(3);
        state.toggle_current(None); // check Apple
        state.set_cursor(2);
        state.toggle_current(None); // check Cherry
        state.set_cursor(0); // cursor back on Apple
        assert_renders_content(
            &multi,
            &mut state,
            &[
                "Fruit                    ",
                "> ✓ Apple                ",
                "  • Banana               ",
                "  ✓ Cherry               ",
            ],
        );
    }

    #[test]
    fn render_cursor_at_second() {
        let opts = options(&["A", "B", "C"]);
        let multi = MultiSelect::new("Pick", &opts);
        let mut state = MultiSelectState::from_options(&opts);
        state.next();
        assert_renders_content(
            &multi,
            &mut state,
            &[
                "Pick                ",
                "  • A               ",
                "> • B               ",
                "  • C               ",
            ],
        );
    }

    #[test]
    fn render_with_description() {
        let opts = options(&["Yes", "No"]);
        let multi = MultiSelect::new("Continue?", &opts).description("Select all that apply.");
        let mut state = MultiSelectState::new(2);
        assert_renders_content(
            &multi,
            &mut state,
            &[
                "Continue?                   ",
                "Select all that apply.      ",
                "> • Yes                     ",
                "  • No                      ",
            ],
        );
    }

    #[test]
    fn render_with_validation_error() {
        let opts = options(&["A", "B"]);
        let multi = MultiSelect::new("Pick", &opts);
        let mut state = MultiSelectState::new(2);
        state.set_validation(Some((ValidationKind::Error, "Select at least one".into())));
        assert_renders_content(
            &multi,
            &mut state,
            &[
                "Pick                       ",
                "> • A                      ",
                "  • B                      ",
                "* Select at least one      ",
            ],
        );
    }

    #[test]
    fn render_disabled_option() {
        let opts = vec![
            MultiSelectOption::new("Mars"),
            MultiSelectOption::new("Europa").enabled(false),
            MultiSelectOption::new("Titan"),
        ];
        let multi = MultiSelect::new("Pick", &opts);
        let mut state = MultiSelectState::new(3);
        assert_renders_content(
            &multi,
            &mut state,
            &[
                "Pick                  ",
                "> • Mars              ",
                "  • Europa            ",
                "  • Titan             ",
            ],
        );

        // Verify disabled option uses faint color
        let area = Rect::new(0, 0, 22, 4);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&multi, area, &mut buf, &mut state);

        let p = Palette::dark();
        let disabled_cell = &buf[ratatui::layout::Position::new(2, 2)];
        assert_eq!(disabled_cell.fg, p.faint);
    }

    #[test]
    fn render_clipped_by_height() {
        let opts = options(&["A", "B", "C", "D", "E"]);
        let multi = MultiSelect::new("Pick", &opts);
        let mut state = MultiSelectState::new(5);
        // Only 3 rows: title + 2 options visible
        assert_renders_content(
            &multi,
            &mut state,
            &[
                "Pick                ",
                "> • A               ",
                "  • B               ",
            ],
        );
    }

    #[test]
    fn render_applies_styles() {
        let opts = options(&["A", "B"]);
        let multi = MultiSelect::new("Title", &opts);
        let mut state = MultiSelectState::new(2);
        state.toggle_current(None); // check A
        let area = Rect::new(0, 0, 20, 4);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&multi, area, &mut buf, &mut state);

        let p = Palette::dark();

        // Title has secondary + bold
        let title_cell = &buf[ratatui::layout::Position::new(0, 0)];
        assert_eq!(title_cell.fg, p.secondary);
        assert!(title_cell.modifier.contains(Modifier::BOLD));

        // Cursor has primary color
        let cursor_cell = &buf[ratatui::layout::Position::new(0, 1)];
        assert_eq!(cursor_cell.fg, p.primary);

        // Checked option indicator has primary color
        let checked_cell = &buf[ratatui::layout::Position::new(2, 1)];
        assert_eq!(checked_cell.fg, p.primary);

        // Unchecked option has foreground color
        let unchecked_cell = &buf[ratatui::layout::Position::new(2, 2)];
        assert_eq!(unchecked_cell.fg, p.foreground);
    }

    #[test]
    fn render_stateless_fallback() {
        let opts = options(&["X", "Y"]);
        let multi = MultiSelect::new("T", &opts);
        let area = Rect::new(0, 0, 20, 4);
        let mut buf = Buffer::empty(area);
        Widget::render(&multi, area, &mut buf);

        let first_option = (0..20u16)
            .map(|x| {
                buf[ratatui::layout::Position::new(x, 1)]
                    .symbol()
                    .to_string()
            })
            .collect::<String>();
        assert!(first_option.contains("X"));
    }

    #[test]
    fn option_from_str() {
        let opt: MultiSelectOption = "hello".into();
        assert_eq!(opt.label, "hello");
        assert!(opt.enabled);
    }

    #[test]
    fn option_disabled() {
        let opt = MultiSelectOption::new("nope").enabled(false);
        assert!(!opt.enabled);
    }

    #[test]
    fn set_selected() {
        let mut state = MultiSelectState::new(3);
        state.set_selected(1, true);
        assert!(state.is_selected(1));
        state.set_selected(1, false);
        assert!(!state.is_selected(1));
    }

    #[test]
    fn set_selected_ignores_disabled() {
        let opts = vec![
            MultiSelectOption::new("A"),
            MultiSelectOption::new("B").enabled(false),
        ];
        let mut state = MultiSelectState::from_options(&opts);
        state.set_selected(1, true);
        assert!(!state.is_selected(1));
    }

    #[test]
    fn set_cursor_snaps_past_disabled() {
        let opts = vec![
            MultiSelectOption::new("A"),
            MultiSelectOption::new("B").enabled(false),
            MultiSelectOption::new("C"),
        ];
        let mut state = MultiSelectState::from_options(&opts);
        state.set_cursor(1);
        assert_eq!(state.cursor(), 2);
    }

    #[test]
    fn toggle_current_refuses_disabled() {
        let opts = vec![
            MultiSelectOption::new("A"),
            MultiSelectOption::new("B").enabled(false),
        ];
        let mut state = MultiSelectState::from_options(&opts);
        // Force the cursor onto a disabled row by bypassing set_cursor's snap.
        // The only way to do that publicly is to construct the state with all
        // options enabled, then sync to a slice where the cursor's option is
        // disabled. But sync_options also snaps — so we instead verify the
        // strongest invariant: even if the cursor lands on a disabled row via
        // a future code path, toggle_current must not flip it.
        state.set_cursor(1);
        // set_cursor snapped forward; cursor is now 0 (only enabled option).
        assert_eq!(state.cursor(), 0);
        // Directly verify: with only one disabled row, toggling does nothing.
        let opts2 = vec![MultiSelectOption::new("only").enabled(false)];
        let mut state2 = MultiSelectState::from_options(&opts2);
        state2.toggle_current(None);
        assert!(!state2.is_selected(0));
    }

    #[test]
    fn from_options_picks_up_enabled_flags() {
        let opts = vec![
            MultiSelectOption::new("A").enabled(false),
            MultiSelectOption::new("B"),
            MultiSelectOption::new("C"),
        ];
        let state = MultiSelectState::from_options(&opts);
        // Cursor snapped onto first enabled option.
        assert_eq!(state.cursor(), 1);
    }

    #[test]
    fn sync_options_truncates_selections() {
        let opts1 = options(&["A", "B", "C", "D"]);
        let mut state = MultiSelectState::from_options(&opts1);
        state.set_selected(0, true);
        state.set_selected(3, true);
        state.set_cursor(3);

        let opts2 = options(&["A", "B"]);
        state.sync_options(&opts2);

        // Selection vec shrank; stale index 3 is gone.
        assert_eq!(state.selected_indices(), vec![0]);
        // Cursor clamped to new length.
        assert_eq!(state.cursor(), 1);
    }

    #[test]
    fn sync_options_does_not_resurrect_selections() {
        let opts1 = options(&["A", "B"]);
        let mut state = MultiSelectState::from_options(&opts1);
        state.set_selected(1, true);

        // Shrink and grow back: the dropped selection must not return.
        state.sync_options(&options(&["A"]));
        state.sync_options(&opts1);

        assert_eq!(state.selected_indices(), Vec::<usize>::new());
    }

    #[test]
    fn sync_options_validator_sees_resized_slice() {
        use std::cell::Cell;
        use std::rc::Rc;
        let observed_len = Rc::new(Cell::new(0));
        let observed_clone = Rc::clone(&observed_len);
        let mut state = MultiSelectState::new(0).validator(move |sel| {
            observed_clone.set(sel.len());
            Ok(None)
        });
        state.sync_options(&options(&["A", "B"]));
        state.set_selected(0, true);
        state.validate();
        assert_eq!(observed_len.get(), 2);

        state.sync_options(&options(&["X"]));
        state.validate();
        assert_eq!(observed_len.get(), 1);
    }

    #[test]
    fn sync_options_refreshes_enabled_mask() {
        let opts1 = options(&["A", "B", "C"]);
        let mut state = MultiSelectState::from_options(&opts1);
        state.set_cursor(2);

        // Disable C; cursor should snap off it.
        let opts2 = vec![
            MultiSelectOption::new("A"),
            MultiSelectOption::new("B"),
            MultiSelectOption::new("C").enabled(false),
        ];
        state.sync_options(&opts2);
        assert_eq!(state.cursor(), 0);
    }
}
