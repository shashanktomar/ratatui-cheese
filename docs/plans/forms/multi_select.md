# Plan: MultiSelect Widget

## Context

Multiple selection widget inspired by huh's MultiSelect field. User toggles options on/off from a vertical list.

## Lessons from Input + Select

- `ValidationResult` lives in `field.rs` — import from there
- Use `error`/`success` from Palette, `faint` for disabled
- Content-only test helper with `unicode_width` for display width
- Options store `enabled: bool` — disabled options rendered faint, skipped by navigation
- `next()`/`prev()` take `&[Option]` not `usize` — needed for skipping disabled
- Example/showcase `VariantData` stores `Vec<MultiSelectOption>` directly (not `Vec<&str>`)
- Clone options before mutable state access to avoid borrow conflicts
- `live_validate` flag on variants for validation-on-change behavior
- Example uses `h`/`l` for variant cycling, showcase too (detail panel captures all keys)

## Layout

```
Toppings                        <- title: secondary, bold
Choose up to 4.                 <- description: muted (optional)

  > ✓ Lettuce                   <- cursor + checked: primary
    ✓ Tomatoes                  <- checked (no cursor): primary
    • Charm Sauce               <- unchecked: muted
    • Jalapeños                 <- disabled: faint (if disabled)
    • Cheese
    • Vegan Cheese
    • Nutella

* validation message            <- error/success (optional)
```

## Files

**Create:**
- `crates/ratatui-cheese/src/multi_select.rs`
- `crates/ratatui-cheese/examples/multi_select.rs`
- `crates/showcase/src/widgets/multi_select.rs`

**Modify:**
- `crates/ratatui-cheese/src/lib.rs` — add `pub mod multi_select;`
- `crates/showcase/src/widgets/mod.rs` — add `pub mod multi_select;`
- `crates/showcase/src/app.rs` — register `MultiSelectComponent`

## Structures

### `MultiSelectOption<'a>`
```rust
pub struct MultiSelectOption<'a> {
    pub label: &'a str,
    pub enabled: bool,
}
```
- `new(label)` constructor, `.enabled(false)` builder
- `From<&str>` for convenience (enabled by default)

### `MultiSelectStyles`
- `title`, `description` — same as Input/Select
- `cursor` — cursor indicator (primary)
- `checked` — checked indicator + label (primary)
- `unchecked` — unchecked indicator + label (foreground)
- `disabled` — disabled option (faint)
- `validation_error` — from `p.error`
- `validation_success` — from `p.success`
- `from_palette(&Palette)` derives all

### `MultiSelect<'a>`
Builder with:
- `title: &'a str`
- `description: Option<&'a str>`
- `options: &'a [MultiSelectOption<'a>]`
- `limit: Option<usize>` — max selections (None = unlimited)
- `cursor_indicator: &'a str` (default: ">")
- `checked_indicator: &'a str` (default: "✓")
- `unchecked_indicator: &'a str` (default: "•")
- `styles: MultiSelectStyles`

### `MultiSelectState`
Constructor: `new(count: usize)` — initializes `selected: Vec<bool>` with `count` false values.

Fields:
- `cursor: usize`
- `selected: Vec<bool>`
- `focused: bool`
- `validation_message: Option<(ValidationKind, String)>`
- `validator: Option<Box<dyn Fn(&[bool]) -> ValidationResult>>`

State methods:
- `next(&[MultiSelectOption])` — move cursor down, skip disabled (wrap)
- `prev(&[MultiSelectOption])` — move cursor up, skip disabled (wrap)
- `toggle_current()` — toggle at cursor (respects limit — only toggle on if under limit)
- `select_all(limit)` — select all enabled (up to limit)
- `deselect_all()` — deselect all
- `cursor() -> usize`
- `is_selected(index) -> bool`
- `selected_indices() -> Vec<usize>`
- `selected_count() -> usize`
- `set_focused(bool)` — validates on blur
- `validate() -> bool`
- `set_validation()`, `validation()`
- `.validator()` — builder method

### Rendering

1. Title line
2. Description line (if set)
3. Options list — each on its own line:
   - Disabled: `  label` in faint style (no indicator)
   - Cursor + checked: `> ✓ Label` (cursor style + checked style)
   - Cursor + unchecked: `> • Label` (cursor style + unchecked style)
   - Checked: `  ✓ Label` (checked style)
   - Unchecked: `  • Label` (unchecked style)
   - Indent: cursor_width + space + indicator_width + space + label
4. Validation message line

### Widget impls
Same 4-impl pattern as Select.

## Example Variants (astrophysics theme)

| Name | Description |
|------|-------------|
| Basic | Planetary instruments, unlimited |
| With limit | "Choose up to 3 experiments", limit enforced |
| Pre-selected | Some instruments pre-checked |
| Disabled options | Some instruments unavailable |
| Validation | Validator requiring at least 1, live validate on toggle |

## Showcase

Same cycling variant pattern. `j`/`k` navigate, `space` toggles, `h`/`l` cycle variants, `enter` validates.

## Verification

1. `just all`
2. `just example multi_select`
3. `just showcase`
