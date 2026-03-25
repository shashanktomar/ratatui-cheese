//! Bubbletea-inspired help widget.
//!
//! Renders keyboard shortcut help text with two modes: a compact single-line view
//! and an expanded multi-column view. Matches the behavior and visual output of
//! Charmbracelet's Bubbles help component.
//!
//! # Example
//!
//! ```rust
//! use ratatui_cheese::help::{Binding, Help, KeyMap};
//!
//! struct MyKeyMap;
//!
//! impl KeyMap for MyKeyMap {
//!     fn short_help(&self) -> Vec<Binding> {
//!         vec![
//!             Binding::new("?", "toggle help"),
//!             Binding::new("q", "quit"),
//!         ]
//!     }
//!
//!     fn full_help(&self) -> Vec<Vec<Binding>> {
//!         vec![
//!             vec![Binding::new("↑/k", "up"), Binding::new("↓/j", "down")],
//!             vec![Binding::new("?", "help"), Binding::new("q", "quit")],
//!         ]
//!     }
//! }
//!
//! let help = Help::new(&MyKeyMap);
//! ```

use crate::utils::display_width;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

/// A key binding for display in the help widget.
///
/// Contains the display text for the key and its description, plus an enabled flag
/// to control visibility. Disabled bindings are skipped during rendering.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Binding {
    key: String,
    description: String,
    enabled: bool,
}

impl Binding {
    /// Creates a new enabled binding with the given key and description.
    #[must_use]
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            enabled: true,
        }
    }

    /// Sets whether this binding is enabled (visible in help).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Returns the key display text.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the description text.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns whether this binding is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Trait for types that provide key bindings for the help widget.
///
/// Implement this on your key map struct to define which bindings appear
/// in short (compact) and full (expanded) help views.
pub trait KeyMap {
    /// Returns bindings for the compact single-line help view.
    fn short_help(&self) -> Vec<Binding>;

    /// Returns grouped bindings for the expanded multi-column help view.
    /// Each inner `Vec` becomes a column.
    fn full_help(&self) -> Vec<Vec<Binding>>;
}

/// Styles for the help widget, split by short and full mode.
///
/// Each mode has separate styles for keys, descriptions, and separators,
/// plus a shared ellipsis style.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct HelpStyles {
    /// Style for the truncation ellipsis.
    pub ellipsis: Style,
    /// Style for key text in short help mode.
    pub short_key: Style,
    /// Style for description text in short help mode.
    pub short_desc: Style,
    /// Style for the separator between items in short help mode.
    pub short_separator: Style,
    /// Style for key text in full help mode.
    pub full_key: Style,
    /// Style for description text in full help mode.
    pub full_desc: Style,
    /// Style for the separator between columns in full help mode.
    pub full_separator: Style,
}

impl Default for HelpStyles {
    fn default() -> Self {
        Self::dark()
    }
}

impl HelpStyles {
    /// Creates styles for dark backgrounds.
    ///
    /// Matches the default dark theme from Charmbracelet's Bubbles help component.
    #[must_use]
    pub fn dark() -> Self {
        // Go source: lightDark(lightColor, darkColor) — dark is the second arg
        let key_style = Style::default().fg(Color::Rgb(0x62, 0x62, 0x62));
        let desc_style = Style::default().fg(Color::Rgb(0x4A, 0x4A, 0x4A));
        let sep_style = Style::default().fg(Color::Rgb(0x3C, 0x3C, 0x3C));

        Self {
            ellipsis: sep_style,
            short_key: key_style,
            short_desc: desc_style,
            short_separator: sep_style,
            full_key: key_style,
            full_desc: desc_style,
            full_separator: sep_style,
        }
    }

