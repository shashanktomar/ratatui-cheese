# Widget README Template

This is the reference for Phase 7 of the create-widget skill. Each new widget gets a section in the crate README and root README.

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

![<Widget Name>](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/<widget>.gif)

See it in action:

\```sh
cargo run --example <widget>
\```
```

## Update the Repo README

Also add an entry in the root `README.md` under "## Widgets":

```markdown
### <Widget Name>

<One-line description.>

![<Widget Name>](https://raw.githubusercontent.com/shashanktomar/ratatui-cheese/images/<widget>.gif)

[Usage & docs](crates/ratatui-cheese/README.md#<widget-name>)
```

## GIF Generation

GIFs are generated with VHS from tape files in `tools/vhs/`. Run `just record <widget>` to generate the GIF locally. GIFs live on the `images` branch — push them there to make README images resolve.
