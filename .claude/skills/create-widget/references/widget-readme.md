# Widget README Template

This is the reference for Phase 6 of the create-widget skill. Each widget crate gets a short README.

## Template

Create `crates/cheese-<name>/README.md` with this structure:

```markdown
# cheese-<name>

<One-line description of what the widget does.>

## Usage

Add to your `Cargo.toml`:

\```toml
[dependencies]
cheese-<name> = "0.1"
\```

\```rust
use cheese_<name>::<WidgetName>;
use ratatui::widgets::Block;

let widget = <WidgetName>::new(/* args */)
    .block(Block::bordered().title("<Name>"))
    .style(Style::default());

// Render with ratatui
frame.render_widget(&widget, area);
\```

## Screenshot

<!-- TODO: Add screenshot from showcase -->

## License

MIT
```

Keep it concise. The screenshot section is a placeholder for now — it will be filled in once we have a screenshot workflow.

## Update the Repo README

After creating the widget crate's README, also update the **root `README.md`** at the project root. Add an entry for the new widget in a widgets section:

```markdown
## Widgets

### <Widget Name>

<One-line description.>

<!-- TODO: Add screenshot -->

[Usage & docs](crates/cheese-<name>/README.md)
```

The root README is the first thing users see — it should showcase each widget with a screenshot and link to the widget's own README for detailed usage. Screenshots are a placeholder for now; we'll add a capture workflow later.
