//! Single-line text input widget.
//!
//! A form field for entering text, passwords, or other short input. Inspired
//! by Charmbracelet's [huh](https://github.com/charmbracelet/huh) input field.
//!
//! # Example
//!
//! ```rust
//! use ratatui_cheese::input::{Input, InputState};
//!
//! let input = Input::new("What's your name?")
//!     .description("For when your order is ready.")
//!     .placeholder("Enter name...");
//!
//! let mut state = InputState::new();
//! state.insert_char('H');
//! state.insert_char('i');
//! assert_eq!(state.value(), "Hi");
//! ```

use crate::field::{ValidationKind, ValidationResult};
use crate::theme::Palette;
use crate::utils::display_width;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{StatefulWidget, Widget};
use unicode_width::UnicodeWidthChar;

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Visual styles for the [`Input`] widget.
///
/// Each field maps to a distinct visual element. Use [`from_palette`](Self::from_palette)
/// to derive a consistent set from a [`Palette`].
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InputStyles {
    /// Title text (e.g., "What's your name?").
    pub title: Style,
    /// Description text below the title.
    pub description: Style,
    /// Prompt indicator (e.g., ">").
    pub prompt: Style,
    /// Entered text.
    pub text: Style,
    /// Placeholder text shown when empty.
    pub placeholder: Style,
    /// Visual cursor (inverted cell at cursor position).
    pub cursor: Style,
    /// Validation error message.
    pub validation_error: Style,
    /// Validation success message.
    pub validation_success: Style,
}

impl Default for InputStyles {
    fn default() -> Self {
        Self::dark()
    }
}

impl InputStyles {
    /// Creates styles from a [`Palette`].
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            title: Style::default()
                .fg(p.secondary)
                .add_modifier(Modifier::BOLD),
            description: Style::default().fg(p.muted),
            prompt: Style::default().fg(p.primary),
            text: Style::default().fg(p.foreground),
            placeholder: Style::default().fg(p.faint),
            cursor: Style::default().fg(p.surface).bg(p.foreground),
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

/// A single-line text input widget.
///
/// `Input` holds appearance configuration (title, placeholder, styles).
/// Mutable state (value, cursor position, validation) lives in [`InputState`].
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::input::Input;
///
/// let input = Input::new("Email")
///     .description("We'll never share your email.")
///     .placeholder("you@example.com");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Input<'a> {
    title: &'a str,
    description: Option<&'a str>,
    placeholder: Option<&'a str>,
    prompt: &'a str,
    char_limit: Option<usize>,
    password_mode: bool,
    password_char: char,
    styles: InputStyles,
}

impl<'a> Input<'a> {
    /// Creates a new input with the given title.
    #[must_use]
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
            placeholder: None,
            prompt: ">",
            char_limit: None,
            password_mode: false,
            password_char: '*',
            styles: InputStyles::default(),
        }
    }

    /// Sets the description shown below the title.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the placeholder text shown when the input is empty.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Sets the prompt indicator (default: ">").
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn prompt(mut self, prompt: &'a str) -> Self {
        self.prompt = prompt;
        self
    }

    /// Sets the maximum number of characters allowed.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    /// Enables password mode — entered text is masked with [`password_char`](Self::password_char).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn password_mode(mut self, enabled: bool) -> Self {
        self.password_mode = enabled;
        self
    }

    /// Sets the masking character for password mode (default: '*').
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn password_char(mut self, ch: char) -> Self {
        self.password_char = ch;
        self
    }

    /// Sets the styles for this input.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn styles(mut self, styles: InputStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Sets styles from a [`Palette`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn palette(mut self, palette: &Palette) -> Self {
        self.styles = InputStyles::from_palette(palette);
        self
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Mutable state for an [`Input`] widget.
///
/// Holds the text value, cursor position, focus state, and optional
/// validation. The validator (if set) runs automatically on blur.
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::input::InputState;
///
/// let mut state = InputState::new();
/// state.insert_char('H');
/// state.insert_char('i');
/// assert_eq!(state.value(), "Hi");
/// assert_eq!(state.cursor_pos(), 2);
///
/// state.move_left();
/// assert_eq!(state.cursor_pos(), 1);
/// ```
pub struct InputState {
    value: String,
    cursor_pos: usize,
    focused: bool,
    validation_message: Option<(ValidationKind, String)>,
    validator: Option<ValidatorFn>,
}

