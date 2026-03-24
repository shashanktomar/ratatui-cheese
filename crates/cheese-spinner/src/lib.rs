//! Bubbletea-inspired spinner widget for Ratatui.
//!
//! A spinner cycles through frames of characters to indicate loading or progress.
//! Includes all 12 preset spinner types from Charmbracelet's Bubbles library with
//! identical frame sequences.
//!
//! # Example
//!
//! ```rust
//! use cheese_spinner::{Spinner, SpinnerState, SpinnerType};
//! use ratatui::style::{Color, Style};
//!
//! let spinner = Spinner::new(SpinnerType::Dot)
//!     .style(Style::default().fg(Color::Blue));
//!
//! let mut state = SpinnerState::default();
//! // In your event loop, call state.tick() at spinner.spinner_type().interval()
//! ```

use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Styled};
use ratatui::widgets::{StatefulWidget, Widget};

/// Preset spinner types matching Charmbracelet's Bubbles spinner variants.
///
/// Each variant defines a sequence of frames and an interval between them.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SpinnerType {
    Line,
    Dot,
    MiniDot,
    Jump,
    Pulse,
    Points,
    Globe,
    Moon,
    Monkey,
    Meter,
    Hamburger,
    Ellipsis,
}

impl SpinnerType {
    /// Returns the frame strings for this spinner type.
    ///
    /// These are identical to the frame sequences in Charmbracelet's Bubbles.
    pub fn frames(&self) -> &'static [&'static str] {
        match self {
            Self::Line => &["|", "/", "-", "\\"],
            Self::Dot => &["⣾ ", "⣽ ", "⣻ ", "⢿ ", "⡿ ", "⣟ ", "⣯ ", "⣷ "],
            Self::MiniDot => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            Self::Jump => &["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"],
            Self::Pulse => &["█", "▓", "▒", "░"],
            Self::Points => &["∙∙∙", "●∙∙", "∙●∙", "∙∙●"],
            Self::Globe => &["🌍", "🌎", "🌏"],
            Self::Moon => &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
            Self::Monkey => &["🙈", "🙉", "🙊"],
            Self::Meter => &["▱▱▱", "▰▱▱", "▰▰▱", "▰▰▰", "▰▰▱", "▰▱▱", "▱▱▱"],
            Self::Hamburger => &["☱", "☲", "☴", "☲"],
            Self::Ellipsis => &["", ".", "..", "..."],
        }
    }

    /// Returns the interval between frames for this spinner type.
    ///
    /// Matches the FPS values from Charmbracelet's Bubbles (FPS = time.Second / N).
    pub fn interval(&self) -> Duration {
        match self {
            Self::Line => Duration::from_millis(100),      // 10 FPS
            Self::Dot => Duration::from_millis(100),       // 10 FPS
            Self::MiniDot => Duration::from_millis(83),    // 12 FPS
            Self::Jump => Duration::from_millis(100),      // 10 FPS
            Self::Pulse => Duration::from_millis(125),     // 8 FPS
            Self::Points => Duration::from_millis(143),    // 7 FPS
            Self::Globe => Duration::from_millis(250),     // 4 FPS
            Self::Moon => Duration::from_millis(125),      // 8 FPS
            Self::Monkey => Duration::from_millis(333),    // 3 FPS
            Self::Meter => Duration::from_millis(143),     // 7 FPS
            Self::Hamburger => Duration::from_millis(333), // 3 FPS
            Self::Ellipsis => Duration::from_millis(333),  // 3 FPS
        }
    }
}

impl Default for SpinnerType {
    /// Defaults to `Line`, matching Bubbles' default.
    fn default() -> Self {
        Self::Line
    }
}

/// A spinner widget that cycles through frames to indicate activity.
///
/// The spinner is configured with a [`SpinnerType`] preset (or custom frames) and a
/// [`Style`]. It renders the current frame from [`SpinnerState`] into the buffer.
///
/// # Example
///
/// ```rust
/// use cheese_spinner::{Spinner, SpinnerType};
/// use ratatui::style::{Color, Style};
///
/// let spinner = Spinner::new(SpinnerType::Dot)
///     .style(Style::default().fg(Color::Blue));
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Spinner {
    spinner_type: SpinnerType,
    custom_frames: Option<Vec<&'static str>>,
    style: Style,
}

impl Spinner {
    /// Creates a new spinner with the given preset type.
    pub fn new(spinner_type: SpinnerType) -> Self {
        Self {
            spinner_type,
            custom_frames: None,
            style: Style::default(),
        }
    }

