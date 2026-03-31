# Plan: Select Widget

## Context

Single selection widget inspired by huh's Select field. User picks one option from a vertical list with a cursor indicator.

## Layout

```
Choose your burger *            <- title: secondary, bold
At Charm we truly have...       <- description: muted (optional)

    Charmburger Classic         <- option: foreground
    Chickwich
  > Fishburger                  <- cursor + selected: primary
    Charmpossible™ Burger

* no fish today, sorry          <- validation (optional)
```

## Files

**Create:**
- `crates/ratatui-cheese/src/select.rs`
- `crates/ratatui-cheese/examples/select.rs`
- `crates/showcase/src/widgets/select.rs`

**Modify:**
- `crates/ratatui-cheese/src/lib.rs` — add `pub mod select;`
- `crates/showcase/src/widgets/mod.rs` — add `pub mod select;`
- `crates/showcase/src/app.rs` — register `SelectComponent`

## Structures

### `SelectOption<'a>`
```rust
pub struct SelectOption<'a> {
    pub label: &'a str,
    pub description: Option<&'a str>,
}
```
Convenience: `impl<'a> From<&'a str> for SelectOption<'a>` so users can pass `&["A", "B", "C"]`.

### `SelectStyles`
- `title`, `description` — same as Input
- `option` — unselected option text (foreground)
- `selected_option` — option at cursor (primary)
- `cursor` — cursor indicator character (primary)
- `validation_error` — from `p.error`
- `validation_success` — from `p.success`
- `from_palette(&Palette)` derives all

### `Select<'a>`
Builder with:
- `title: &'a str`
- `description: Option<&'a str>`
- `options: &'a [SelectOption<'a>]`
- `cursor_indicator: &'a str` (default: ">")
- `styles: SelectStyles`

Builder methods: `.description()`, `.options()`, `.cursor_indicator()`, `.styles()`, `.palette()`

### `SelectState`
- `cursor: usize` — currently highlighted index
- `focused: bool`
- `validation_message: Option<(ValidationKind, String)>`
- `validator: Option<Box<dyn Fn(usize) -> ValidationResult>>` — validates selected index

State methods:
- `next()` — move cursor down (wrap at end)
- `prev()` — move cursor up (wrap at start)
- `selected() -> usize` — current index
- `set_cursor(usize)` — set directly (clamps)
- `set_focused(bool)` — validates on blur
- `validate() -> bool`
- `set_validation()`, `validation()`
- `.validator()` — builder method

### Rendering

1. Title line
2. Description line (if set)
3. Options list — each option on its own line:
   - Cursor indicator + space + label (selected option in `selected_option` style)
   - Padding spaces + label (unselected options in `option` style)
   - The cursor indicator width determines the indent for all options
4. Validation message line

Note: No scrolling for v1. Options render within available height, overflow is clipped. Scrolling can be added later if needed.

### Widget impls
Same 4-impl pattern. Validator takes `usize` (selected index) not `&str`.

## Example Variants

| Name | Description |
|------|-------------|
| Basic | Burger menu with 4 options |
| With description | Options have sub-descriptions |
| Custom cursor | Uses "→" instead of ">" |
| Pre-selected | Starts with 3rd option selected |
| Validation | Validator rejects certain options, pre-set to show error |
| Long list | 10+ options to show clipping behavior |

## Showcase

Add `SelectComponent` with cycling variants. `j`/`k` navigate options within a variant, `←/→` switch variants.

## Verification

1. `just check`
2. `just lint`
3. `just test`
4. `just example select`
5. `just showcase`
