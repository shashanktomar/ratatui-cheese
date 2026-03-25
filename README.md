# ratatui-cheese

[![CI](https://github.com/shashanktomar/ratatui-cheese/actions/workflows/ci.yml/badge.svg)](https://github.com/shashanktomar/ratatui-cheese/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ratatui-cheese)](https://crates.io/crates/ratatui-cheese)
[![License: MIT](https://img.shields.io/crates/l/ratatui-cheese)](https://github.com/shashanktomar/ratatui-cheese/blob/main/LICENSE)

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

![Spinners](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/spinners.gif)

[Usage & docs](crates/ratatui-cheese/README.md#spinner)

### Help

Keyboard shortcut help view with short (single-line) and full (multi-column) modes, matching Charmbracelet's Bubbles help component.

![Help](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/help.gif)

[Usage & docs](crates/ratatui-cheese/README.md#help)

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