    /// Creates a spinner with custom frames.
    ///
    /// Use this when none of the presets fit. The interval defaults to 100ms;
    /// the caller controls tick timing.
    pub fn custom(frames: Vec<&'static str>) -> Self {
        Self {
            spinner_type: SpinnerType::Line, // unused when custom_frames is Some
            custom_frames: Some(frames),
            style: Style::default(),
        }
    }

    /// Sets the style for the spinner frame.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Returns the spinner type.
    pub fn spinner_type(&self) -> SpinnerType {
        self.spinner_type
    }

    /// Returns the frames this spinner uses.
    pub fn frames(&self) -> &[&'static str] {
        match &self.custom_frames {
            Some(frames) => frames,
            None => self.spinner_type.frames(),
        }
    }
}

impl Styled for Spinner {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

/// Mutable state for a [`Spinner`] widget.
///
/// Tracks the current frame index. Call [`tick()`](SpinnerState::tick) to advance
/// to the next frame — typically on a timer matching [`SpinnerType::interval()`].
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct SpinnerState {
    frame: usize,
}

impl SpinnerState {
    /// Returns the current frame index.
    pub fn frame(&self) -> usize {
        self.frame
    }

    /// Advances to the next frame, wrapping around at the end.
    ///
    /// `frame_count` is the total number of frames in the spinner.
    /// Typically: `state.tick(spinner.frames().len())`
    pub fn tick(&mut self, frame_count: usize) {
        if frame_count == 0 {
            return;
        }
        self.frame = (self.frame + 1) % frame_count;
    }
}

// Widget for owned — delegates to &ref
impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

// Widget for &ref — renders frame 0 (stateless fallback)
impl Widget for &Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = SpinnerState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// StatefulWidget for owned — delegates to &ref
impl StatefulWidget for Spinner {
    type State = SpinnerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

// StatefulWidget for &ref — the real render logic
impl StatefulWidget for &Spinner {
    type State = SpinnerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.is_empty() {
            return;
        }

        let frames = self.frames();
        if frames.is_empty() {
            return;
        }

        let frame_idx = state.frame % frames.len();
        let frame = frames[frame_idx];

