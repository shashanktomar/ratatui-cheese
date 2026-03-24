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
use ratatui_cheese::spinner::{Spinner, SpinnerState, SpinnerType};
use ratatui::style::{Color, Style};

let spinner = Spinner::new(SpinnerType::Dot)
    .style(Style::default().fg(Color::Blue));

let mut state = SpinnerState::default();

// In your render loop:
frame.render_stateful_widget(&spinner, area, &mut state);

// On a timer matching the spinner's interval:
state.tick(spinner.frames().len());
```

## License

MIT
