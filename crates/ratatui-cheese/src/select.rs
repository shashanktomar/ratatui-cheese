//! Single-selection widget.
//!
//! A form field for picking one option from a vertical list. Inspired
//! by Charmbracelet's [huh](https://github.com/charmbracelet/huh) select field.
//!
//! # Example
//!
//! ```rust
//! use ratatui_cheese::select::{Select, SelectOption, SelectState};
//!
//! let options: Vec<SelectOption> = vec!["Apple".into(), "Banana".into(), "Cherry".into()];
//! let select = Select::new("Pick a fruit", &options);
//!
//! let mut state = SelectState::new(options.len());
//! state.next();
//! assert_eq!(state.selected(), 1);
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

/// A single option in a [`Select`] widget.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SelectOption<'a> {
    /// Display label for this option.
    pub label: &'a str,
    /// Whether this option can be selected. Disabled options are
    /// rendered in a faint style and skipped by cursor navigation.
    pub enabled: bool,
}

impl<'a> SelectOption<'a> {
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

impl<'a> From<&'a str> for SelectOption<'a> {
    fn from(label: &'a str) -> Self {
        Self::new(label)
    }
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Visual styles for the [`Select`] widget.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SelectStyles {
    /// Title text.
    pub title: Style,
    /// Description text below the title.
    pub description: Style,
    /// Cursor indicator character.
    pub cursor: Style,
    /// Option at cursor position.
    pub selected_option: Style,
    /// Unselected option text.
    pub option: Style,
    /// Disabled option text.
    pub disabled: Style,
    /// Validation error message.
    pub validation_error: Style,
    /// Validation success message.
    pub validation_success: Style,
}

impl Default for SelectStyles {
    fn default() -> Self {
        Self::dark()
    }
}

impl SelectStyles {
    /// Creates styles from a [`Palette`].
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            title: Style::default()
                .fg(p.secondary)
                .add_modifier(Modifier::BOLD),
            description: Style::default().fg(p.muted),
            cursor: Style::default().fg(p.primary),
            selected_option: Style::default().fg(p.primary),
            option: Style::default().fg(p.foreground),
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

/// A single-selection widget.
///
/// `Select` holds appearance configuration (title, options, styles).
/// Mutable state (cursor position, validation) lives in [`SelectState`].
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::select::{Select, SelectOption};
///
/// let options: Vec<SelectOption> = vec!["Red".into(), "Green".into(), "Blue".into()];
/// let select = Select::new("Color", &options)
///     .description("Pick your favorite.");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Select<'a> {
    title: &'a str,
    description: Option<&'a str>,
    options: &'a [SelectOption<'a>],
    cursor_indicator: &'a str,
    styles: SelectStyles,
}

impl<'a> Select<'a> {
    /// Creates a new select with the given title and options.
    #[must_use]
    pub fn new(title: &'a str, options: &'a [SelectOption<'a>]) -> Self {
        Self {
            title,
            description: None,
            options,
            cursor_indicator: ">",
            styles: SelectStyles::default(),
        }
    }

    /// Sets the description shown below the title.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the cursor indicator (default: ">").
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn cursor_indicator(mut self, indicator: &'a str) -> Self {
        self.cursor_indicator = indicator;
        self
    }

    /// Sets the styles for this select.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn styles(mut self, styles: SelectStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Sets styles from a [`Palette`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn palette(mut self, palette: &Palette) -> Self {
        self.styles = SelectStyles::from_palette(palette);
        self
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Mutable state for a [`Select`] widget.
///
/// Holds cursor position, focus state, and optional validation.
/// The validator (if set) runs automatically on blur.
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::select::{SelectOption, SelectState};
///
/// let options: Vec<SelectOption> = vec!["A".into(), "B".into(), "C".into()];
/// let mut state = SelectState::new(options.len());
/// assert_eq!(state.selected(), 0);
///
/// state.next();
/// assert_eq!(state.selected(), 1);
/// ```
pub struct SelectState {
    cursor: usize,
    total: usize,
    enabled: Vec<bool>,
    focused: bool,
    validation_message: Option<(ValidationKind, String)>,
    validator: Option<Box<dyn Fn(usize) -> ValidationResult>>,
}

impl std::fmt::Debug for SelectState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectState")
            .field("cursor", &self.cursor)
            .field("total", &self.total)
            .field("enabled", &self.enabled)
            .field("focused", &self.focused)
            .field("validation_message", &self.validation_message)
            .field("validator", &self.validator.as_ref().map(|_| ".."))
            .finish()
    }
}