    /// Creates styles for light backgrounds.
    ///
    /// Matches the light theme from Charmbracelet's Bubbles help component.
    #[must_use]
    pub fn light() -> Self {
        let key_style = Style::default().fg(Color::Rgb(0x90, 0x90, 0x90));
        let desc_style = Style::default().fg(Color::Rgb(0xB2, 0xB2, 0xB2));
        let sep_style = Style::default().fg(Color::Rgb(0xDA, 0xDA, 0xDA));

        Self {
            ellipsis: sep_style,
            short_key: key_style,
            short_desc: desc_style,
            short_separator: sep_style,
            full_key: key_style,
            full_desc: desc_style,
            full_separator: sep_style,
        }
    }
}

/// A help widget that renders keyboard shortcut bindings.
///
/// Supports two display modes: a compact single-line view (`show_all = false`)
/// and an expanded multi-column view (`show_all = true`). Bindings are provided
/// via the [`KeyMap`] trait at construction time.
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::help::{Binding, Help, KeyMap};
///
/// struct Keys;
/// impl KeyMap for Keys {
///     fn short_help(&self) -> Vec<Binding> {
///         vec![Binding::new("?", "help"), Binding::new("q", "quit")]
///     }
///     fn full_help(&self) -> Vec<Vec<Binding>> {
///         vec![vec![Binding::new("?", "help"), Binding::new("q", "quit")]]
///     }
/// }
///
/// let help = Help::new(&Keys).show_all(true);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Help {
    show_all: bool,
    short_separator: String,
    full_separator: String,
    ellipsis: String,
    styles: HelpStyles,
    bindings: Vec<Binding>,
    binding_groups: Vec<Vec<Binding>>,
}

impl Default for Help {
    fn default() -> Self {
        Self {
            show_all: false,
            short_separator: " • ".into(),
            full_separator: "    ".into(),
            ellipsis: "…".into(),
            styles: HelpStyles::default(),
            bindings: Vec::new(),
            binding_groups: Vec::new(),
        }
    }
}

impl Help {
    /// Creates a help widget from a [`KeyMap`] implementation.
    ///
    /// Calls `short_help()` and `full_help()` on the key map to populate bindings.
    #[must_use]
    pub fn new(keymap: &impl KeyMap) -> Self {
        Self {
            bindings: keymap.short_help(),
            binding_groups: keymap.full_help(),
            ..Default::default()
        }
    }