type ValidatorFn = Box<dyn Fn(&str) -> ValidationResult>;

impl std::fmt::Debug for InputState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputState")
            .field("value", &self.value)
            .field("cursor_pos", &self.cursor_pos)
            .field("focused", &self.focused)
            .field("validation_message", &self.validation_message)
            .field("validator", &self.validator.as_ref().map(|_| ".."))
            .finish()
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    /// Creates a new empty input state.
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor_pos: 0,
            focused: false,
            validation_message: None,
            validator: None,
        }
    }

    /// Sets a validator function that runs on blur or when [`validate()`](Self::validate)
    /// is called.
    ///
    /// See [`ValidationResult`] for the return type semantics.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn validator(mut self, f: impl Fn(&str) -> ValidationResult + 'static) -> Self {
        self.validator = Some(Box::new(f));
        self
    }

    /// Returns the current text value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the text value and clamps the cursor.
    pub fn set_value(&mut self, value: String) {
        let char_count = value.chars().count();
        self.value = value;
        self.cursor_pos = self.cursor_pos.min(char_count);
    }

    /// Returns the cursor position (char index, not byte index).
    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    /// Returns whether the input is focused.
    pub fn focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// When set to `false` (blur), auto-runs the validator if one is set.
    pub fn set_focused(&mut self, focused: bool) {
        let was_focused = self.focused;
        self.focused = focused;

        // Run validator on blur
        if was_focused && !focused {
            self.validate();
        }
    }

    /// Runs the validator against the current value and updates the
    /// validation message.
    ///
    /// Returns `true` if validation passed (or no validator is set),
    /// `false` if validation failed.
    pub fn validate(&mut self) -> bool {
        if let Some(ref validator) = self.validator {
            match validator(&self.value) {
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

    /// Inserts a character at the cursor position and advances the cursor.
    ///
    /// Respects the char limit set on the [`Input`] widget. Pass the limit
    /// from the widget, or `None` if unlimited.
    pub fn insert_char(&mut self, ch: char) {
        let byte_idx = self.byte_offset(self.cursor_pos);
        self.value.insert(byte_idx, ch);
        self.cursor_pos += 1;
    }

    /// Inserts a character at the cursor, respecting a char limit.
    ///
    /// Returns `false` if the limit was reached and the char was not inserted.
    pub fn insert_char_limited(&mut self, ch: char, limit: Option<usize>) -> bool {
        if let Some(limit) = limit
            && self.value.chars().count() >= limit
        {
            return false;
        }
        self.insert_char(ch);
        true
    }

    /// Deletes the character before the cursor (backspace).
    pub fn delete_before(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let start = self.byte_offset(self.cursor_pos - 1);
        let end = self.byte_offset(self.cursor_pos);
        self.value.drain(start..end);
        self.cursor_pos -= 1;
    }

    /// Deletes the character at the cursor (delete key).
    pub fn delete_at(&mut self) {
        let char_count = self.value.chars().count();
        if self.cursor_pos >= char_count {
            return;
        }
        let start = self.byte_offset(self.cursor_pos);
        let end = self.byte_offset(self.cursor_pos + 1);
        self.value.drain(start..end);
    }

    /// Moves the cursor one character to the left.
    pub fn move_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    /// Moves the cursor one character to the right.
    pub fn move_right(&mut self) {
        let char_count = self.value.chars().count();
        if self.cursor_pos < char_count {
            self.cursor_pos += 1;
        }
    }

    /// Moves the cursor to the beginning.
    pub fn home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Moves the cursor to the end.
    pub fn end(&mut self) {
        self.cursor_pos = self.value.chars().count();
    }

    /// Converts a char index to a byte offset in the value string.
    fn byte_offset(&self, char_idx: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len())
    }
}

// ---------------------------------------------------------------------------
// Widget + StatefulWidget impls
// ---------------------------------------------------------------------------

// Widget for owned — delegates to &ref
impl Widget for Input<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

// Widget for &ref — stateless fallback with default state
impl Widget for &Input<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = InputState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// StatefulWidget for owned — delegates to &ref
impl StatefulWidget for Input<'_> {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

// StatefulWidget for &ref — the real render logic
impl StatefulWidget for &Input<'_> {
    type State = InputState;

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

        // 3. Prompt + text line
        if y < area.bottom() {
            let prompt_with_space = format!("{} ", self.prompt);
            let prompt_width = display_width(&prompt_with_space);
            buf.set_string(area.x, y, &prompt_with_space, styles.prompt);

            let text_x = area.x + prompt_width as u16;
            let text_max_width = area.width.saturating_sub(prompt_width as u16) as usize;

            if state.value.is_empty() {
                // Show placeholder
                if let Some(placeholder) = self.placeholder {
                    let display = truncate_to_width(placeholder, text_max_width);
                    buf.set_string(text_x, y, &display, styles.placeholder);
                }
                // Cursor at start of text area when focused
                if state.focused && text_x < area.right() {
                    buf[ratatui::layout::Position::new(text_x, y)].set_style(styles.cursor);
                }
            } else {
                // Render text (or masked text)
                let display_text: String = if self.password_mode {
                    std::iter::repeat_n(self.password_char, state.value.chars().count()).collect()
                } else {
                    state.value.clone()
                };

                let display = truncate_to_width(&display_text, text_max_width);
                buf.set_string(text_x, y, &display, styles.text);

                // Visual cursor
                if state.focused {
                    let cursor_display_x = if self.password_mode {
                        // Each password char has width 1 (for ASCII mask chars)
                        state.cursor_pos * UnicodeWidthChar::width(self.password_char).unwrap_or(1)
                    } else {
                        // Sum display widths of chars before cursor
                        state
                            .value
                            .chars()
                            .take(state.cursor_pos)
                            .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
                            .sum()
                    };

                    let abs_x = text_x + cursor_display_x as u16;
                    if abs_x < area.right() {
                        buf[ratatui::layout::Position::new(abs_x, y)].set_style(styles.cursor);
                    }
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

    /// Renders and compares text content only (ignores styles).
    #[track_caller]
    fn assert_renders_content(input: &Input, state: &mut InputState, expected_lines: &[&str]) {
        let width = expected_lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
        let height = expected_lines.len() as u16;
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(input, area, &mut buf, state);

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
    fn new_state_is_empty() {
        let state = InputState::new();
        assert_eq!(state.value(), "");
        assert_eq!(state.cursor_pos(), 0);
        assert!(!state.focused());
        assert!(state.validation().is_none());
    }

    #[test]
    fn insert_char_advances_cursor() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.insert_char('b');
        assert_eq!(state.value(), "ab");
        assert_eq!(state.cursor_pos(), 2);
    }

    #[test]
    fn insert_char_at_middle() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.insert_char('c');
        state.move_left();
        state.insert_char('b');
        assert_eq!(state.value(), "abc");
        assert_eq!(state.cursor_pos(), 2);
    }

    #[test]
    fn insert_char_limited_respects_limit() {
        let mut state = InputState::new();
        assert!(state.insert_char_limited('a', Some(2)));
        assert!(state.insert_char_limited('b', Some(2)));
        assert!(!state.insert_char_limited('c', Some(2)));
        assert_eq!(state.value(), "ab");
    }

    #[test]
    fn insert_char_limited_none_is_unlimited() {
        let mut state = InputState::new();
        for _ in 0..100 {
            assert!(state.insert_char_limited('x', None));
        }
        assert_eq!(state.value().len(), 100);
    }

    #[test]
    fn delete_before_at_start_is_noop() {
        let mut state = InputState::new();
        state.delete_before();
        assert_eq!(state.value(), "");
        assert_eq!(state.cursor_pos(), 0);
    }

    #[test]
    fn delete_before_removes_char() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.insert_char('b');
        state.delete_before();
        assert_eq!(state.value(), "a");
        assert_eq!(state.cursor_pos(), 1);
    }

    #[test]
    fn delete_at_removes_char_under_cursor() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.insert_char('b');
        state.home();
        state.delete_at();
        assert_eq!(state.value(), "b");
        assert_eq!(state.cursor_pos(), 0);
    }

    #[test]
    fn delete_at_end_is_noop() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.delete_at();
        assert_eq!(state.value(), "a");
    }

    #[test]
    fn move_left_and_right() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.insert_char('b');
        assert_eq!(state.cursor_pos(), 2);

        state.move_left();
        assert_eq!(state.cursor_pos(), 1);

        state.move_left();
        assert_eq!(state.cursor_pos(), 0);

        // Can't go further left
        state.move_left();
        assert_eq!(state.cursor_pos(), 0);

        state.move_right();
        assert_eq!(state.cursor_pos(), 1);
    }

    #[test]
    fn move_right_stops_at_end() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.move_right();
        assert_eq!(state.cursor_pos(), 1);
    }

    #[test]
    fn home_and_end() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.insert_char('b');
        state.insert_char('c');

        state.home();
        assert_eq!(state.cursor_pos(), 0);

        state.end();
        assert_eq!(state.cursor_pos(), 3);
    }

    #[test]
    fn set_value_clamps_cursor() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.insert_char('b');
        state.insert_char('c');
        assert_eq!(state.cursor_pos(), 3);

        state.set_value("x".into());
        assert_eq!(state.cursor_pos(), 1);
    }

    #[test]
    fn unicode_chars() {
        let mut state = InputState::new();
        state.insert_char('é');
        state.insert_char('ñ');
        assert_eq!(state.value(), "éñ");
        assert_eq!(state.cursor_pos(), 2);

        state.delete_before();
        assert_eq!(state.value(), "é");
    }

    // ---- Validator ----

    #[test]
    fn validator_runs_on_blur() {
        let mut state = InputState::new().validator(|v| {
            if v.is_empty() { Err("Required".into()) } else { Ok(None) }
        });

        state.set_focused(true);
        state.set_focused(false); // blur triggers validation

        let (kind, msg) = state.validation().unwrap();
        assert_eq!(*kind, ValidationKind::Error);
        assert_eq!(msg, "Required");
    }

    #[test]
    fn validator_clears_on_success() {
        let mut state = InputState::new().validator(|v| {
            if v.is_empty() { Err("Required".into()) } else { Ok(None) }
        });

        state.set_focused(true);
        state.set_focused(false); // fails
        assert!(state.validation().is_some());

        state.insert_char('x');
        state.set_focused(true);
        state.set_focused(false); // passes
        assert!(state.validation().is_none());
    }

    #[test]
    fn manual_validation_override() {
        let mut state = InputState::new();
        state.set_validation(Some((ValidationKind::Success, "Looks good!".into())));

        let (kind, msg) = state.validation().unwrap();
        assert_eq!(*kind, ValidationKind::Success);
        assert_eq!(msg, "Looks good!");

        state.set_validation(None);
        assert!(state.validation().is_none());
    }

    #[test]
    fn no_validator_blur_does_not_touch_validation() {
        let mut state = InputState::new();
        state.set_validation(Some((ValidationKind::Success, "OK".into())));

        state.set_focused(true);
        state.set_focused(false);

        // Manual validation preserved when no validator is set
        assert!(state.validation().is_some());
    }

    // ---- Rendering ----

    #[test]
    fn render_empty_area() {
        let input = Input::new("Title");
        let mut state = InputState::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        StatefulWidget::render(&input, buf.area, &mut buf, &mut state);
        // No panic = pass
    }

    #[test]
    fn render_title_only() {
        let input = Input::new("Name");
        let mut state = InputState::new();
        assert_renders_content(
            &input,
            &mut state,
            &[
                "Name                ",
                ">                   ",
                "                    ",
            ],
        );
    }

    #[test]
    fn render_with_description() {
        let input = Input::new("Name").description("Your full name");
        let mut state = InputState::new();
        assert_renders_content(
            &input,
            &mut state,
            &[
                "Name                ",
                "Your full name      ",
                ">                   ",
            ],
        );
    }

    #[test]
    fn render_with_value() {
        let input = Input::new("Name");
        let mut state = InputState::new();
        state.insert_char('H');
        state.insert_char('i');
        assert_renders_content(
            &input,
            &mut state,
            &[
                "Name                ",
                "> Hi                ",
                "                    ",
            ],
        );
    }

    #[test]
    fn render_with_placeholder() {
        let input = Input::new("Name").placeholder("Enter name...");
        let mut state = InputState::new();
        assert_renders_content(
            &input,
            &mut state,
            &[
                "Name                ",
                "> Enter name...     ",
                "                    ",
            ],
        );
    }

    #[test]
    fn render_password_mode() {
        let input = Input::new("Password").password_mode(true);
        let mut state = InputState::new();
        state.insert_char('s');
        state.insert_char('e');
        state.insert_char('c');
        assert_renders_content(
            &input,
            &mut state,
            &[
                "Password            ",
                "> ***               ",
                "                    ",
            ],
        );
    }

    #[test]
    fn render_with_validation_error() {
        let input = Input::new("Name");
        let mut state = InputState::new();
        state.set_validation(Some((ValidationKind::Error, "Required field".into())));
        assert_renders_content(
            &input,
            &mut state,
            &[
                "Name                ",
                ">                   ",
                "* Required field    ",
            ],
        );
    }

    #[test]
    fn render_stateless_fallback() {
        let input = Input::new("Name");
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        Widget::render(&input, area, &mut buf);

        let actual_lines: Vec<String> = (0..3)
            .map(|y| {
                (0..20u16)
                    .map(|x| {
                        buf[ratatui::layout::Position::new(x, y)]
                            .symbol()
                            .to_string()
                    })
                    .collect::<String>()
            })
            .collect();

        assert_eq!(actual_lines[0], "Name                ");
        assert_eq!(actual_lines[1], ">                   ");
    }

    #[test]
    fn render_applies_styles() {
        let input = Input::new("Title");
        let mut state = InputState::new();
        state.insert_char('x');
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&input, area, &mut buf, &mut state);

        // Title should have secondary color + bold
        let title_cell = &buf[ratatui::layout::Position::new(0, 0)];
        let p = Palette::dark();
        assert_eq!(title_cell.fg, p.secondary);
        assert!(title_cell.modifier.contains(Modifier::BOLD));

        // Prompt should have primary color
        let prompt_cell = &buf[ratatui::layout::Position::new(0, 1)];
        assert_eq!(prompt_cell.fg, p.primary);

        // Text should have foreground color
        let text_cell = &buf[ratatui::layout::Position::new(2, 1)];
        assert_eq!(text_cell.fg, p.foreground);
    }

    #[test]
    fn render_cursor_when_focused() {
        let input = Input::new("T");
        let mut state = InputState::new();
        state.set_focused(true);
        state.insert_char('a');
        // Cursor should be at position after 'a'
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&input, area, &mut buf, &mut state);

        // Cursor position: prompt "> " = 2 chars, then 'a' = 1 char, cursor at x=3
        let cursor_cell = &buf[ratatui::layout::Position::new(3, 1)];
        let styles = InputStyles::dark();
        assert_eq!(cursor_cell.bg, styles.cursor.bg.unwrap());
    }
}
