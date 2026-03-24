# Widget Testing Patterns

This is the reference for Phase 4 of the create-widget skill. It covers how to test ratatui-cheese widgets using buffer-based snapshot testing.

## Core Idea

Ratatui widgets render into a `Buffer`. You create a buffer of a known size, render the widget into it, and compare against an expected buffer. This gives you pixel-perfect (well, cell-perfect) verification of rendering output.

## Test Helper

Start every widget's test module with this helper:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style, Stylize};
    use ratatui::widgets::{Block, Widget};

    /// Renders a widget into a buffer and compares against expected output.
    #[track_caller]
    fn assert_renders(widget: &impl Widget, expected: &Buffer) {
        let area = expected.area;
        let mut actual = Buffer::empty(Rect::new(0, 0, area.width, area.height));
        widget.render(actual.area, &mut actual);
        assert_eq!(actual, *expected);
    }
}
```

`#[track_caller]` makes assertion failures point to the test that called the helper, not the helper itself.

## Creating Expected Buffers

**From strings** — simplest approach, one string per row:

```rust
let expected = Buffer::with_lines(["Hello     ", "World     "]);
```

Each string becomes a row. All strings must be the same width (pad with spaces). The buffer dimensions are inferred from the strings.

**Empty buffer** — for testing that nothing renders:

```rust
let expected = Buffer::empty(Rect::new(0, 0, 10, 3));
```

**With styles** — set styles on specific cells after creating:

```rust
let mut expected = Buffer::with_lines(["Hello"]);
// Style the first 5 cells
for x in 0..5 {
    expected[(x, 0)].set_style(Style::default().fg(Color::Green));
}
```

## Minimum Test Coverage

Every widget should have at least these tests:

### 1. Empty area renders nothing

```rust
#[test]
fn render_empty_area() {
    let widget = MyWidget::new(/* args */);
    let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
    widget.render(buf.area, &mut buf);
    // No panic = pass
}
```

### 2. Basic render

```rust
#[test]
fn render_basic() {
    let widget = MyWidget::new(/* args */);
    let expected = Buffer::with_lines([
        "expected output line 1",
        "expected output line 2",
    ]);
    assert_renders(&widget, &expected);
}
```

### 3. Render with block

```rust
#[test]
fn render_with_block() {
    let widget = MyWidget::new(/* args */)
        .block(Block::bordered().title("Title"));
    let expected = Buffer::with_lines([
        "┌Title───┐",
        "│content │",
        "└────────┘",
    ]);
    assert_renders(&widget, &expected);
}
```

### 4. Render with style

```rust
#[test]
fn render_with_style() {
    let widget = MyWidget::new(/* args */)
        .style(Style::default().fg(Color::Red));
    // Verify the style is applied to the rendered cells
    let area = Rect::new(0, 0, 10, 1);
    let mut buf = Buffer::empty(area);
    widget.render(area, &mut buf);
    assert_eq!(buf[(0, 0)].fg, Color::Red);
}
```

### 5. Widget-specific behavior

Add tests for each unique behavior: truncation, wrapping, selection state, overflow, alignment, etc.

## Testing StatefulWidget

```rust
#[test]
fn render_stateful() {
    let widget = MyWidget::new(/* args */);
    let mut state = MyWidgetState::default();
    let area = Rect::new(0, 0, 20, 5);
    let mut buf = Buffer::empty(area);

    // Render with initial state
    StatefulWidget::render(&widget, area, &mut buf, &mut state);
    // Assert initial render...

    // Modify state and re-render
    state.selected = Some(1);
    let mut buf = Buffer::empty(area);
    StatefulWidget::render(&widget, area, &mut buf, &mut state);
    // Assert updated render...
}
```

## Running Tests

```bash
just t           # runs cargo nextest run
cargo test       # also works
```

## Tips

- Buffer comparison failures print a nice diff showing which cells differ
- Use `Stylize` trait for concise style builders in tests: `"text".green().bold()`
- Test edge cases: area smaller than content, area width of 1, very long text
- If testing unicode, remember that wide characters (CJK, emoji) take 2 cells
- Ratatui's own tests use `rstest` for fixtures and parameterized tests — consider adding it as a dev-dependency if you need those features, but plain `#[test]` is fine for most cases
- Ratatui names its test helper `test_case` — we use `assert_renders` for clarity
