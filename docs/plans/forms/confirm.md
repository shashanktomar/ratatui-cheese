# Plan: Confirm Widget

## Context

Yes/No toggle widget inspired by huh's Confirm field. Simplest stateful form widget — two buttons, one highlighted.

## Layout

```
Would you like 15% off?         <- title: secondary, bold
A special offer just for you.   <- description: muted (optional)

  Yes!    No.                   <- active: highlight bg + on_highlight fg
                                   inactive: faint

* validation message            <- error/success (optional)
```

## Files

**Create:**
- `crates/ratatui-cheese/src/confirm.rs`
- `crates/ratatui-cheese/examples/confirm.rs`
- `crates/showcase/src/widgets/confirm.rs`

**Modify:**
- `crates/ratatui-cheese/src/lib.rs` — add `pub mod confirm;`
- `crates/showcase/src/widgets/mod.rs` — add `pub mod confirm;`
- `crates/showcase/src/app.rs` — register `ConfirmComponent`

## Structures

### `ConfirmStyles`
- `title`, `description` — same as Input
- `active_button` — highlighted button: `bg(p.highlight)`, `fg(p.on_highlight)`
- `inactive_button` — dimmed button: `fg(p.faint)`
- `validation_error` — from `p.error`
- `validation_success` — from `p.success`
- `from_palette(&Palette)` derives all

### `Confirm<'a>`
Builder with:
- `title: &'a str`
- `description: Option<&'a str>`
- `affirmative: &'a str` (default: "Yes")
- `negative: &'a str` (default: "No")
- `styles: ConfirmStyles`

Builder methods: `.description()`, `.affirmative()`, `.negative()`, `.styles()`, `.palette()`

### `ConfirmState`
- `value: bool` — true = affirmative selected
- `focused: bool`
- `validation_message: Option<(ValidationKind, String)>`
- `validator: Option<Box<dyn Fn(bool) -> ValidationResult>>` — validates the boolean value

State methods:
- `toggle()` — flip value
- `accept()` — set true
- `reject()` — set false
- `value() -> bool`
- `set_value(bool)`
- `set_focused(bool)` — validates on blur
- `validate() -> bool`
- `set_validation()`, `validation()`
- `.validator()` — builder method

### Rendering

1. Title line
2. Description line (if set)
3. Blank line
4. Button row:
   - Render affirmative label with active/inactive style based on `value`
   - Gap (3-4 spaces)
   - Render negative label with opposite style
   - Both buttons get padding: `[ Yes! ]` style with spaces inside
5. Validation message line

### Widget impls
Same 4-impl pattern. Default state: `value = true` (affirmative selected).

## Example Variants

| Name | Description |
|------|-------------|
| Basic | "Would you like 15% off?" Yes/No |
| Custom labels | "Confirm deletion?" with "Delete" / "Cancel" |
| With description | Title + description + buttons |
| Default No | Starts with No selected |
| Validation | Validator that requires Yes, starts at No with error shown |

## Showcase

Add `ConfirmComponent` with cycling variants. `h`/`l` or `←/→` toggle, `←/→` at variant level — need to decide: since confirm uses left/right to switch value, use dedicated keys for variant cycling. Options:
- Confirm uses `h`/`l`/`y`/`n`/`space` to toggle value
- `←/→` cycle variants (since left/right aren't needed for a binary toggle)

## Verification

1. `just check`
2. `just lint`
3. `just test`
4. `just example confirm`
5. `just showcase`
