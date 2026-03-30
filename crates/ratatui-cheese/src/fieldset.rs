//! A container widget with decorated horizontal rule lines.
//!
//! Renders top and bottom rule lines with optional title text and a repeating
//! fill character. The fill extends to the available width, creating a visual
//! section boundary. Child content is rendered in the area between the two
//! lines via [`Fieldset::inner`].
//!
//! # Example
//!
//! ```rust
//! use ratatui::layout::Alignment;
//! use ratatui_cheese::fieldset::{Fieldset, FieldsetFill};
//!
//! let fieldset = Fieldset::new()
//!     .title("Section Title")
//!     .title_bottom("End")
//!     .fill(FieldsetFill::Slash)
//!     .top_alignment(Alignment::Left)
//!     .bottom_alignment(Alignment::Center);
//!
//! // Use fieldset.inner(area) to get the content area between the rules.
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Styled};
use ratatui::widgets::Widget;

use crate::theme::Palette;
use crate::utils::{display_width, truncate_with_ellipsis};

// ---------------------------------------------------------------------------
// Fill
// ---------------------------------------------------------------------------

/// The fill character(s) used in the rule lines.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub enum FieldsetFill {
    /// Forward slash: `/`
    #[default]
    Slash,
    /// Box-drawing dash: `─`
    Dash,
    /// Bullet dot: `•`
    Dot,
    /// Double line: `═`
    Double,
    /// Thick line: `━`
    Thick,
    /// Star: `✦`
    Star,
    /// User-provided character(s), repeated to fill the available width.
    Custom(String),
}

impl FieldsetFill {
    /// Returns the fill pattern string for this variant.
    fn pattern(&self) -> &str {
        match self {
            Self::Slash => "/",
            Self::Dash => "─",
            Self::Dot => "•",
            Self::Double => "═",
            Self::Thick => "━",
            Self::Star => "✦",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Returns the display name for this fill variant.
    pub fn name(&self) -> &str {
        match self {
            Self::Slash => "Slash",
            Self::Dash => "Dash",
            Self::Dot => "Dot",
            Self::Double => "Double",
            Self::Thick => "Thick",
            Self::Star => "Star",
            Self::Custom(_) => "Custom",
        }
    }
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

/// Styles for the fieldset elements.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FieldsetStyles {
    /// Style applied to title text (top and bottom).
    pub title: Style,
    /// Style applied to fill characters.
    pub rule: Style,
}

impl Default for FieldsetStyles {
    fn default() -> Self {
        Self::dark()
    }
}

impl FieldsetStyles {
    /// Creates styles from a [`Palette`].
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            title: Style::default()
                .fg(p.secondary)
                .add_modifier(ratatui::style::Modifier::BOLD),
            rule: Style::default().fg(p.border),
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

/// A container widget that renders decorated top/bottom rule lines with optional titles.
///
/// The fieldset draws a top and bottom horizontal line filled with a repeating
/// character pattern. Either line can carry a title that is aligned left, center,
/// or right. Use [`Fieldset::inner`] to obtain the content area between the rules,
/// then render child widgets into that area.
///
/// # Example
///
/// ```rust
/// use ratatui::layout::Alignment;
/// use ratatui_cheese::fieldset::{Fieldset, FieldsetFill, FieldsetStyles};
/// use ratatui_cheese::theme::Palette;
///
/// let fieldset = Fieldset::new()
///     .title("My Section")
///     .fill(FieldsetFill::Dash)
///     .styles(FieldsetStyles::from_palette(&Palette::charm()));
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Fieldset<'a> {
    title_top: Option<&'a str>,
    title_bottom: Option<&'a str>,
    top_alignment: Alignment,
    bottom_alignment: Alignment,
    fill: FieldsetFill,
    styles: FieldsetStyles,
}

impl Default for Fieldset<'_> {
    fn default() -> Self {
        Self {
            title_top: None,
            title_bottom: None,
            top_alignment: Alignment::Left,
            bottom_alignment: Alignment::Left,
            fill: FieldsetFill::default(),
            styles: FieldsetStyles::default(),
        }
    }
}

impl<'a> Fieldset<'a> {
    /// Creates a new fieldset with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the top title text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn title(mut self, title: &'a str) -> Self {
        self.title_top = Some(title);
        self
    }

