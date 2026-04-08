# Plan: Textarea Widget

## Context

Multi-line text input widget inspired by huh's Text field. Builds on patterns established by the Input widget.

## Lessons from Input

- Use `ValidationResult` (`Result<Option<String>, String>`) for validators from the start
- Use `error`/`success` from Palette — never hardcode colors
- Content-only test helper (`assert_renders_content`) — don't compare styles in layout tests
- Separate style tests that check specific cells
- Showcase must match example — cycling variants with paginator
- `Variant` struct with `char_limit`, `live_validate` flags to control example behavior
- Showcase uses split-focus (Tab to switch sidebar/detail) — all keys go to component when detail is focused

## Layout

```
Tell me a story.                <- title: secondary, bold
A short description.            <- description: muted (optional)

│ 1 Makin' my way downtown      <- border + line numbers + text
│ 2 Walking█                    <- cursor on current line
│ 3                             <- empty lines
│ 4

* validation message            <- error/success (optional)
```

## Files

**Create:**
- `crates/ratatui-cheese/src/textarea.rs`
- `crates/ratatui-cheese/examples/textarea.rs`
- `crates/showcase/src/widgets/textarea.rs`

**Modify:**
- `crates/ratatui-cheese/src/lib.rs` — add `pub mod textarea;`
- `crates/showcase/src/widgets/mod.rs` — add `pub mod textarea;`
- `crates/showcase/src/app.rs` — register `TextareaComponent`

## Structures

### `TextareaStyles`
- `title`, `description` — same as Input
- `line_number` — style for line numbers (muted)
- `border` — left border pipe (faint)
- `text` — entered text (foreground)
- `cursor` — visual cursor (inverted)
- `placeholder` — placeholder text (faint)
- `validation_error` — from `p.error`
- `validation_success` — from `p.success`
- `from_palette(&Palette)` derives all

### `Textarea<'a>`
Builder with:
- `title: &'a str`
- `description: Option<&'a str>`
- `placeholder: Option<&'a str>`
- `show_line_numbers: bool` (default: true)
- `styles: TextareaStyles`

Builder methods: `.description()`, `.placeholder()`, `.show_line_numbers()`, `.styles()`, `.palette()`

### `TextareaState`
- `lines: Vec<String>` — one string per line
- `cursor_row: usize`, `cursor_col: usize` — char-indexed
- `scroll_offset: usize` — first visible line
- `focused: bool`
- `validation_message: Option<(ValidationKind, String)>`
- `validator: Option<ValidatorFn>` — uses same `ValidationResult` pattern

State methods:
- `insert_char(ch)` — insert at cursor, handle wide chars
- `insert_newline()` — split line at cursor, move to next
- `delete_before()` — backspace (join lines if at col 0)
- `delete_at()` — delete (join with next line if at end)
- `move_left()`, `move_right()`, `move_up()`, `move_down()`
- `home()`, `end()` — within current line
- `value() -> String` — joins lines with `\n`
- `set_value(String)` — splits on `\n`
- `line_count() -> usize`
- `set_focused(bool)` — validates on blur
- `validate() -> bool`
- `set_validation()`, `validation()`
- `.validator()` — builder method

### Rendering

1. Title line
2. Description line (if set)
3. Text area with border:
   - Left border: `│` in `border` style
   - Line numbers (if enabled): right-aligned, `line_number` style
   - Space separator
   - Text content with cursor
4. Scrolling: adjust `scroll_offset` to keep cursor visible
5. Visual cursor: invert style at cursor position when focused
6. Validation message line

### Widget impls
Same 4-impl pattern as Input: `Widget` for owned/ref, `StatefulWidget` for owned/ref.

## Example Variants

| Name | Description |
|------|-------------|
| Basic | Title + description + placeholder, line numbers on |
| No line numbers | `show_line_numbers(false)` |
| With content | Pre-filled multi-line text |
| Validation | Validator requiring non-empty, pre-filled, shows error |
| Live validation | Character count validator, validates on every keystroke |

## Showcase

Add `TextareaComponent` with same cycling variant pattern. Uses `←/→` to switch variants, all other keys go to textarea.

## Verification

1. `just check`
2. `just lint`
3. `just test`
4. `just example textarea`
5. `just showcase` — verify textarea component works
