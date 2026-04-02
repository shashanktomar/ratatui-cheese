//! Shared color palette for consistent widget theming.
//!
//! The [`Palette`] struct provides semantic color names that all widget style
//! structs can derive from via `from_palette()`. This gives a single place to
//! define your color vocabulary while letting each widget map those colors to
//! its own style fields.
//!
//! # Example
//!
//! ```rust
//! use ratatui::style::Color;
//! use ratatui_cheese::theme::Palette;
//! use ratatui_cheese::tree::TreeStyles;
//!
//! // Use a preset
//! let styles = TreeStyles::from_palette(&Palette::dark());
//!
//! // Or customize
//! let mut p = Palette::dark();
//! p.primary = Color::Indexed(69); // blue accent
//! let styles = TreeStyles::from_palette(&p);
//! ```

use ratatui::style::Color;

/// A semantic color palette for theming widgets.
///
/// Inspired by shadcn/ui, Material Design, and Catppuccin. Each field represents
/// a role, not a specific color. Widgets map these roles to their internal
/// style fields via `from_palette()`.
///
/// | Role        | Purpose                                          |
/// |-------------|--------------------------------------------------|
/// | `foreground`| Primary text                                     |
/// | `muted`     | Secondary text (child items, counts, keys)       |
/// | `faint`     | Tertiary text (help descriptions, separators)    |
/// | `primary`   | Main accent (selection, active indicators)       |
/// | `secondary` | Second accent (headings, filenames, labels)      |
/// | `surface`   | Raised background (selected row, hover)          |
/// | `border`    | Borders, dividers                                |
/// | `highlight` | Bold background (title bars, badges)             |
/// | `on_highlight`| Text on `highlight` background                 |
/// | `error`       | Error, failure, destructive actions               |
/// | `success`     | Success, confirmation, positive outcomes          |
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Palette {
    /// Primary text color.
    pub foreground: Color,
    /// Secondary, de-emphasized text (child items, counts, keys).
    pub muted: Color,
    /// Tertiary, most receded text (help descriptions, separators, disabled elements).
    pub faint: Color,
    /// Main accent color (selection, active indicators).
    pub primary: Color,
    /// Second accent color (headings, filenames, labels).
    pub secondary: Color,
    /// Raised background color (selected row, hover state).
    pub surface: Color,
    /// Borders, dividers.
    pub border: Color,
    /// Bold background color (title bars, badges, active states).
    pub highlight: Color,
    /// Text color for content on `highlight` background.
    pub on_highlight: Color,
    /// Error, failure, destructive actions.
    pub error: Color,
    /// Success, confirmation, positive outcomes.
    pub success: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self::dark()
    }
}

impl Palette {
    /// Creates a palette for dark backgrounds.
    #[must_use]
    pub fn dark() -> Self {
        Self {
            foreground: Color::Indexed(252),
            muted: Color::Indexed(245),
            faint: Color::Indexed(239),
            primary: Color::Indexed(212),
            secondary: Color::Indexed(141),
            surface: Color::Indexed(237),
            border: Color::Indexed(238),
            highlight: Color::Indexed(141),
            on_highlight: Color::Indexed(235),
            error: Color::Indexed(203),
            success: Color::Indexed(114),
        }
    }

    /// Creates a palette for light backgrounds.
    #[must_use]
    pub fn light() -> Self {
        Self {
            foreground: Color::Indexed(235),
            muted: Color::Indexed(243),
            faint: Color::Indexed(248),
            primary: Color::Indexed(162),
            secondary: Color::Indexed(97),
            surface: Color::Indexed(254),
            border: Color::Indexed(250),
            highlight: Color::Indexed(97),
            on_highlight: Color::Indexed(255),
            error: Color::Indexed(160),
            success: Color::Indexed(28),
        }
    }

    /// Charm-inspired pink and purple on dark.
    ///
    /// Based on Charmbracelet's lipgloss examples.
    #[must_use]
    pub fn charm() -> Self {
        Self {
            foreground: Color::Rgb(0xFA, 0xFA, 0xFA),
            muted: Color::Indexed(250),
            faint: Color::Indexed(240),
            primary: Color::Rgb(0xEE, 0x6F, 0xF8),
            secondary: Color::Rgb(0x7D, 0x56, 0xF4),
            surface: Color::Indexed(237),
            border: Color::Indexed(238),
            highlight: Color::Rgb(0x7D, 0x56, 0xF4),
            on_highlight: Color::Rgb(0xFA, 0xFA, 0xFA),
            error: Color::Rgb(0xFF, 0x55, 0x55),
            success: Color::Rgb(0x3D, 0xD6, 0x8C),
        }
    }

    /// Cool ocean blues and teals on dark.
    #[must_use]
    pub fn ocean() -> Self {
        Self {
            foreground: Color::Rgb(0xE0, 0xE0, 0xE0),
            muted: Color::Indexed(246),
            faint: Color::Indexed(240),
            primary: Color::Rgb(0x00, 0xD7, 0x87),
            secondary: Color::Rgb(0x5F, 0xAF, 0xFF),
            surface: Color::Indexed(236),
            border: Color::Indexed(238),
            highlight: Color::Rgb(0x5F, 0xAF, 0xFF),
            on_highlight: Color::Indexed(235),
            error: Color::Rgb(0xFF, 0x6B, 0x6B),
            success: Color::Rgb(0x00, 0xD7, 0x87),
        }
    }

    /// Warm sunset oranges and golds on dark.
    #[must_use]
    pub fn sunset() -> Self {
        Self {
            foreground: Color::Rgb(0xFA, 0xF0, 0xE6),
            muted: Color::Indexed(248),
            faint: Color::Indexed(242),
            primary: Color::Rgb(0xFF, 0x87, 0x5F),
            secondary: Color::Rgb(0xFF, 0xD7, 0x00),
            surface: Color::Indexed(236),
            border: Color::Indexed(239),
            highlight: Color::Rgb(0xFF, 0x87, 0x5F),
            on_highlight: Color::Indexed(235),
            error: Color::Rgb(0xFF, 0x5F, 0x5F),
            success: Color::Rgb(0x87, 0xD7, 0x5F),
        }
    }

    /// Returns all built-in palette presets with their names.
    #[must_use]
    pub fn presets() -> Vec<(&'static str, Self)> {
        vec![
            ("Dark", Self::dark()),
            ("Light", Self::light()),
            ("Charm", Self::charm()),
            ("Ocean", Self::ocean()),
            ("Sunset", Self::sunset()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_palette() {
        let p = Palette::dark();
        assert_eq!(p.primary, Color::Indexed(212));
        assert_eq!(p.secondary, Color::Indexed(141));
    }

    #[test]
    fn light_palette() {
        let p = Palette::light();
        assert_eq!(p.primary, Color::Indexed(162));
        assert_eq!(p.secondary, Color::Indexed(97));
    }

    #[test]
    fn default_is_dark() {
        assert_eq!(Palette::default(), Palette::dark());
    }

    #[test]
    fn customizable() {
        let mut p = Palette::dark();
        p.primary = Color::Indexed(69);
        assert_eq!(p.primary, Color::Indexed(69));
        assert_eq!(p.foreground, Palette::dark().foreground);
    }

    #[test]
    fn presets_has_all() {
        let presets = Palette::presets();
        assert_eq!(presets.len(), 5);
        assert_eq!(presets[0].0, "Dark");
        assert_eq!(presets[2].0, "Charm");
    }
}