    /// Sets the bottom title text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn title_bottom(mut self, title: &'a str) -> Self {
        self.title_bottom = Some(title);
        self
    }

    /// Sets the alignment for the top rule line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn top_alignment(mut self, alignment: Alignment) -> Self {
        self.top_alignment = alignment;
        self
    }

    /// Sets the alignment for the bottom rule line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn bottom_alignment(mut self, alignment: Alignment) -> Self {
        self.bottom_alignment = alignment;
        self
    }

    /// Sets the fill character pattern.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn fill(mut self, fill: FieldsetFill) -> Self {
        self.fill = fill;
        self
    }

    /// Sets the title style.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn title_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.styles.title = style.into();
        self
    }

    /// Sets the rule (fill character) style.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn rule_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.styles.rule = style.into();
        self
    }

    /// Sets all styles at once.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn styles(mut self, styles: FieldsetStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Convenience: sets styles from a palette.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn palette(mut self, palette: &Palette) -> Self {
        self.styles = FieldsetStyles::from_palette(palette);
        self
    }

    /// Returns the inner area available for content between the top and bottom rules.
    #[must_use]
    pub fn inner(&self, area: Rect) -> Rect {
        if area.height < 2 {
            return Rect::new(area.x, area.y, area.width, 0);
        }
        Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(2),
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

impl Fieldset<'_> {
    /// Renders a single rule line (top or bottom) into the buffer at the given y position.
    fn render_rule_line(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        width: u16,
        title: Option<&str>,
        alignment: Alignment,
    ) {
        if width == 0 {
            return;
        }

        let w = width as usize;

        match title.filter(|t| !t.is_empty()) {
            None => {
                // Fill-only line
                let fill_str = build_fill(self.fill.pattern(), w);
                buf.set_string(x, y, &fill_str, self.styles.rule);
            }
            Some(title_text) => {
                let title_w = display_width(title_text);

                if title_w >= w {
                    // Title fills the whole line — truncate, no fill
                    let truncated = truncate_with_ellipsis(title_text, w, "…");
                    buf.set_string(x, y, &truncated, self.styles.title);
                    return;
                }

                // We need at least 1 char gap + 1 char fill on each side of the title
                // that has fill. For left/right, fill is only on one side.
                let gap = 1usize;

                match alignment {
                    Alignment::Left => {
                        // [title] [gap] [fill...]
                        let fill_width = w.saturating_sub(title_w + gap);
                        buf.set_string(x, y, title_text, self.styles.title);
                        if fill_width > 0 {
                            let fill_str = build_fill(self.fill.pattern(), fill_width);
                            let fill_x = x + (title_w + gap) as u16;
                            buf.set_string(fill_x, y, &fill_str, self.styles.rule);
                        }
                    }
                    Alignment::Right => {
                        // [fill...] [gap] [title]
                        let fill_width = w.saturating_sub(title_w + gap);
                        if fill_width > 0 {
                            let fill_str = build_fill(self.fill.pattern(), fill_width);
                            buf.set_string(x, y, &fill_str, self.styles.rule);
                        }
                        let title_x = x + w as u16 - title_w as u16;
                        buf.set_string(title_x, y, title_text, self.styles.title);
                    }
                    Alignment::Center => {
                        // [fill...] [gap] [title] [gap] [fill...]
                        let side_budget = w.saturating_sub(title_w + gap * 2);
                        let left_fill = side_budget / 2;
                        let right_fill = side_budget - left_fill;

                        if left_fill > 0 {
                            let fill_str = build_fill(self.fill.pattern(), left_fill);
                            buf.set_string(x, y, &fill_str, self.styles.rule);
                        }

                        let title_x = x + (left_fill + gap) as u16;
                        buf.set_string(title_x, y, title_text, self.styles.title);

                        if right_fill > 0 {
                            let fill_str = build_fill(self.fill.pattern(), right_fill);
                            let right_x = title_x + title_w as u16 + gap as u16;
                            buf.set_string(right_x, y, &fill_str, self.styles.rule);
                        }
                    }
                }
            }
        }
    }
}

