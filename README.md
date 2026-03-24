# ratatui-cheese

Bubbletea-inspired components for [Ratatui](https://github.com/ratatui/ratatui). Bringing the ergonomics of [Charm's Bubbles](https://github.com/charmbracelet/bubbles) to the Rust TUI ecosystem.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-cheese = "0.1"
```

## Contributing

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [just](https://github.com/casey/just) — command runner
- [nushell](https://www.nushell.sh/) — used for tooling scripts
- [gum](https://github.com/charmbracelet/gum) — terminal UI for scripts

### Getting Started

Clone the repo and run the setup command. This installs all required tools and syncs reference repositories:

```sh
just setup
```

### Development

```sh
just c          # check for compilation errors
just t          # run tests
just l          # lint (clippy + fmt check)
just f          # auto-fix formatting and clippy warnings
just w          # start bacon watch for live feedback
just all        # run all quality gates
```

### Reference Repos

The project tracks upstream repositories (ratatui, bubbles, bubbletea) for reference. These are synced into `.tmp/code/` and gitignored.

```sh
just update-reference-repos   # clone/update reference repos
just reference-status         # show sync status
```

## License

MIT