        buf.set_string(area.x, area.y, frame, self.style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

    // ---- Constructor & presets ----

    #[test]
    fn default_spinner_is_line() {
        let spinner = Spinner::default();
        assert_eq!(spinner.spinner_type(), SpinnerType::Line);
    }

    #[test]
    fn new_with_type() {
        let spinner = Spinner::new(SpinnerType::Dot);
        assert_eq!(spinner.spinner_type(), SpinnerType::Dot);
    }

    #[test]
    fn custom_frames() {
        let spinner = Spinner::custom(vec!["a", "b", "c"]);
        assert_eq!(spinner.frames(), &["a", "b", "c"]);
    }

    #[test]
    fn style_builder() {
        let style = Style::default().fg(Color::Red);
        let spinner = Spinner::new(SpinnerType::Line).style(style);
        assert_eq!(Styled::style(&spinner), style);
    }

    // ---- Preset frame sequences ----

    #[test]
    fn preset_line_frames() {
        assert_eq!(SpinnerType::Line.frames(), &["|", "/", "-", "\\"]);
    }

    #[test]
    fn preset_dot_frames() {
        assert_eq!(
            SpinnerType::Dot.frames(),
            &["⣾ ", "⣽ ", "⣻ ", "⢿ ", "⡿ ", "⣟ ", "⣯ ", "⣷ "]
        );
    }

    #[test]
    fn preset_minidot_frames() {
        assert_eq!(
            SpinnerType::MiniDot.frames(),
            &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
        );
    }

    #[test]
    fn preset_jump_frames() {
        assert_eq!(
            SpinnerType::Jump.frames(),
            &["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"]
        );
    }

    #[test]
    fn preset_pulse_frames() {
        assert_eq!(SpinnerType::Pulse.frames(), &["█", "▓", "▒", "░"]);
    }

    #[test]
    fn preset_points_frames() {
        assert_eq!(SpinnerType::Points.frames(), &["∙∙∙", "●∙∙", "∙●∙", "∙∙●"]);
    }

    #[test]
    fn preset_globe_frames() {
        assert_eq!(SpinnerType::Globe.frames(), &["🌍", "🌎", "🌏"]);
    }

    #[test]
    fn preset_moon_frames() {
        assert_eq!(
            SpinnerType::Moon.frames(),
            &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"]
        );
    }

    #[test]
    fn preset_monkey_frames() {
        assert_eq!(SpinnerType::Monkey.frames(), &["🙈", "🙉", "🙊"]);
    }

    #[test]
    fn preset_meter_frames() {
        assert_eq!(
            SpinnerType::Meter.frames(),
            &["▱▱▱", "▰▱▱", "▰▰▱", "▰▰▰", "▰▰▱", "▰▱▱", "▱▱▱"]
        );
    }

    #[test]
    fn preset_hamburger_frames() {
        assert_eq!(SpinnerType::Hamburger.frames(), &["☱", "☲", "☴", "☲"]);
    }

    #[test]
    fn preset_ellipsis_frames() {
        assert_eq!(SpinnerType::Ellipsis.frames(), &["", ".", "..", "..."]);
    }

    // ---- Intervals ----

    #[test]
    fn preset_intervals() {
        assert_eq!(SpinnerType::Line.interval(), Duration::from_millis(100));
        assert_eq!(SpinnerType::Dot.interval(), Duration::from_millis(100));
        assert_eq!(SpinnerType::MiniDot.interval(), Duration::from_millis(83));
        assert_eq!(SpinnerType::Globe.interval(), Duration::from_millis(250));
        assert_eq!(SpinnerType::Monkey.interval(), Duration::from_millis(333));
    }

    // ---- State & tick ----

    #[test]
    fn tick_advances_frame() {
        let mut state = SpinnerState::default();
        assert_eq!(state.frame(), 0);
        state.tick(4);
        assert_eq!(state.frame(), 1);
        state.tick(4);
        assert_eq!(state.frame(), 2);
    }

    #[test]
    fn tick_wraps_around() {
        let mut state = SpinnerState::default();
        for _ in 0..4 {
            state.tick(4);
        }
        assert_eq!(state.frame(), 0);
    }

    #[test]
    fn tick_zero_frame_count_is_noop() {
        let mut state = SpinnerState::default();
        state.tick(0);
        assert_eq!(state.frame(), 0);
    }

    // ---- Rendering ----

    #[track_caller]
    fn assert_renders_stateful(spinner: &Spinner, state: &mut SpinnerState, expected: &Buffer) {
        let area = expected.area;
        let mut actual = Buffer::empty(Rect::new(0, 0, area.width, area.height));
        StatefulWidget::render(spinner, actual.area, &mut actual, state);
        assert_eq!(actual, *expected);
    }

    #[test]
    fn render_empty_area() {
        let spinner = Spinner::new(SpinnerType::Line);
        let mut state = SpinnerState::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        StatefulWidget::render(&spinner, buf.area, &mut buf, &mut state);
        // No panic = pass
    }

    #[test]
    fn render_first_frame() {
        let spinner = Spinner::new(SpinnerType::Line);
        let mut state = SpinnerState::default();
        let expected = Buffer::with_lines(["|  "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_second_frame_after_tick() {
        let spinner = Spinner::new(SpinnerType::Line);
        let mut state = SpinnerState::default();
        state.tick(spinner.frames().len());
        let expected = Buffer::with_lines(["/  "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_with_style() {
        let style = Style::default().fg(Color::Red);
        let spinner = Spinner::new(SpinnerType::Pulse).style(style);
        let mut state = SpinnerState::default();
        let area = Rect::new(0, 0, 3, 1);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&spinner, area, &mut buf, &mut state);
        assert_eq!(buf[(0, 0)].fg, Color::Red);
        assert_eq!(buf[(0, 0)].symbol(), "█");
    }

    #[test]
    fn render_stateless_shows_frame_zero() {
        let spinner = Spinner::new(SpinnerType::Line);
        let expected = Buffer::with_lines(["|  "]);
        let mut actual = Buffer::empty(Rect::new(0, 0, 3, 1));
        Widget::render(&spinner, actual.area, &mut actual);
        assert_eq!(actual, expected);
    }

    #[test]
    fn render_dot_braille() {
        let spinner = Spinner::new(SpinnerType::Dot);
        let mut state = SpinnerState::default();
        let expected = Buffer::with_lines(["⣾ "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_points_multichar() {
        let spinner = Spinner::new(SpinnerType::Points);
        let mut state = SpinnerState::default();
        let expected = Buffer::with_lines(["∙∙∙  "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_ellipsis_empty_first_frame() {
        let spinner = Spinner::new(SpinnerType::Ellipsis);
        let mut state = SpinnerState::default();
        // First frame is empty string — buffer should remain blank
        let expected = Buffer::with_lines(["   "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_cycles_all_line_frames() {
        let spinner = Spinner::new(SpinnerType::Line);
        let mut state = SpinnerState::default();
        let frames = ["|", "/", "-", "\\"];
        for expected_frame in &frames {
            let area = Rect::new(0, 0, 3, 1);
            let mut buf = Buffer::empty(area);
            StatefulWidget::render(&spinner, area, &mut buf, &mut state);
            assert_eq!(buf[(0, 0)].symbol(), *expected_frame);
            state.tick(spinner.frames().len());
        }
    }
}
