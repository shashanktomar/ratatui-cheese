# Widget README Template

This is the reference for Phase 6 of the create-widget skill. Each new widget gets a section in the crate README and root README.

## Add to Crate README

Add a new section to `crates/ratatui-cheese/README.md` under "## Widgets":

```markdown
### <Widget Name>

<One-line description.>

\```rust
use ratatui_cheese::<widget>::{<WidgetName>, <WidgetState>};

let widget = <WidgetName>::new(/* args */)
    .style(Style::default());

frame.render_widget(&widget, area);
\```
```

## Update the Repo README

Also add an entry in the root `README.md` under "## Widgets":

```markdown
### <Widget Name>

<One-line description.>

<!-- TODO: Add screenshot -->

[Usage & docs](crates/ratatui-cheese/README.md#<widget-name>)
```

The root README is the first thing users see — it should showcase each widget with a screenshot and link to the crate README for detailed usage. Screenshots are a placeholder for now; we'll add a capture workflow later.
