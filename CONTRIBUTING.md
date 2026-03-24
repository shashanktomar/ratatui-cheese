# Contributing to ratatui-cheese

Thanks for your interest in contributing! This guide will help you get set up and familiar with the workflow.

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [just](https://github.com/casey/just) — command runner
- [nushell](https://www.nushell.sh/) — used for tooling scripts
- [gum](https://github.com/charmbracelet/gum) — terminal UI for scripts
- [vhs](https://github.com/charmbracelet/vhs) — terminal GIF recorder for docs

## Getting Started

Clone the repo and run the setup command. This installs all required tools and syncs reference repositories:

```sh
git clone https://github.com/shashanktomar/ratatui-cheese.git
cd ratatui-cheese
just setup
```

## Development

```sh
just c          # check for compilation errors
just t          # run tests
just l          # lint (clippy + fmt check)
just f          # auto-fix formatting and clippy warnings
just w          # start bacon watch for live feedback
just all        # run all quality gates
```

## Reference Repos

The project tracks upstream repositories (ratatui, bubbles, bubbletea) for reference. These are synced into `.ref/code/` and gitignored.

```sh
just update-reference-repos   # clone/update reference repos
just reference-status         # show sync status
```

## Submitting Changes

1. Fork the repo and create a branch from `main`
2. Run `just all` to ensure all quality gates pass
3. Open a pull request with a clear description of your changes
