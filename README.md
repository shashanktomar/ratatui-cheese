# ratatui-cheese

Bubbletea-inspired components for [Ratatui](https://github.com/ratatui/ratatui). Bringing the ergonomics of [Charm's Bubbles](https://github.com/charmbracelet/bubbles) to the Rust TUI ecosystem.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-cheese = "0.1"
```

## Widgets

### Spinner

Animated spinner with 12 preset types matching Charmbracelet's Bubbles — Line, Dot, MiniDot, Jump, Pulse, Points, Globe, Moon, Monkey, Meter, Hamburger, Ellipsis.

<!-- TODO: Add screenshot -->

[Usage & docs](crates/ratatui-cheese/README.md#spinner)

## Examples

Run the showcase to see all widgets in action:

```sh
just showcase
```

Run individual examples:

```sh
just example spinners
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions and development workflow.

## License

MIT