impl Default for SelectState {
    fn default() -> Self {
        Self::new(0)
    }
}

impl SelectState {
    /// Creates a new state for a select with the given number of options.
    ///
    /// All options are enabled by default. Use [`set_enabled`](Self::set_enabled)
    /// to disable specific options.
    pub fn new(total: usize) -> Self {
        Self {
            cursor: 0,
            total,
            enabled: vec![true; total],
            focused: false,
            validation_message: None,
            validator: None,
        }
    }

    /// Marks an option as enabled or disabled.
    ///
    /// Disabled options are skipped by [`next`](Self::next) and
    /// [`prev`](Self::prev). If the cursor is currently on the given
    /// index and it becomes disabled, the cursor is **not** moved
    /// automatically — call `next` or `prev` to advance it.
    pub fn set_enabled(&mut self, index: usize, enabled: bool) {
        if index < self.enabled.len() {
            self.enabled[index] = enabled;
        }
    }

    /// Updates the total number of options and clamps the cursor.
    pub fn set_total(&mut self, total: usize) {
        self.total = total;
        self.enabled.resize(total, true);
        if total == 0 {
            self.cursor = 0;
        } else {
            self.cursor = self.cursor.min(total - 1);
        }
    }

    /// Sets a validator function that runs on blur.
    ///
    /// The validator receives the selected index.
    /// See [`ValidationResult`] for return type semantics.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn validator(mut self, f: impl Fn(usize) -> ValidationResult + 'static) -> Self {
        self.validator = Some(Box::new(f));
        self
    }

    /// Returns the currently selected index.
    pub fn selected(&self) -> usize {
        self.cursor
    }

    /// Moves the cursor to the next enabled option (wraps around).
    ///
    /// Skips disabled options. If all options are disabled, the cursor
    /// does not move.
    pub fn next(&mut self) {
        if self.total == 0 {
            return;
        }
        for _ in 0..self.total {
            self.cursor = (self.cursor + 1) % self.total;
            if self.enabled.get(self.cursor).copied().unwrap_or(true) {
                return;
            }
        }
    }

    /// Moves the cursor to the previous enabled option (wraps around).
    ///
    /// Skips disabled options. If all options are disabled, the cursor
    /// does not move.
    pub fn prev(&mut self) {
        if self.total == 0 {
            return;
        }
        for _ in 0..self.total {
            self.cursor = if self.cursor == 0 { self.total - 1 } else { self.cursor - 1 };
            if self.enabled.get(self.cursor).copied().unwrap_or(true) {
                return;
            }
        }
    }

    /// Sets the cursor directly (clamps to valid range).
    pub fn set_cursor(&mut self, index: usize) {
        if self.total == 0 {
            self.cursor = 0;
        } else {
            self.cursor = index.min(self.total - 1);
        }
    }

    /// Returns whether the select is focused.
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

    /// Runs the validator against the current selection and updates the
    /// validation message.
    ///
    /// Returns `true` if validation passed (or no validator is set),
    /// `false` if validation failed.
    pub fn validate(&mut self) -> bool {
        if let Some(ref validator) = self.validator {
            match validator(self.cursor) {
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

impl Widget for Select<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Select<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = SelectState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl StatefulWidget for Select<'_> {
    type State = SelectState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

impl StatefulWidget for &Select<'_> {
    type State = SelectState;

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

        // 3. Options — clamp cursor to actual options length
        if !self.options.is_empty() {
            state.cursor = state.cursor.min(self.options.len() - 1);
        }
        let cursor_width = display_width(self.cursor_indicator);
        let indent = cursor_width + 1; // cursor + space

        for (i, option) in self.options.iter().enumerate() {
            if y >= area.bottom() {
                break;
            }

            let is_cursor = i == state.cursor;
            let label_x = area.x + indent as u16;
            let max_label_width = area.width.saturating_sub(indent as u16) as usize;
            let label = truncate_to_width(option.label, max_label_width);

            if !option.enabled {
                buf.set_string(label_x, y, &label, styles.disabled);
            } else if is_cursor {
                buf.set_string(area.x, y, self.cursor_indicator, styles.cursor);
                buf.set_string(label_x, y, &label, styles.selected_option);
            } else {
                buf.set_string(label_x, y, &label, styles.option);
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

    fn options<'a>(labels: &'a [&'a str]) -> Vec<SelectOption<'a>> {
        labels.iter().map(|&l| l.into()).collect()
    }

    /// Renders and compares text content only (ignores styles).
    #[track_caller]
    fn assert_renders_content(select: &Select, state: &mut SelectState, expected_lines: &[&str]) {
        let width = expected_lines
            .iter()
            .map(|l| unicode_width::UnicodeWidthStr::width(*l))
            .max()
            .unwrap_or(0) as u16;
        let height = expected_lines.len() as u16;
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(select, area, &mut buf, state);

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
        let state = SelectState::new(5);
        assert_eq!(state.selected(), 0);
        assert!(!state.focused());
        assert!(state.validation().is_none());
    }

    #[test]
    fn next_wraps() {
        let mut state = SelectState::new(3);
        state.next();
        assert_eq!(state.selected(), 1);
        state.next();
        assert_eq!(state.selected(), 2);
        state.next();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn prev_wraps() {
        let mut state = SelectState::new(3);
        state.prev();
        assert_eq!(state.selected(), 2);
        state.prev();
        assert_eq!(state.selected(), 1);
        state.prev();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn next_zero_options() {
        let mut state = SelectState::new(0);
        state.next();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn next_skips_disabled() {
        let mut state = SelectState::new(3);
        state.set_enabled(1, false);
        state.next(); // skips index 1, lands on 2
        assert_eq!(state.selected(), 2);
    }

    #[test]
    fn prev_skips_disabled() {
        let mut state = SelectState::new(3);
        state.set_enabled(1, false);
        state.set_cursor(2);
        state.prev(); // skips index 1, lands on 0
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn all_disabled_no_move() {
        let mut state = SelectState::new(2);
        state.set_enabled(0, false);
        state.set_enabled(1, false);
        state.next();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn set_cursor_clamps() {
        let mut state = SelectState::new(3);
        state.set_cursor(10);
        assert_eq!(state.selected(), 2);
    }

    #[test]
    fn set_cursor_zero_total() {
        let mut state = SelectState::new(0);
        state.set_cursor(5);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn set_total_shrinks_clamps_cursor() {
        let mut state = SelectState::new(5);
        state.set_cursor(4);
        state.set_total(2);
        assert_eq!(state.selected(), 1);
    }

    #[test]
    fn validator_runs_on_blur() {
        let mut state = SelectState::new(3).validator(|i| {
            if i == 1 { Err("Not allowed".into()) } else { Ok(None) }
        });

        state.set_cursor(1);
        state.set_focused(true);
        state.set_focused(false);

        let (kind, msg) = state.validation().unwrap();
        assert_eq!(*kind, ValidationKind::Error);
        assert_eq!(msg, "Not allowed");
    }

    #[test]
    fn validator_clears_on_success() {
        let mut state = SelectState::new(3).validator(|i| {
            if i == 1 { Err("Not allowed".into()) } else { Ok(None) }
        });

        state.set_cursor(1);
        state.set_focused(true);
        state.set_focused(false);
        assert!(state.validation().is_some());

        state.set_cursor(0);
        state.set_focused(true);
        state.set_focused(false);
        assert!(state.validation().is_none());
    }

    #[test]
    fn manual_validation() {
        let mut state = SelectState::new(3);
        state.set_validation(Some((ValidationKind::Success, "Good choice!".into())));
        assert!(state.validation().is_some());

        state.set_validation(None);
        assert!(state.validation().is_none());
    }

    // ---- Rendering ----

    #[test]
    fn render_empty_area() {
        let opts = options(&["A", "B"]);
        let select = Select::new("Title", &opts);
        let mut state = SelectState::new(2);
        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        StatefulWidget::render(&select, buf.area, &mut buf, &mut state);
    }

    #[test]
    fn render_basic() {
        let opts = options(&["Apple", "Banana", "Cherry"]);
        let select = Select::new("Fruit", &opts);
        let mut state = SelectState::new(3);
        assert_renders_content(
            &select,
            &mut state,
            &[
                "Fruit              ",
                "> Apple            ",
                "  Banana           ",
                "  Cherry           ",
            ],
        );
    }

    #[test]
    fn render_cursor_at_second() {
        let opts = options(&["A", "B", "C"]);
        let select = Select::new("Pick", &opts);
        let mut state = SelectState::new(3);
        state.next();
        assert_renders_content(
            &select,
            &mut state,
            &[
                "Pick          ",
                "  A           ",
                "> B           ",
                "  C           ",
            ],
        );
    }

    #[test]
    fn render_with_description() {
        let opts = options(&["Yes", "No"]);
        let select = Select::new("Continue?", &opts).description("Are you sure?");
        let mut state = SelectState::new(2);
        assert_renders_content(
            &select,
            &mut state,
            &[
                "Continue?          ",
                "Are you sure?      ",
                "> Yes              ",
                "  No               ",
            ],
        );
    }

    #[test]
    fn render_custom_cursor() {
        let opts = options(&["Red", "Blue"]);
        let select = Select::new("Color", &opts).cursor_indicator("→");
        let mut state = SelectState::new(2);
        assert_renders_content(
            &select,
            &mut state,
            &["Color          ", "→ Red          ", "  Blue         "],
        );
    }

    #[test]
    fn render_with_validation_error() {
        let opts = options(&["A", "B"]);
        let select = Select::new("Pick", &opts);
        let mut state = SelectState::new(2);
        state.set_validation(Some((ValidationKind::Error, "Invalid choice".into())));
        assert_renders_content(
            &select,
            &mut state,
            &[
                "Pick               ",
                "> A                ",
                "  B                ",
                "* Invalid choice   ",
            ],
        );
    }

    #[test]
    fn render_stateless_fallback() {
        let opts = options(&["X", "Y"]);
        let select = Select::new("T", &opts);
        let area = Rect::new(0, 0, 15, 4);
        let mut buf = Buffer::empty(area);
        Widget::render(&select, area, &mut buf);

        let first_option = (0..15u16)
            .map(|x| {
                buf[ratatui::layout::Position::new(x, 1)]
                    .symbol()
                    .to_string()
            })
            .collect::<String>();
        assert!(first_option.contains("X"));
    }

    #[test]
    fn render_clipped_by_height() {
        let opts = options(&["A", "B", "C", "D", "E"]);
        let select = Select::new("Pick", &opts);
        let mut state = SelectState::new(5);
        // Only 3 rows of height: title + 2 options visible
        assert_renders_content(
            &select,
            &mut state,
            &["Pick          ", "> A           ", "  B           "],
        );
    }

    #[test]
    fn render_applies_styles() {
        let opts = options(&["A", "B"]);
        let select = Select::new("Title", &opts);
        let mut state = SelectState::new(2);
        let area = Rect::new(0, 0, 15, 4);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&select, area, &mut buf, &mut state);

        let p = Palette::dark();

        // Title has secondary + bold
        let title_cell = &buf[ratatui::layout::Position::new(0, 0)];
        assert_eq!(title_cell.fg, p.secondary);
        assert!(title_cell.modifier.contains(Modifier::BOLD));

        // Cursor has primary color
        let cursor_cell = &buf[ratatui::layout::Position::new(0, 1)];
        assert_eq!(cursor_cell.fg, p.primary);

        // Selected option has primary color
        let selected_cell = &buf[ratatui::layout::Position::new(2, 1)];
        assert_eq!(selected_cell.fg, p.primary);

        // Unselected option has foreground color
        let unselected_cell = &buf[ratatui::layout::Position::new(2, 2)];
        assert_eq!(unselected_cell.fg, p.foreground);
    }

    #[test]
    fn option_from_str() {
        let opt: SelectOption = "hello".into();
        assert_eq!(opt.label, "hello");
        assert!(opt.enabled);
    }

    #[test]
    fn option_disabled() {
        let opt = SelectOption::new("nope").enabled(false);
        assert!(!opt.enabled);
    }

    #[test]
    fn render_disabled_option() {
        let opts = vec![
            SelectOption::new("Mars"),
            SelectOption::new("Europa").enabled(false),
            SelectOption::new("Titan"),
        ];
        let select = Select::new("Pick", &opts);
        let mut state = SelectState::new(3);
        assert_renders_content(
            &select,
            &mut state,
            &[
                "Pick               ",
                "> Mars             ",
                "  Europa           ",
                "  Titan            ",
            ],
        );

        // Verify disabled option uses faint color
        let area = Rect::new(0, 0, 19, 4);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&select, area, &mut buf, &mut state);

        let p = Palette::dark();
        let disabled_cell = &buf[ratatui::layout::Position::new(2, 2)];
        assert_eq!(disabled_cell.fg, p.faint);
    }
}
