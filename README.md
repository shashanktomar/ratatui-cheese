# ratatui-cheese

[![CI](https://github.com/shashanktomar/ratatui-cheese/actions/workflows/ci.yml/badge.svg)](https://github.com/shashanktomar/ratatui-cheese/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ratatui-cheese)](https://crates.io/crates/ratatui-cheese)
[![License: MIT](https://img.shields.io/crates/l/ratatui-cheese)](https://github.com/shashanktomar/ratatui-cheese/blob/main/LICENSE)

Bubbletea-inspired components for [Ratatui](https://github.com/ratatui/ratatui). Bringing the ergonomics of [Charm's Bubbles](https://github.com/charmbracelet/bubbles) to the Rust TUI ecosystem.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-cheese = "0.6"
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

### Tree

Expandable parent/child tree list with collapsible groups, right-aligned counts, text truncation, and palette-based theming.

![Tree](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/tree.gif)

[Usage & docs](crates/ratatui-cheese/README.md#tree)

### Paginator

Page indicator with dot and arabic display modes. Tracks pagination state and provides slice helpers.

![Paginator](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/paginator.gif)

[Usage & docs](crates/ratatui-cheese/README.md#paginator)

### List

Paginated list with item delegation. Each item controls its own rendering via the `ListItem` trait. Supports custom headers, configurable selection indicators, and palette-based theming.

![List](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/list.gif)

[Usage & docs](crates/ratatui-cheese/README.md#list)

### Fieldset

Container widget with decorated horizontal rule lines and optional titles. Supports multiple fill styles and independent top/bottom alignment.

![Fieldset](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/fieldset.gif)

[Usage & docs](crates/ratatui-cheese/README.md#fieldset)

### Select

Single-selection widget for picking one option from a vertical list. Supports disabled options, custom cursor, and validation.

![Select](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/select.gif)

[Usage & docs](crates/ratatui-cheese/README.md#select)

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
