# Plan: MultiSelect Widget

## Context

Multiple selection widget inspired by huh's MultiSelect field. User toggles options on/off from a vertical list.

## Layout

```
Toppings                        <- title: secondary, bold
Choose up to 4.                 <- description: muted (optional)

  > ✓ Lettuce                   <- cursor + checked: primary
    ✓ Tomatoes                  <- checked (no cursor): primary
    • Charm Sauce               <- unchecked: muted
    • Jalapeños
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
Same as `SelectOption<'a>` — `label` + optional `description`. Reuse the same type from a shared location, or define identically. Consider putting `SelectOption` in `field.rs` and reusing it as the option type for both widgets.

### `MultiSelectStyles`
- `title`, `description` — same as Input
- `option` — unselected option text (foreground)
- `cursor` — cursor indicator (primary)
- `checked` — checked indicator + text (primary)
- `unchecked` — unchecked indicator (muted)
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

Builder methods: `.description()`, `.options()`, `.limit()`, `.cursor_indicator()`, `.checked_indicator()`, `.unchecked_indicator()`, `.styles()`, `.palette()`

### `MultiSelectState`
- `cursor: usize` — currently highlighted index
- `selected: Vec<bool>` — one per option
- `focused: bool`
- `validation_message: Option<(ValidationKind, String)>`
- `validator: Option<Box<dyn Fn(&[bool]) -> ValidationResult>>` — validates selection state

State methods:
- `next()` — move cursor down (wrap)
- `prev()` — move cursor up (wrap)
- `toggle_current()` — toggle selection at cursor (respects limit)
- `select_all()` — select all (respects limit)
- `deselect_all()` — deselect all
- `cursor() -> usize`
- `is_selected(index) -> bool`
- `selected_indices() -> Vec<usize>`
- `selected_count() -> usize`
- `set_focused(bool)` — validates on blur
- `validate() -> bool`
- `set_validation()`, `validation()`
- `.validator()` — builder method

Constructor: `MultiSelectState::new(count: usize)` — initializes `selected` vec with `count` false values.

### Rendering

1. Title line
2. Description line (if set)
3. Options list — each on its own line:
   - `> ✓ Label` — cursor + checked (cursor style + checked style)
   - `  ✓ Label` — no cursor, checked (checked style)
   - `> • Label` — cursor + unchecked (cursor style + unchecked style)
   - `  • Label` — no cursor, unchecked (unchecked style)
   - Indent: cursor_indicator width + space + indicator + space + label
4. Validation message line

No scrolling for v1 — same as Select.

### Widget impls
Same 4-impl pattern.

## Example Variants

| Name | Description |
|------|-------------|
| Basic | Toppings list, unlimited selections |
| With limit | "Choose up to 3", limit enforced |
| Pre-selected | Some options pre-checked |
| Custom indicators | Uses "x" / "○" instead of "✓" / "•" |
| Validation | Validator requiring at least 1 selection, starts empty |

## Showcase

Add `MultiSelectComponent` with cycling variants. `j`/`k` navigate, `space` toggles, `←/→` switch variants.

## Verification

1. `just check`
2. `just lint`
3. `just test`
4. `just example multi_select`
5. `just showcase`
