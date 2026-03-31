# Plan: Note Widget

## Context

Display-only informational widget inspired by huh's Note field. No state needed — purely renders text. Simplest widget in the form family.

## Layout

```
Important Notice                <- title: secondary, bold
Please read the terms and       <- body: muted, word-wrapped
conditions before proceeding.
```

## Files

**Create:**
- `crates/ratatui-cheese/src/note.rs`
- `crates/ratatui-cheese/examples/note.rs`
- `crates/showcase/src/widgets/note.rs`

**Modify:**
- `crates/ratatui-cheese/src/lib.rs` — add `pub mod note;`
- `crates/showcase/src/widgets/mod.rs` — add `pub mod note;`
- `crates/showcase/src/app.rs` — register `NoteComponent`

## Structures

### `NoteStyles`
- `title` — title text: `fg(p.secondary)`, bold
- `body` — body text: `fg(p.muted)`
- `from_palette(&Palette)` derives all

### `Note<'a>`
Builder with:
- `title: Option<&'a str>`
- `body: Option<&'a str>`
- `styles: NoteStyles`

Builder methods: `.title()`, `.body()`, `.styles()`, `.palette()`

Constructor: `Note::new()` — both title and body optional.

### No state

Note is stateless — implements only `Widget`, not `StatefulWidget`. Follows the pattern of `Fieldset` (stateless widget).

### Rendering

1. Title line (if set) — secondary, bold
2. Body text (if set) — muted, rendered line by line
   - No word wrapping in v1 — just render what fits, truncate at width
   - Each line of body on its own row

### Widget impls
`Widget` for owned and `&Note` — no StatefulWidget needed.

## Example Variants

| Name | Description |
|------|-------------|
| Title + body | Both title and body text |
| Title only | Just a heading |
| Body only | Just paragraph text |
| Long body | Multi-line body to show line rendering |
| Styled | Custom palette (e.g., ocean) |

## Showcase

Add `NoteComponent` with cycling variants. `←/→` switch variants. No other interaction since Note is display-only.

## Verification

1. `just check`
2. `just lint`
3. `just test`
4. `just example note`
5. `just showcase`
