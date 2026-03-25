use unicode_width::UnicodeWidthStr;

/// Returns the display width of a string in terminal cells.
pub(crate) fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}
