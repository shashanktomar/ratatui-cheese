# Fieldset Widget Design

## Context

The project needs a new widget inspired by a UI pattern seen in Charm's bubbletea demos: a section container with decorated horizontal rule lines (top and bottom) that can carry a title/label. The fill character repeats to the edge of the available width. This is not available in bubbles and is a novel addition to ratatui-cheese.

The widget wraps child content between two ruled lines, acting as a visual grouping mechanism — similar to HTML's `<fieldset>` element.

## Design

### Data Model

```rust
/// Preset fill styles for the ruled lines.
pub enum FieldsetFill {
    Slash,           // /
    Dash,            // ─
    Dot,             // ·
    Double,          // ═
    Thick,           // ━
    Star,            // ✦
    Custom(String),  // user-provided character(s), repeated to fill
}

/// Styles for the fieldset elements.
pub struct FieldsetStyles {
    pub title: Style,  // style applied to title text (top and bottom)
    pub rule: Style,   // style applied to fill characters
}

/// A container widget that renders decorated top/bottom rule lines with optional titles.
pub struct Fieldset<'a> {
    title_top: Option<Line<'a>>,
    title_bottom: Option<Line<'a>>,
    top_alignment: Alignment,      // default: Left
    bottom_alignment: Alignment,   // default: Left (independent from top)
    fill: FieldsetFill,
    styles: FieldsetStyles,
}
```

### Builder API

```rust
Fieldset::new()
    .title("Section Title")            // sets title_top
    .title_bottom("End")               // sets title_bottom (default: None)
    .top_alignment(Alignment::Left)    // default: Left
    .bottom_alignment(Alignment::Center) // independent from top
    .fill(FieldsetFill::Slash)         // default: Slash
    .title_style(Style::new().green().bold())
    .rule_style(Style::new().magenta())
    .styles(FieldsetStyles::from_palette(&palette))  // bulk set from palette
    .palette(&Palette::dark())         // convenience for .styles(FieldsetStyles::from_palette(...))
```

### Rendering Logic

Both top and bottom lines follow the same rendering algorithm, parameterized by their title and alignment.

**Left-aligned** (title "Hello", fill `/`):
```
Hello ///////////////////////////
^text ^gap ^fill to end
```

**Center-aligned:**
```
/////////// Hello ///////////////
^fill ^gap ^text ^gap ^fill
```

**Right-aligned:**
```
/////////////////////////// Hello
^fill to start ^gap ^text
```

Rules:
- Gap between text and fill is 1 space
- If no title text, the entire line is fill characters
- If title text is wider than the area, truncate with ellipsis and show no fill
- Custom fill strings are repeated and truncated to fit the remaining width
- Both top and bottom lines always render (even if no title — they show fill-only)

### Inner Area

```rust
impl Fieldset<'_> {
    pub fn inner(&self, area: Rect) -> Rect {
        Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(2),
        }
    }
}
```

No padding — the caller handles spacing between the rules and their content.

### Widget Traits

Stateless only — no `FieldsetState` needed. No interactive behavior, no animation.

```rust
impl Widget for Fieldset<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) { ... }
}

impl Widget for &Fieldset<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) { ... }
}
```

### Palette Integration

```rust
impl FieldsetStyles {
    pub fn from_palette(palette: &Palette) -> Self {
        Self {
            title: Style::new().fg(palette.secondary).bold(),
            rule: Style::new().fg(palette.border),
        }
    }

    pub fn dark() -> Self { Self::from_palette(&Palette::dark()) }
    pub fn light() -> Self { Self::from_palette(&Palette::light()) }
}
```

- `title` uses `palette.secondary` — the heading/label color, consistent with other widgets
- `rule` uses `palette.border` — semantic match for decorative lines

### Not Using ratatui's Block

The fieldset is built standalone, not wrapping ratatui's `Block`, because:
- Block renders box-drawing borders with corners — fieldset has no side borders, no corners
- Block's title rendering places text within border lines between corners — fieldset wants text followed by repeating fill
- Block's `border_set` expects fixed corner/edge characters, not a repeating pattern
- The alignment model (fill on both sides of centered text) doesn't match Block's title model
- The only shared concept is `.inner()` returning a `Rect`, which is trivial to implement

## Files to Create/Modify

1. **`crates/ratatui-cheese/src/fieldset.rs`** — new widget module
2. **`crates/ratatui-cheese/src/lib.rs`** — add `pub mod fieldset;` and re-exports
3. **`crates/ratatui-cheese/examples/fieldset.rs`** — standalone example
4. **`crates/showcase/src/widgets/fieldset.rs`** — showcase component
5. **`crates/showcase/src/widgets/mod.rs`** — register fieldset component

### Reusable Utilities

- `utils::display_width()` and `utils::truncate_with_ellipsis()` from `crates/ratatui-cheese/src/utils.rs` for measuring and truncating title text
- `Palette` and theming from `crates/ratatui-cheese/src/theme.rs`
- `Component` trait from `crates/showcase/src/widgets/mod.rs`

## Example & Showcase

**Standalone example** (`examples/fieldset.rs`):
- Cycles through fill styles with `h/l` or arrow keys (Slash, Dash, Dot, Double, Thick, Star, Custom)
- Shows the current fill name in the top title
- Bottom title shows alignment info
- Help bar with keybindings
- Uses `Palette::charm()`

**Showcase component** (`showcase/src/widgets/fieldset.rs`):
- Implements `Component` trait
- Cycles through fill presets with `h/l`
- Renders a fieldset wrapping sample content
- Shows fill name in the showcase block title

Both follow the existing pattern of cycling through variants with `h/l` keys.

## Verification

1. `cargo build` — compiles without errors
2. `cargo test` — all existing tests pass
3. `cargo run --example fieldset` — example runs, cycles through fills and alignments
4. `cargo run -p showcase` — fieldset appears in showcase, cycles correctly
5. Visual check: all 6 presets + custom render correctly, alignment works for left/center/right on both top and bottom lines
6. Edge cases: very narrow widths, empty titles, long titles that need truncation
