//! Bubbletea-inspired spinner widget.
//!
//! A spinner cycles through frames of characters to indicate loading or progress.
//! Includes all 12 preset spinner types from Charmbracelet's Bubbles library with
//! identical frame sequences.
//!
//! # Example
//!
//! ```rust
//! use std::time::Duration;
//! use ratatui_cheese::spinner::{Spinner, SpinnerState, SpinnerType};
//! use ratatui::style::{Color, Style};
//!
//! let spinner = Spinner::default()
//!     .style(Style::default().fg(Color::Blue));
//!
//! let mut state = SpinnerState::new(SpinnerType::Dot);
//!
//! // In your event loop, pass elapsed time — state handles frame timing:
//! state.tick(Duration::from_millis(16));
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
            Self::Dot => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
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

/// A spinner widget that renders the current frame from [`SpinnerState`].
///
/// `Spinner` is purely an appearance configuration — style and rendering.
/// The spinner type, frames, and animation timing live in [`SpinnerState`].
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::spinner::Spinner;
/// use ratatui::style::{Color, Style};
///
/// let spinner = Spinner::default()
///     .style(Style::default().fg(Color::Blue));
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Spinner {
    style: Style,
}

impl Spinner {
    /// Sets the style for the spinner frame.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
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
/// Owns the spinner type, frames, and an internal time accumulator.
/// Pass elapsed time to [`tick()`](SpinnerState::tick) and the state
/// advances frames automatically at the correct interval.
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use ratatui_cheese::spinner::{SpinnerState, SpinnerType};
///
/// let mut state = SpinnerState::new(SpinnerType::Dot);
/// state.tick(Duration::from_millis(200)); // advances frames as needed
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SpinnerState {
    spinner_type: SpinnerType,
    custom_frames: Option<Vec<&'static str>>,
    custom_interval: Option<Duration>,
    frame: usize,
    elapsed: Duration,
}

impl SpinnerState {
    /// Creates a new state with the given spinner preset.
    pub fn new(spinner_type: SpinnerType) -> Self {
        Self {
            spinner_type,
            custom_frames: None,
            custom_interval: None,
            frame: 0,
            elapsed: Duration::ZERO,
        }
    }

    /// Creates a state with custom frames and interval.
    pub fn custom(frames: Vec<&'static str>, interval: Duration) -> Self {
        Self {
            spinner_type: SpinnerType::Line,
            custom_frames: Some(frames),
            custom_interval: Some(interval),
            frame: 0,
            elapsed: Duration::ZERO,
        }
    }

    /// Advances the spinner by the given elapsed time.
    ///
    /// Accumulates time internally and advances frames when the interval
    /// is reached. Handles multiple frame advances if `dt` is large.
    pub fn tick(&mut self, dt: Duration) {
        let frame_count = self.frames().len();
        if frame_count == 0 {
            return;
        }

        let interval = self.interval();
        if interval.is_zero() {
            return;
        }

        self.elapsed += dt;
        while self.elapsed >= interval {
            self.elapsed -= interval;
            self.frame = (self.frame + 1) % frame_count;
        }
    }

    /// Returns the current frame index.
    pub fn frame(&self) -> usize {
        self.frame
    }

    /// Returns the current frame string.
    pub fn frame_str(&self) -> &str {
        let frames = self.frames();
        if frames.is_empty() {
            return "";
        }
        frames[self.frame % frames.len()]
    }

    /// Returns the frames this spinner uses.
    pub fn frames(&self) -> &[&'static str] {
        match &self.custom_frames {
            Some(frames) => frames,
            None => self.spinner_type.frames(),
        }
    }

