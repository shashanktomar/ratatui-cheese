# cheese-spinner

Bubbletea-inspired spinner widget for [Ratatui](https://github.com/ratatui/ratatui). Includes all 12 preset spinner types from [Charmbracelet's Bubbles](https://github.com/charmbracelet/bubbles) with identical frame sequences.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cheese-spinner = "0.1"
```

```rust
use cheese_spinner::{Spinner, SpinnerState, SpinnerType};
use ratatui::style::{Color, Style};

let spinner = Spinner::new(SpinnerType::Dot)
    .style(Style::default().fg(Color::Blue));

let mut state = SpinnerState::default();

// In your render loop:
frame.render_stateful_widget(&spinner, area, &mut state);

// On a timer matching the spinner's interval:
state.tick(spinner.frames().len());
```

## Spinner Types

Line, Dot, MiniDot, Jump, Pulse, Points, Globe, Moon, Monkey, Meter, Hamburger, Ellipsis

## Screenshot

<!-- TODO: Add screenshot from showcase -->

## License

MIT
