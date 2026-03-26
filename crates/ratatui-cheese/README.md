# ratatui-cheese

Bubbletea-inspired widgets for [Ratatui](https://github.com/ratatui/ratatui). Bringing the ergonomics of [Charm's Bubbles](https://github.com/charmbracelet/bubbles) to the Rust TUI ecosystem.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-cheese = "0.3"
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

## License

MIT
