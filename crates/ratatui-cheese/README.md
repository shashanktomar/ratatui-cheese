# ratatui-cheese

Bubbletea-inspired widgets for [Ratatui](https://github.com/ratatui/ratatui). Bringing the ergonomics of [Charm's Bubbles](https://github.com/charmbracelet/bubbles) to the Rust TUI ecosystem.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-cheese = "0.1"
```

## Widgets

### Spinner

Animated spinner with 12 preset types matching Charmbracelet's Bubbles — Line, Dot, MiniDot, Jump, Pulse, Points, Globe, Moon, Monkey, Meter, Hamburger, Ellipsis.

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

![Spinners](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/spinners.gif)

See it in action (a direct port of the [Bubbletea spinners example](https://github.com/charmbracelet/bubbletea/tree/main/examples/spinners)):

```sh
cargo run --example spinners
```

## License

MIT
