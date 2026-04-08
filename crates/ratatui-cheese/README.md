# ratatui-cheese

Bubbletea-inspired widgets for [Ratatui](https://github.com/ratatui/ratatui). Bringing the ergonomics of [Charm's Bubbles](https://github.com/charmbracelet/bubbles) to the Rust TUI ecosystem.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-cheese = "0.6"
```

## Widgets

### Spinner

Animated spinner with 12 preset types matching Charmbracelet's Bubbles — Line, Dot, MiniDot, Jump, Pulse, Points, Globe, Moon, Monkey, Meter, Hamburger, Ellipsis.

<details>
<summary>Usage</summary>

```rust
use std::time::Instant;
use ratatui_cheese::spinner::{Spinner, SpinnerState, SpinnerType};
use ratatui::style::{Color, Style};

let spinner = Spinner::default()
    .style(Style::default().fg(Color::Blue));

let mut state = SpinnerState::new(SpinnerType::Dot);

// In your event loop — pass elapsed time, state handles frame timing:
let now = Instant::now();
state.tick(now - last_tick);
last_tick = now;

// In your draw function:
frame.render_stateful_widget(&spinner, area, &mut state);
```

</details>

![Spinners](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/spinners.gif)

See it in action (a direct port of the [Bubbletea spinners example](https://github.com/charmbracelet/bubbletea/tree/main/examples/spinners)):

```sh
cargo run --example spinners
```

### Help

Keyboard shortcut help view with short (single-line) and full (multi-column) modes, matching Charmbracelet's Bubbles help component.

<details>
<summary>Usage</summary>

```rust
use ratatui_cheese::help::{Binding, Help};

let help = Help::default()
    .bindings(vec![Binding::new("?", "help"), Binding::new("q", "quit")])
    .binding_groups(vec![
        vec![Binding::new("?", "help"), Binding::new("q", "quit")],
    ])
    .show_all(true);

// In your draw function:
frame.render_widget(&help, area);
```

</details>

![Help](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/help.gif)

See it in action (a direct port of the [Bubbletea help example](https://github.com/charmbracelet/bubbletea/tree/main/examples/help)):

```sh
cargo run --example help
```

### Tree

Expandable parent/child tree list with one-level depth. Supports right-aligned counts, text truncation, vertical scrolling, and three display modes (simple, explicit count, parent fallback). Themeable via `Palette`.

<details>
<summary>Usage</summary>

```rust
use ratatui_cheese::tree::{Tree, TreeGroup, TreeItem, TreeState, TreeStyles};
use ratatui_cheese::theme::Palette;

let tree = Tree::default()
    .groups(vec![
        TreeGroup::new(TreeItem::new("Documents").count(87))
            .children(vec![
                TreeItem::new("Architecture Decision Records"),
                TreeItem::new("Team Onboarding Guide"),
            ]),
        TreeGroup::new(TreeItem::new("Bookmarks").count(31)),
    ])
    .styles(TreeStyles::from_palette(&Palette::dark()));

let mut state = TreeState::new(2);

// Navigation:
// state.select_next(&groups);   // move down
// state.toggle_selected();       // expand/collapse
// state.expand_all();            // open everything

// In your draw function:
// frame.render_stateful_widget(&tree, area, &mut state);
```

</details>

![Tree](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/tree.gif)

```sh
cargo run --example tree
```

### Paginator

Page indicator widget with dot (`•••••`) and arabic (`2/10`) display modes. Tracks page state and provides helpers for slicing item collections. Ported from Charmbracelet's Bubbles paginator.

<details>
<summary>Usage</summary>

```rust
use ratatui_cheese::paginator::{Paginator, PaginatorState, PaginatorMode, PaginatorStyles};
use ratatui_cheese::theme::Palette;

let paginator = Paginator::default()
    .mode(PaginatorMode::Dots)
    .styles(PaginatorStyles::from_palette(&Palette::dark()));

let mut state = PaginatorState::new(100, 10); // 100 items, 10 per page

// Navigation:
// state.next_page();
// state.prev_page();

// Slice your items for the current page:
// let (start, end) = state.get_slice_bounds(items.len());
// let page_items = &items[start..end];

// In your draw function:
// frame.render_stateful_widget(&paginator, area, &mut state);
```

</details>

![Paginator](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/paginator.gif)

```sh
cargo run --example paginator
```

### List

Paginated list with item delegation. Each item implements the `ListItem` trait to control its own height and rendering. Supports custom headers via the `ListHeader` trait, configurable selection indicators, item spacing, and palette-based theming. Uses the Paginator widget internally.

<details>
<summary>Usage</summary>

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui_cheese::list::{DefaultHeader, List, ListItem, ListItemContext, ListState};
use ratatui_cheese::theme::Palette;

struct Item(String);
impl ListItem for Item {
    fn height(&self) -> u16 { 1 }
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let p = &ctx.palette;
        let style = if ctx.selected {
            Style::default().fg(p.primary)
        } else {
            Style::default().fg(p.foreground)
        };
        buf.set_string(area.x, area.y, &self.0, style);
    }
}

let items = vec![Item("Apple".into()), Item("Banana".into())];
let header = DefaultHeader::new("Fruits").show_count(true);
let list = List::new(&items)
    .header(&header)
    .palette(Palette::charm());

let mut state = ListState::new(items.len());

// Navigation:
// state.select_next(items.len(), false);
// state.select_prev(items.len(), false);
// state.next_page(items.len());
// state.prev_page(items.len());

// In your draw function:
// frame.render_stateful_widget(&list, area, &mut state);
```

</details>

![List](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/list.gif)

```sh
cargo run --example list
```

### Fieldset

Container widget with decorated horizontal rule lines. Renders top and bottom lines with optional title text and a repeating fill character (slash, dash, dot, double, thick, star, or custom). Supports independent left/center/right alignment for top and bottom titles.

<details>
<summary>Usage</summary>

```rust
use ratatui::layout::Alignment;
use ratatui_cheese::fieldset::{Fieldset, FieldsetFill, FieldsetStyles};
use ratatui_cheese::theme::Palette;

let fieldset = Fieldset::new()
    .title("Section Title")
    .title_bottom("End")
    .top_alignment(Alignment::Left)
    .bottom_alignment(Alignment::Center)
    .fill(FieldsetFill::Slash)
    .styles(FieldsetStyles::from_palette(&Palette::charm()));

// Get the inner area for content:
// let inner = fieldset.inner(area);
// frame.render_widget(&fieldset, area);
// render children into `inner`
```

</details>

![Fieldset](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/fieldset.gif)

```sh
cargo run --example fieldset
```

### Input

Single-line text input with placeholder, password mode, character limit, custom prompt, and validation. Inspired by Charmbracelet's huh input field.

<details>
<summary>Usage</summary>

```rust
use ratatui_cheese::input::{Input, InputState};
use ratatui_cheese::theme::Palette;

let input = Input::new("What's your name?")
    .description("For when your order is ready.")
    .placeholder("Enter name...")
    .palette(&Palette::charm());

let mut state = InputState::new();
state.set_focused(true);

// Text manipulation:
// state.insert_char('H');
// state.delete_before();  // backspace
// state.move_left();      // cursor left

// In your draw function:
// frame.render_stateful_widget(&input, area, &mut state);
```

</details>

![Input](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/input.gif)

See it in action:

```sh
cargo run --example input
```

### Select

Single-selection widget for picking one option from a vertical list. Supports disabled options, custom cursor indicator, and validation. Inspired by Charmbracelet's huh select field.

<details>
<summary>Usage</summary>

```rust
use ratatui_cheese::select::{Select, SelectOption, SelectState};
use ratatui_cheese::theme::Palette;

let options: Vec<SelectOption> = vec!["Mars".into(), "Europa".into(), "Titan".into()];
let select = Select::new("Destination", &options)
    .description("Where would you like to go?")
    .palette(&Palette::charm());

let mut state = SelectState::new(options.len());

// Navigation:
// state.next();   // move cursor down
// state.prev();   // move cursor up

// In your draw function:
// frame.render_stateful_widget(&select, area, &mut state);
```

</details>

![Select](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/select.gif)

See it in action:

```sh
cargo run --example select
```

## License

MIT
