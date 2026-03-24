# Widget Implementation Patterns

This is the reference for Phase 3 of the create-widget skill. It covers the full implementation pattern for a ratatui-cheese widget.

**Visual fidelity is the top priority.** Every widget must look identical to its Bubbles/Bubbletea counterpart — same spacing, same box-drawing characters, same alignment, same behavior. Run the original Go example and match it cell-for-cell. "Close enough" is not enough.

## Widget Struct

Every widget follows this structure:

```rust
use ratatui::style::Style;
use ratatui::widgets::Block;

/// One-line description of the widget.
///
/// A longer explanation of what the widget does, how it's used,
/// and any important behavior notes.
///
/// # Example
///
/// rust
/// use cheese_<name>::<WidgetName>;
/// use ratatui::widgets::Block;
///
/// let widget = <WidgetName>::new(/* args */)
///     .block(Block::bordered().title("Title"))
///     .style(Style::default().fg(Color::Green));
///
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct MyWidget<'a> {
    /// Optional block wrapping the widget (border, title, padding).
    block: Option<Block<'a>>,
    /// Base style applied to the entire widget area.
    style: Style,
    // ... widget-specific fields
}
```

**Key rules:**

- Use lifetime `'a` when the struct borrows data (Block, Text, Line, Span all borrow)
- Always include `block` and `style` — they're the standard composability points in ratatui
- Derive `Debug`, `Default`, `Clone`, `Eq`, `PartialEq`, `Hash` — ratatui's own widgets derive all of these. Only omit one if a field doesn't support it.

## Constructor and Builder Methods

```rust
impl<'a> MyWidget<'a> {
    pub fn new(/* required args */) -> Self {
        Self {
            block: None,
            style: Style::default(),
            // ... defaults for widget-specific fields
        }
    }

    /// Wraps the widget in a block (border, title, padding).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the base style for the widget area.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
}
```

**Key rules:**

- `#[must_use]` on every builder method — prevents silently discarding the modified value
- Use `const fn` where possible (when the body is const-compatible, e.g., simple field assignment without trait calls). Ratatui uses `pub const fn` for methods like `wrap()`, `scroll()`, `alignment()`.
- Accept `Into<T>` for flexibility (e.g., `Into<Style>`, `Into<Text<'a>>`, `Into<Line<'a>>`). Note: `Into<T>` is not const-compatible, so methods accepting generics can't be `const fn`.
- Return `Self` for chaining

## Widget Trait Implementation

Implement on both the owned type and a reference. The owned version delegates to the reference version:

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

impl Widget for MyWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &MyWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clip to buffer bounds
        let area = area.intersection(buf.area);

        // Apply base style
        buf.set_style(area, self.style);

        // Render optional block (border/title)
        self.block.as_ref().render(area, buf);

        // Get the inner area (inside the block, if any)
        let inner = self.block.inner_if_some(area);

        // Render widget content
        self.render_content(inner, buf);
    }
}
```

## Rendering Content

Split the actual rendering into a private method:

```rust
impl MyWidget<'_> {
    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        // Your rendering logic here. Common buffer operations:

        // Set text at a position (returns next position after the text)
        buf.set_string(area.x, area.y, "hello", self.style);

        // Set a styled Line
        let line = Line::from(vec![
            Span::styled("key: ", Style::default().bold()),
            Span::raw("value"),
        ]);
        buf.set_line(area.x, area.y, &line, area.width);

        // Set individual cells
        buf[(area.x, area.y)].set_symbol("█").set_style(self.style);

        // Fill an area
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_symbol("·");
            }
        }
    }
}
```

## Styled Trait

Implement `Styled` so the widget works with ratatui's `.style()` and `.set_style()` ecosystem:

```rust
use ratatui::style::Styled;

impl Styled for MyWidget<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}
```

## StatefulWidget (When Needed)

Use `StatefulWidget` when the widget has mutable state that persists across frames — selection index, scroll offset, cursor position, animation frame, etc.

The full delegation chain (matching ratatui's List widget pattern):

```rust
use ratatui::widgets::StatefulWidget;

pub struct MyWidget<'a> { /* config/appearance fields */ }

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct MyWidgetState {
    pub selected: Option<usize>,
    pub offset: usize,
    // ... mutable state
}

// 1. Widget for owned — delegates to Widget for &ref
impl Widget for MyWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

// 2. Widget for &ref — delegates to StatefulWidget with a default state
//    This lets users render without state when they don't need it.
impl Widget for &MyWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = MyWidgetState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// 3. StatefulWidget for owned — delegates to StatefulWidget for &ref
impl StatefulWidget for MyWidget<'_> {
    type State = MyWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

// 4. StatefulWidget for &ref — the REAL render logic lives here
impl StatefulWidget for &MyWidget<'_> {
    type State = MyWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);
        self.block.as_ref().render(area, buf);
        let inner = self.block.inner_if_some(area);

        if inner.is_empty() {
            return;
        }

        // Rendering logic that reads/writes `state`
    }
}
```

**Separation of concerns:** The widget struct holds _appearance configuration_ (style, block, labels). The state struct holds _runtime data_ (selection, scroll, cursor). This lets you recreate the widget each frame with fresh config while state persists.

**Why four impls?** This gives users maximum flexibility — they can render with `frame.render_widget(&widget, area)` for simple cases or `frame.render_stateful_widget(&widget, area, &mut state)` when they need persistent state. All paths funnel to a single render implementation.

## File Organization

For simple widgets, everything can live in `lib.rs`. For complex widgets, split:

```
src/
├── lib.rs          # Re-exports: pub mod widget; pub use widget::MyWidget;
├── widget.rs       # Widget struct, builder, Widget/Styled impls
└── state.rs        # State struct (if StatefulWidget)
```

## Imports Cheat Sheet

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Styled, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BlockExt, Padding, StatefulWidget, Widget};
```

## Consulting Reference Code

When implementing, check these reference locations:

- **Ratatui built-in widgets:** `.ref/code/ratatui/ratatui-widgets/src/` — look at `paragraph.rs`, `tabs.rs`, `list/` for proven patterns
- **Ratatui examples:** `.ref/code/ratatui/examples/` — for rendering and layout techniques
- **Bubbles (Go):** `.ref/code/bubbles/` — for the component's behavior and API design inspiration (translate the Go patterns to Rust idioms)