    /// Sets whether to show the full expanded help view.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn show_all(mut self, show_all: bool) -> Self {
        self.show_all = show_all;
        self
    }

    /// Sets the separator string between items in short help mode.
    ///
    /// Default: `" • "`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn short_separator(mut self, sep: impl Into<String>) -> Self {
        self.short_separator = sep.into();
        self
    }

    /// Sets the separator string between columns in full help mode.
    ///
    /// Default: `"    "` (four spaces)
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn full_separator(mut self, sep: impl Into<String>) -> Self {
        self.full_separator = sep.into();
        self
    }

    /// Sets the truncation ellipsis string.
    ///
    /// Default: `"…"`
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn ellipsis(mut self, ellipsis: impl Into<String>) -> Self {
        self.ellipsis = ellipsis.into();
        self
    }

    /// Sets the styles for the help widget.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn styles(mut self, styles: HelpStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Sets the bindings for short help mode directly.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn bindings(mut self, bindings: Vec<Binding>) -> Self {
        self.bindings = bindings;
        self
    }

    /// Sets the binding groups for full help mode directly.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn binding_groups(mut self, groups: Vec<Vec<Binding>>) -> Self {
        self.binding_groups = groups;
        self
    }

    /// Renders the short (single-line) help view into the buffer.
    fn render_short_help(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || self.bindings.is_empty() {
            return;
        }

        let max_width = area.width as usize;
        let mut x = area.x;
        let y = area.y;
        let mut total_width: usize = 0;
        let mut first = true;

        for binding in &self.bindings {
            if !binding.is_enabled() {
                continue;
            }

            // Calculate the display width of separator + "key desc"
            let sep_w = if first { 0 } else { display_width(&self.short_separator) };
            let item_w =
                sep_w + display_width(binding.key()) + 1 + display_width(binding.description());

            // Check if this item fits
            if max_width > 0 && total_width + item_w > max_width {
                // Try to fit ellipsis
                let tail_w = 1 + display_width(&self.ellipsis);
                if total_width + tail_w < max_width {
                    buf.set_string(x, y, " ", Style::default());
                    buf.set_string(x + 1, y, &self.ellipsis, self.styles.ellipsis);
                }
                break;
            }

            // Render separator
            if !first {
                buf.set_string(x, y, &self.short_separator, self.styles.short_separator);
                x += display_width(&self.short_separator) as u16;
            }

            // Render key
            buf.set_string(x, y, binding.key(), self.styles.short_key);
            x += display_width(binding.key()) as u16;

            // Space between key and description
            buf.set_string(x, y, " ", self.styles.short_desc);
            x += 1;

            // Render description
            buf.set_string(x, y, binding.description(), self.styles.short_desc);
            x += display_width(binding.description()) as u16;

            total_width = (x - area.x) as usize;
            first = false;
        }
    }

    /// Renders the full (multi-column) help view into the buffer.
    fn render_full_help(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || self.binding_groups.is_empty() {
            return;
        }

        let max_width = area.width as usize;
        let mut col_x = area.x;
        let mut total_width: usize = 0;
        let mut first_col = true;

        for group in &self.binding_groups {
            // Skip groups with no enabled bindings
            if !group.iter().any(Binding::is_enabled) {
                continue;
            }

            // Collect enabled bindings
            let enabled: Vec<&Binding> = group.iter().filter(|b| b.is_enabled()).collect();
            if enabled.is_empty() {
                continue;
            }

            // Calculate max key display width for alignment within this column
            let max_key_w = enabled
                .iter()
                .map(|b| display_width(b.key()))
                .max()
                .unwrap_or(0);

            // Calculate total column display width: separator + max_key + space + max_desc
            let max_desc_w = enabled
                .iter()
                .map(|b| display_width(b.description()))
                .max()
                .unwrap_or(0);
            let sep_w = if first_col { 0 } else { display_width(&self.full_separator) };
            let col_w = sep_w + max_key_w + 1 + max_desc_w;

            // Check if column fits
            if max_width > 0 && total_width + col_w > max_width {
                // Try to fit ellipsis
                let tail_w = 1 + display_width(&self.ellipsis);
                if total_width + tail_w < max_width {
                    buf.set_string(col_x, area.y, " ", Style::default());
                    buf.set_string(col_x + 1, area.y, &self.ellipsis, self.styles.ellipsis);
                }
                break;
            }

            // Render separator
            if !first_col {
                let sep_display_w = display_width(&self.full_separator) as u16;
                for row in 0..enabled.len().min(area.height as usize) {
                    buf.set_string(
                        col_x,
                        area.y + row as u16,
                        &self.full_separator,
                        self.styles.full_separator,
                    );
                }
                col_x += sep_display_w;
            }

            // Render each binding in the column
            for (row, binding) in enabled.iter().enumerate() {
                if row >= area.height as usize {
                    break;
                }
                let y = area.y + row as u16;

                // Render key
                buf.set_string(col_x, y, binding.key(), self.styles.full_key);

                // Space after key (padded to max_key_w)
                let key_end = col_x + max_key_w as u16;
                buf.set_string(key_end, y, " ", self.styles.full_desc);

                // Render description
                buf.set_string(key_end + 1, y, binding.description(), self.styles.full_desc);
            }

            col_x += (max_key_w + 1 + max_desc_w) as u16;
            total_width += col_w;
            first_col = false;
        }
    }
}