/// Builds a fill string by repeating the pattern to fill the given width.
fn build_fill(pattern: &str, width: usize) -> String {
    if pattern.is_empty() || width == 0 {
        return String::new();
    }

    let pattern_w = display_width(pattern);
    if pattern_w == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut current_w = 0;

    while current_w < width {
        for ch in pattern.chars() {
            let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if current_w + cw > width {
                break;
            }
            result.push(ch);
            current_w += cw;
        }
        // If the pattern is wider than remaining space, we already broke out
        if current_w + pattern_w > width && current_w < width {
            // Fill remaining with first chars that fit
            for ch in pattern.chars() {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if current_w + cw > width {
                    break;
                }
                result.push(ch);
                current_w += cw;
            }
            break;
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Widget trait impls
// ---------------------------------------------------------------------------

impl Widget for Fieldset<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Fieldset<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);

        if area.is_empty() {
            return;
        }

        // Render top rule line
        self.render_rule_line(
            buf,
            area.x,
            area.y,
            area.width,
            self.title_top,
            self.top_alignment,
        );

        // Render bottom rule line (if there's room)
        if area.height >= 2 {
            let bottom_y = area.y + area.height - 1;
            self.render_rule_line(
                buf,
                area.x,
                bottom_y,
                area.width,
                self.title_bottom,
                self.bottom_alignment,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Styled trait
// ---------------------------------------------------------------------------

impl Styled for Fieldset<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.styles.rule
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.rule_style(style)
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
    use ratatui::style::{Color, Modifier, Style};

    /// Renders a fieldset into a buffer and compares against expected output.
    #[track_caller]
    fn assert_renders(widget: &Fieldset, expected: &Buffer) {
        let area = expected.area;
        let mut actual = Buffer::empty(Rect::new(0, 0, area.width, area.height));
        Widget::render(widget, actual.area, &mut actual);
        assert_eq!(actual, *expected);
    }

    #[test]
    fn render_empty_area() {
        let widget = Fieldset::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        widget.render(buf.area, &mut buf);
        // No panic = pass
    }

    #[test]
    fn render_fill_only_no_title() {
        let widget = Fieldset::new()
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["//////////", "          ", "          ", "//////////"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_left_aligned_title() {
        let widget = Fieldset::new()
            .title("Hi")
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["Hi ///////", "          ", "          ", "//////////"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_right_aligned_title() {
        let widget = Fieldset::new()
            .title("Hi")
            .top_alignment(Alignment::Right)
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["/////// Hi", "          ", "          ", "//////////"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_center_aligned_title() {
        let widget = Fieldset::new()
            .title("Hi")
            .top_alignment(Alignment::Center)
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["/// Hi ///", "          ", "          ", "//////////"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_bottom_title() {
        let widget = Fieldset::new()
            .title("Top")
            .title_bottom("Bot")
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["Top //////", "          ", "          ", "Bot //////"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_independent_alignments() {
        let widget = Fieldset::new()
            .title("Top")
            .title_bottom("Bot")
            .top_alignment(Alignment::Left)
            .bottom_alignment(Alignment::Right)
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["Top //////", "          ", "          ", "////// Bot"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_dash_fill() {
        let widget = Fieldset::new()
            .fill(FieldsetFill::Dash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["──────────", "          ", "──────────"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_double_fill() {
        let widget = Fieldset::new()
            .fill(FieldsetFill::Double)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["══════════", "          ", "══════════"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_custom_fill() {
        let widget = Fieldset::new()
            .fill(FieldsetFill::Custom("/\\".to_string()))
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["/\\/\\/\\/\\/\\", "          ", "/\\/\\/\\/\\/\\"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_custom_fill_with_title() {
        let widget = Fieldset::new()
            .title("Hi")
            .fill(FieldsetFill::Custom("/\\".to_string()))
            .title_style(Style::default())
            .rule_style(Style::default());
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);
        Widget::render(&widget, area, &mut buf);
        // Top line: "Hi " + fill
        let top_line: String = (0..10)
            .map(|x| buf[(x, 0)].symbol().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(top_line.starts_with("Hi "));
    }

    #[test]
    fn title_truncation_when_too_wide() {
        let widget = Fieldset::new()
            .title("Hello World")
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let area = Rect::new(0, 0, 5, 3);
        let mut buf = Buffer::empty(area);
        Widget::render(&widget, area, &mut buf);
        // Title "Hello World" (11 chars) should be truncated to fit in 5 cells
        let top_line: String = (0..5)
            .map(|x| buf[(x, 0)].symbol().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert_eq!(display_width(&top_line), 5);
    }

    #[test]
    fn inner_area_calculation() {
        let widget = Fieldset::new();
        let area = Rect::new(5, 10, 20, 8);
        let inner = widget.inner(area);
        assert_eq!(inner, Rect::new(5, 11, 20, 6));
    }

    #[test]
    fn inner_area_too_small() {
        let widget = Fieldset::new();
        let area = Rect::new(0, 0, 10, 1);
        let inner = widget.inner(area);
        assert_eq!(inner.height, 0);
    }

    #[test]
    fn inner_area_exactly_two_rows() {
        let widget = Fieldset::new();
        let area = Rect::new(0, 0, 10, 2);
        let inner = widget.inner(area);
        assert_eq!(inner, Rect::new(0, 1, 10, 0));
    }

    #[test]
    fn render_height_one_only_top_line() {
        let widget = Fieldset::new()
            .title("Hi")
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let expected = Buffer::with_lines(["Hi ///////"]);
        assert_renders(&widget, &expected);
    }

    #[test]
    fn render_with_styles() {
        let widget = Fieldset::new()
            .title("Hi")
            .fill(FieldsetFill::Slash)
            .title_style(Style::default().fg(Color::Green))
            .rule_style(Style::default().fg(Color::Red));
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);
        Widget::render(&widget, area, &mut buf);

        // Title cells should be green
        assert_eq!(buf[(0, 0)].fg, Color::Green);
        assert_eq!(buf[(1, 0)].fg, Color::Green);
        // Fill cells should be red
        assert_eq!(buf[(3, 0)].fg, Color::Red);
    }

    #[test]
    fn from_palette_styles() {
        let palette = Palette::dark();
        let styles = FieldsetStyles::from_palette(&palette);
        assert_eq!(styles.title.fg, Some(palette.secondary));
        assert!(styles.title.add_modifier.contains(Modifier::BOLD));
        assert_eq!(styles.rule.fg, Some(palette.border));
    }

    #[test]
    fn default_fill_is_slash() {
        assert_eq!(FieldsetFill::default(), FieldsetFill::Slash);
    }

    #[test]
    fn fill_names() {
        assert_eq!(FieldsetFill::Slash.name(), "Slash");
        assert_eq!(FieldsetFill::Dash.name(), "Dash");
        assert_eq!(FieldsetFill::Custom("x".to_string()).name(), "Custom");
    }

    #[test]
    fn build_fill_repeats_pattern() {
        assert_eq!(build_fill("/", 5), "/////");
        assert_eq!(build_fill("/\\", 6), "/\\/\\/\\");
        assert_eq!(build_fill("/\\", 5), "/\\/\\/");
    }

    #[test]
    fn build_fill_empty() {
        assert_eq!(build_fill("", 5), "");
        assert_eq!(build_fill("/", 0), "");
    }

    #[test]
    fn render_width_one() {
        let widget = Fieldset::new()
            .title("Hi")
            .fill(FieldsetFill::Slash)
            .title_style(Style::default())
            .rule_style(Style::default());
        let area = Rect::new(0, 0, 1, 3);
        let mut buf = Buffer::empty(area);
        Widget::render(&widget, area, &mut buf);
        // Should not panic, title gets truncated
    }

    #[test]
    fn styled_trait() {
        let widget = Fieldset::new().rule_style(Style::default().fg(Color::Red));
        assert_eq!(widget.style().fg, Some(Color::Red));
    }
}