    /// Returns the interval between frames.
    pub fn interval(&self) -> Duration {
        match self.custom_interval {
            Some(interval) => interval,
            None => self.spinner_type.interval(),
        }
    }
}

impl Default for SpinnerState {
    /// Defaults to `SpinnerType::Line`.
    fn default() -> Self {
        Self::new(SpinnerType::default())
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
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        let frame = state.frame_str();
        if frame.is_empty() {
            return;
        }

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
    fn default_state_is_line() {
        let state = SpinnerState::default();
        assert_eq!(state.frames(), SpinnerType::Line.frames());
        assert_eq!(state.interval(), SpinnerType::Line.interval());
    }

    #[test]
    fn new_with_type() {
        let state = SpinnerState::new(SpinnerType::Dot);
        assert_eq!(state.frames(), SpinnerType::Dot.frames());
        assert_eq!(state.interval(), SpinnerType::Dot.interval());
    }

    #[test]
    fn custom_frames_and_interval() {
        let state = SpinnerState::custom(vec!["a", "b", "c"], Duration::from_millis(200));
        assert_eq!(state.frames(), &["a", "b", "c"]);
        assert_eq!(state.interval(), Duration::from_millis(200));
    }

    #[test]
    fn style_builder() {
        let style = Style::default().fg(Color::Red);
        let spinner = Spinner::default().style(style);
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
            &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]
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
    fn tick_advances_frame_after_interval() {
        let mut state = SpinnerState::new(SpinnerType::Line); // 100ms interval
        assert_eq!(state.frame(), 0);

        // Not enough time — no advance
        state.tick(Duration::from_millis(50));
        assert_eq!(state.frame(), 0);

        // Now enough — advances
        state.tick(Duration::from_millis(50));
        assert_eq!(state.frame(), 1);
    }

    #[test]
    fn tick_skips_frames_on_large_dt() {
        let mut state = SpinnerState::new(SpinnerType::Line); // 100ms, 4 frames
        state.tick(Duration::from_millis(250));
        assert_eq!(state.frame(), 2); // skipped 0→1→2
    }

    #[test]
    fn tick_wraps_around() {
        let mut state = SpinnerState::new(SpinnerType::Line); // 4 frames
        state.tick(Duration::from_millis(400));
        assert_eq!(state.frame(), 0); // wrapped
    }

    #[test]
    fn tick_accumulates_remainder() {
        let mut state = SpinnerState::new(SpinnerType::Line); // 100ms
        state.tick(Duration::from_millis(150)); // frame 1, 50ms remainder
        assert_eq!(state.frame(), 1);

        state.tick(Duration::from_millis(60)); // 50+60=110ms >= 100ms → frame 2
        assert_eq!(state.frame(), 2);
    }

    #[test]
    fn frame_str_returns_current() {
        let mut state = SpinnerState::new(SpinnerType::Line);
        assert_eq!(state.frame_str(), "|");
        state.tick(Duration::from_millis(100));
        assert_eq!(state.frame_str(), "/");
    }

    #[test]
    fn tick_empty_frames_is_noop() {
        let mut state = SpinnerState::custom(vec![], Duration::from_millis(100));
        state.tick(Duration::from_millis(200));
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
        let spinner = Spinner::default();
        let mut state = SpinnerState::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        StatefulWidget::render(&spinner, buf.area, &mut buf, &mut state);
        // No panic = pass
    }

    #[test]
    fn render_first_frame() {
        let spinner = Spinner::default();
        let mut state = SpinnerState::new(SpinnerType::Line);
        let expected = Buffer::with_lines(["|  "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_second_frame_after_tick() {
        let spinner = Spinner::default();
        let mut state = SpinnerState::new(SpinnerType::Line);
        state.tick(Duration::from_millis(100));
        let expected = Buffer::with_lines(["/  "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_with_style() {
        let style = Style::default().fg(Color::Red);
        let spinner = Spinner::default().style(style);
        let mut state = SpinnerState::new(SpinnerType::Pulse);
        let area = Rect::new(0, 0, 3, 1);
        let mut buf = Buffer::empty(area);
        StatefulWidget::render(&spinner, area, &mut buf, &mut state);
        assert_eq!(buf[(0, 0)].fg, Color::Red);
        assert_eq!(buf[(0, 0)].symbol(), "█");
    }

    #[test]
    fn render_stateless_shows_frame_zero() {
        let spinner = Spinner::default();
        let expected = Buffer::with_lines(["|  "]);
        let mut actual = Buffer::empty(Rect::new(0, 0, 3, 1));
        Widget::render(&spinner, actual.area, &mut actual);
        assert_eq!(actual, expected);
    }

    #[test]
    fn render_dot_braille() {
        let spinner = Spinner::default();
        let mut state = SpinnerState::new(SpinnerType::Dot);
        let expected = Buffer::with_lines(["⣾ "]); // braille char + trailing space from buffer
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_points_multichar() {
        let spinner = Spinner::default();
        let mut state = SpinnerState::new(SpinnerType::Points);
        let expected = Buffer::with_lines(["∙∙∙  "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_ellipsis_empty_first_frame() {
        let spinner = Spinner::default();
        let mut state = SpinnerState::new(SpinnerType::Ellipsis);
        let expected = Buffer::with_lines(["   "]);
        assert_renders_stateful(&spinner, &mut state, &expected);
    }

    #[test]
    fn render_cycles_all_line_frames() {
        let spinner = Spinner::default();
        let mut state = SpinnerState::new(SpinnerType::Line);
        let frames = ["|", "/", "-", "\\"];
        let interval = SpinnerType::Line.interval();
        for expected_frame in &frames {
            let area = Rect::new(0, 0, 3, 1);
            let mut buf = Buffer::empty(area);
            StatefulWidget::render(&spinner, area, &mut buf, &mut state);
            assert_eq!(buf[(0, 0)].symbol(), *expected_frame);
            state.tick(interval);
        }
    }

    #[test]
    fn render_clips_when_area_row_is_outside_buffer() {
        let spinner = Spinner::default();
        let mut state = SpinnerState::new(SpinnerType::Line);
        let area = Rect::new(0, 1, 3, 1);
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 1));
        StatefulWidget::render(&spinner, area, &mut buf, &mut state);
        assert_eq!(buf, Buffer::with_lines(["   "]));
    }
}
