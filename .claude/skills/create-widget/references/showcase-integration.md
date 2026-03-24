# Showcase Integration

This is the reference for Phase 5 of the create-widget skill. It covers how to add a new widget to the showcase demo app.

## Overview

`showcase` is a binary crate at `crates/showcase/` that serves as a live demo of all ratatui-cheese widgets. Users run it via `just showcase` to see every widget in action.

The showcase has a two-panel layout: a sidebar listing all widgets (navigated with j/k) and a detail panel showing the selected widget's demo. Follow the existing pattern when adding new widgets.

## No Dependency Changes Needed

The showcase already depends on `ratatui-cheese` (workspace), so new widget modules are available automatically. No Cargo.toml changes needed.

## Adding Your Widget

The showcase follows this pattern. Existing code in `crates/showcase/src/main.rs`:

1. **Add to the widget registry** — append your widget name to the `WIDGETS` array
2. **Add state** to the `App` struct for your widget (if it has state)
3. **Add key handlers** — wire up widget-specific controls in the key match
4. **Add a draw function** — create `draw_<widget>_detail(frame, app, area)` and call it from `draw_detail`

```rust
use ratatui_cheese::<widget>::{<WidgetName>, <WidgetState>};

// In WIDGETS array:
const WIDGETS: &[&str] = &["Spinner", "<NewWidget>"];

// In draw_detail:
if WIDGETS[app.selected_widget] == "<NewWidget>" {
    draw_<widget>_detail(frame, app, area);
}
```

## Verification

```bash
just showcase    # run and visually inspect
just all         # ensure all quality gates still pass
```
