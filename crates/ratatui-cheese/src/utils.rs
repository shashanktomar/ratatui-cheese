use unicode_width::UnicodeWidthStr;

/// Returns the display width of a string in terminal cells.
pub(crate) fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Takes characters from `text` up to `max_width` terminal cells.
pub(crate) fn take_width(text: &str, max_width: usize) -> String {
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

/// Truncates `text` to fit within `max_width` terminal cells, appending
/// `ellipsis` if truncation occurs.
pub(crate) fn truncate_with_ellipsis(text: &str, max_width: usize, ellipsis: &str) -> String {
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