impl Widget for Help {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Help {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        if self.show_all {
            self.render_full_help(area, buf);
        } else {
            self.render_short_help(area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    /// Renders a widget into a buffer and compares against expected output.
    #[track_caller]
    fn assert_renders(widget: &Help, expected: &Buffer) {
        let area = expected.area;
        let mut actual = Buffer::empty(Rect::new(0, 0, area.width, area.height));
        Widget::render(widget, actual.area, &mut actual);
        assert_eq!(actual, *expected);
    }

    /// Creates a Help widget with unstyled defaults for content-only testing.
    fn unstyled() -> HelpStyles {
        HelpStyles {
            ellipsis: Style::default(),
            short_key: Style::default(),
            short_desc: Style::default(),
            short_separator: Style::default(),
            full_key: Style::default(),
            full_desc: Style::default(),
            full_separator: Style::default(),
        }
    }

    fn test_bindings() -> Vec<Binding> {
        vec![
            Binding::new("↑/k", "move up"),
            Binding::new("↓/j", "move down"),
            Binding::new("?", "toggle help"),
            Binding::new("q", "quit"),
        ]
    }

    fn test_binding_groups() -> Vec<Vec<Binding>> {
        vec![
            vec![
                Binding::new("↑/k", "move up"),
                Binding::new("↓/j", "move down"),
            ],
            vec![Binding::new("?", "toggle help"), Binding::new("q", "quit")],
        ]
    }

    struct TestKeyMap;

    impl KeyMap for TestKeyMap {
        fn short_help(&self) -> Vec<Binding> {
            test_bindings()
        }

        fn full_help(&self) -> Vec<Vec<Binding>> {
            test_binding_groups()
        }
    }

    // === Binding tests ===

    #[test]
    fn binding_new() {
        let b = Binding::new("q", "quit");
        assert_eq!(b.key(), "q");
        assert_eq!(b.description(), "quit");
        assert!(b.is_enabled());
    }

    #[test]
    fn binding_disabled() {
        let b = Binding::new("q", "quit").enabled(false);
        assert!(!b.is_enabled());
    }

    // === Help construction tests ===

    #[test]
    fn new_from_keymap() {
        let help = Help::new(&TestKeyMap);
        assert_eq!(help.bindings.len(), 4);
        assert_eq!(help.binding_groups.len(), 2);
        assert!(!help.show_all);
    }

    #[test]
    fn builder_methods() {
        let help = Help::new(&TestKeyMap)
            .show_all(true)
            .short_separator(" | ")
            .full_separator("  ")
            .ellipsis("...");
        assert!(help.show_all);
        assert_eq!(help.short_separator, " | ");
        assert_eq!(help.full_separator, "  ");
        assert_eq!(help.ellipsis, "...");
    }

    // === Short help rendering tests ===

    #[test]
    fn render_empty_area() {
        let help = Help::new(&TestKeyMap);
        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        help.render(buf.area, &mut buf);
    }

    #[test]
    fn render_empty_bindings() {
        let help = Help::default();
        let expected = Buffer::with_lines(["          "]);
        assert_renders(&help, &expected);
    }

    #[test]
    fn render_short_help_single_binding() {
        let help = Help::default()
            .styles(unstyled())
            .bindings(vec![Binding::new("q", "quit")]);
        let expected = Buffer::with_lines(["q quit     "]);
        assert_renders(&help, &expected);
    }

    #[test]
    fn render_short_help_multiple_bindings() {
        let help = Help::default()
            .styles(unstyled())
            .bindings(vec![Binding::new("?", "help"), Binding::new("q", "quit")]);
        let expected = Buffer::with_lines(["? help • q quit     "]);
        assert_renders(&help, &expected);
    }

    #[test]
    fn render_short_help_skips_disabled() {
        let help = Help::default().styles(unstyled()).bindings(vec![
            Binding::new("?", "help"),
            Binding::new("x", "hidden").enabled(false),
            Binding::new("q", "quit"),
        ]);
        let expected = Buffer::with_lines(["? help • q quit     "]);
        assert_renders(&help, &expected);
    }

    #[test]
    fn render_short_help_truncation() {
        let help = Help::default().styles(unstyled()).bindings(vec![
            Binding::new("?", "help"),
            Binding::new("q", "quit"),
            Binding::new("x", "extra long binding that wont fit"),
        ]);
        // Width 19: "? help • q quit" is 15, next item won't fit, " …" = 2 chars
        let expected = Buffer::with_lines(["? help • q quit …  "]);
        assert_renders(&help, &expected);
    }

    // === Full help rendering tests ===

    #[test]
    fn render_full_help_single_column() {
        let help = Help::default()
            .styles(unstyled())
            .show_all(true)
            .binding_groups(vec![vec![
                Binding::new("?", "help"),
                Binding::new("q", "quit"),
            ]]);
        let expected = Buffer::with_lines(["? help     ", "q quit     "]);
        assert_renders(&help, &expected);
    }

    #[test]
    fn render_full_help_multiple_columns() {
        let help = Help::default()
            .styles(unstyled())
            .show_all(true)
            .binding_groups(vec![
                vec![Binding::new("↑/k", "up"), Binding::new("↓/j", "down")],
                vec![Binding::new("?", "help"), Binding::new("q", "quit")],
            ]);
        // Col1: max_key=3 ("↑/k"), max_desc=4 ("down") → col=8
        // Sep: "    " = 4, col2 starts at 12
        // Col2: max_key=1, max_desc=4 → col=6
        // Total: 8 + 4 + 6 = 18
        let expected = Buffer::with_lines(["↑/k up      ? help", "↓/j down    q quit"]);
        assert_renders(&help, &expected);
    }

    #[test]
    fn render_full_help_skips_disabled() {
        let help = Help::default()
            .styles(unstyled())
            .show_all(true)
            .binding_groups(vec![vec![
                Binding::new("?", "help"),
                Binding::new("x", "hidden").enabled(false),
                Binding::new("q", "quit"),
            ]]);
        let expected = Buffer::with_lines(["? help     ", "q quit     "]);
        assert_renders(&help, &expected);
    }

    #[test]
    fn render_full_help_empty_groups() {
        let help = Help::default().styles(unstyled()).show_all(true);
        let expected = Buffer::with_lines(["          "]);
        assert_renders(&help, &expected);
    }

    #[test]
    fn render_full_help_skips_all_disabled_group() {
        let help = Help::default()
            .styles(unstyled())
            .show_all(true)
            .binding_groups(vec![
                vec![Binding::new("x", "hidden").enabled(false)],
                vec![Binding::new("?", "help"), Binding::new("q", "quit")],
            ]);
        let expected = Buffer::with_lines(["? help     ", "q quit     "]);
        assert_renders(&help, &expected);
    }

    // === Styles tests ===

    #[test]
    fn dark_styles_created() {
        let styles = HelpStyles::dark();
        assert_eq!(styles.short_key.fg, Some(Color::Rgb(0x62, 0x62, 0x62)));
        assert_eq!(styles.short_desc.fg, Some(Color::Rgb(0x4A, 0x4A, 0x4A)));
        assert_eq!(
            styles.short_separator.fg,
            Some(Color::Rgb(0x3C, 0x3C, 0x3C))
        );
    }

    #[test]
    fn light_styles_created() {
        let styles = HelpStyles::light();
        assert_eq!(styles.short_key.fg, Some(Color::Rgb(0x90, 0x90, 0x90)));
        assert_eq!(styles.short_desc.fg, Some(Color::Rgb(0xB2, 0xB2, 0xB2)));
        assert_eq!(
            styles.short_separator.fg,
            Some(Color::Rgb(0xDA, 0xDA, 0xDA))
        );
    }

    #[test]
    fn render_short_help_with_styles() {
        let help = Help::default()
            .styles(HelpStyles::dark())
            .bindings(vec![Binding::new("q", "quit")]);
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        help.render(area, &mut buf);
        assert_eq!(buf[(0, 0)].fg, Color::Rgb(0x62, 0x62, 0x62)); // key style
        assert_eq!(buf[(2, 0)].fg, Color::Rgb(0x4A, 0x4A, 0x4A)); // desc style
    }
}
