# Showcase Integration

This is the reference for Phase 5 of the create-widget skill. It covers how to add a new widget to the cheese-showcase demo app.

## Overview

`cheese-showcase` is a binary crate at `crates/cheese-showcase/` that serves as a live demo of all ratatui-cheese widgets. Users run it via `just showcase` to see every widget in action.

**Note:** The showcase app is still being built out. If it's currently a placeholder (`println!` stub), you'll need to set up the basic TUI app structure first before adding your widget. If the showcase already has a working TUI app with other widgets, follow the existing pattern for adding yours.

## Adding a Dependency

In `crates/cheese-showcase/Cargo.toml`, add your widget crate:

```toml
[dependencies]
cheese-core = { workspace = true }
cheese-<name> = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
```

## Showcase App Structure

If the showcase app needs to be built from scratch, use this minimal structure:

```rust
use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn draw(frame: &mut Frame) {
    let areas = Layout::vertical([
        Constraint::Length(3),  // widget 1
        Constraint::Length(3),  // widget 2
        Constraint::Min(0),    // remaining space
    ]).split(frame.area());

    // Render each widget in its area
    frame.render_widget(&my_widget, areas[0]);
}
```

## Adding Your Widget

If the showcase already exists, the pattern is:

1. Import your widget at the top of `main.rs`
2. Add a layout slot for it (adjust `Constraint` values)
3. Create an instance with representative data and render it
4. If it's a `StatefulWidget`, add its state to the app state struct

```rust
use cheese_<name>::<WidgetName>;

// In the draw function, add:
let widget = <WidgetName>::new(/* demo data */)
    .block(Block::bordered().title(" <Widget Name> "))
    .style(Style::default());
frame.render_widget(&widget, areas[N]);
```

## Verification

```bash
just showcase    # run and visually inspect
just all         # ensure all quality gates still pass
```
