---
name: create-widget
description: Create a new widget module in the ratatui-cheese project. Use this skill whenever the user wants to add a new widget, component, or UI element to the project. Also trigger when the user mentions creating a spinner, list, input, textarea, button, or any TUI component, or says things like "add a new widget", "create a component", "build a new module". Even if the user just says something like "let's do a spinner next" in the context of this project, use this skill.
---

# Create Widget

This skill guides you through creating a new widget module in the ratatui-cheese project — a Bubbletea-inspired component library for Ratatui.

## Project Context

ratatui-cheese is a Rust workspace with:
- `crates/ratatui-cheese/` — the main library crate, one module per widget
- `crates/showcase/` — TUI demo app showing all widgets

All widgets live as modules inside `crates/ratatui-cheese/src/`. Users import as `use ratatui_cheese::<widget>::*`.

Reference implementations live in `.ref/code/` (ratatui, bubbles, bubbletea) — consult them for design inspiration. The `project-manifest.yaml` lists these repos.

**Key versions:** ratatui 0.30, crossterm 0.29, Rust edition 2024.

## Design Principle: Visual Fidelity

The entire point of this project is to bring Charmbracelet's polished TUI experience to Rust. Every widget must be **visually identical** to its Bubbles/Bubbletea counterpart — same spacing, same characters, same behavior, same feel. This is non-negotiable.

When implementing a widget, run the original Go example (in `.ref/code/bubbles/` or `.ref/code/bubbletea/examples/`) and match its visual output cell-for-cell. If something looks "close enough" but not exact, it's not done. The reference implementations are the spec.

## Workflow

Follow these phases in order. Each phase has a reference doc in `references/` — read it when you reach that phase.

### Phase 1: Understand the Widget

Before writing code, get clear on what the widget does:

1. **Ask the user** what the widget should do, what interactions it supports, and any visual examples they have in mind
2. **Find the Bubbles/Bubbletea original** — this is the visual spec. Look in `.ref/code/bubbles/<widget>/` for the Go component and `.ref/code/bubbletea/examples/` for usage examples. Study the View() output carefully — your Rust widget must produce identical visual output.
3. **Research ratatui patterns** in `.ref/code/`:
   - Check ratatui's built-in widgets in `.ref/code/ratatui/ratatui-widgets/src/` for similar patterns you can build on
   - Check ratatui's examples in `.ref/code/ratatui/examples/` for rendering techniques
4. **Present a side-by-side API comparison** to the user before writing code:
   - **Bubbles API** — list the Go model's public fields, methods, constructor options, and what `View()` returns
   - **Proposed Rust API** — show the equivalent struct, builder methods, trait impls, and state
   - **Differences & tradeoffs** — call out anything you're adding (e.g., `block()`) or dropping, and why. Don't blindly copy the widget-patterns template — if the Bubbles original doesn't have a concept (like Block wrapping), don't add it just because the template says to. Fidelity to the original trumps template defaults.

### Phase 2: Add the Module

Create a new file at `crates/ratatui-cheese/src/<widget>.rs` and re-export it from `lib.rs`:

```rust
// In crates/ratatui-cheese/src/lib.rs, add:
pub mod <widget>;
```

No new crate, no Cargo.toml changes. Run `cargo check --all-targets` to verify it compiles.

### Phase 3: Implement the Widget

Read `references/widget-patterns.md` for the full implementation guide.

The short version — a typical widget needs:

1. **Widget struct** with `style: Style` and widget-specific fields. Add `block: Option<Block<'a>>` only if the Bubbles original has a wrapping/border concept — don't add it just because the template shows it.
2. **Constructor** (`new()`) and **builder methods** (`.style()`, etc.) with `#[must_use]`
3. **`Widget` trait** implemented on both `Self` (consuming) and `&Self` (borrowing)
4. **`Styled` trait** implementation
5. Optionally, `StatefulWidget` if the widget has mutable state (selection, cursor position, etc.)

### Phase 4: Write Tests

Read `references/widget-tests.md` for testing patterns.

Use ratatui's `Buffer`-based snapshot testing: render the widget into a buffer and compare against expected output. Cover at minimum:
- Empty/zero-size area
- Basic render
- Render with block (if applicable)
- Render with styling
- Any widget-specific behavior

### Phase 5: Write an Example

If the Bubbles/Bubbletea repo has an example for this widget (check `.ref/code/bubbletea/examples/`), port it as a standalone example at `crates/ratatui-cheese/examples/<widget>.rs`.

The example should be a direct port of the Go original — same layout, same keybindings, same colors. Keep it minimal and self-contained (~80-120 lines). Run with `just example <widget>`.

### Phase 6: Add to Showcase

Read `references/showcase-integration.md` for how to integrate into the demo app.

Add the widget to `showcase` so users can see it in action via `just showcase`.

### Phase 7: Update READMEs

Read `references/widget-readme.md` for the template.

Add a section for the new widget in:
- `crates/ratatui-cheese/README.md` — usage example and `cargo run --example <widget>` command
- Root `README.md` — screenshot placeholder and link

### Phase 8: Verify

Run all quality gates before calling it done:

```bash
just all    # check + test + lint + dead-code
just showcase  # visually verify
just example <widget>  # verify example matches Go original
```

## Reference Files

These contain detailed patterns — read them when you reach the relevant phase:

| File | When to read |
|------|-------------|
| `references/widget-patterns.md` | Phase 3 — implementing the widget |
| `references/widget-tests.md` | Phase 4 — writing tests |
| `references/showcase-integration.md` | Phase 5 — adding to showcase |
| `references/widget-readme.md` | Phase 6 — writing the README |
