# List Widget Design Spec

Saved for later implementation (separate branch after paginator is merged).

## Overview

A generic paginated list widget with item delegation. Uses the Paginator widget internally.

## ListItem Trait

```rust
pub struct ListItemContext {
    pub index: usize,
    pub selected: bool,
    pub page: usize,
}

pub trait ListItem {
    /// Height in rows this item occupies.
    fn height(&self) -> u16;

    /// Render this item into the given area.
    fn render(&self, area: Rect, buf: &mut Buffer, context: &ListItemContext);

    /// Handle a key event. Returns true if consumed.
    fn handle_key(&mut self, key: KeyCode) -> bool { false }
}
```

- Single trait hides item internals — list doesn't assume title/description structure
- `height()` needed for pagination calculation (how many items fit per page)
- `render()` receives context with index, selected state, and current page
- `handle_key()` for item-level key delegation, default no-op

## ListStyles

```rust
pub struct ListStyles {
    pub title: Style,
    pub count: Style,
    pub selected: Style,  // selection indicator
}
```

- `from_palette()` / `dark()` / `light()` — matches TreeStyles pattern
- Default is `dark()`

## List Widget

```rust
pub struct List<'a, T: ListItem> {
    items: &'a [T],              // borrows, doesn't own (for &mut key delegation)
    styles: ListStyles,
    title: Option<String>,
    show_count: bool,            // "23 items" below title
    show_paginator: bool,
    paginator: Paginator,        // rendering config for the paginator
    selection_indicator: String,  // default "│"
    infinite_scrolling: bool,    // wrap at boundaries
}
```

- Borrows items (not owns) so user can `&mut` items for `handle_key`
- Lifetime `'a` from borrowed items
- Builder methods: `.items()`, `.styles()`, `.title()`, `.show_count()`, `.show_paginator()`, `.paginator()`, `.selection_indicator()`, `.infinite_scrolling()`
- Trait impls: Widget, StatefulWidget, Styled (same 4-impl pattern)

## ListState

```rust
pub struct ListState {
    selected: usize,
    paginator: PaginatorState,  // owns pagination state internally
}
```

Methods:
- `new(total_items, per_page)`
- `selected()` → usize
- `select(index)`
- `select_next(item_count, infinite)` — auto-advances page at boundary
- `select_prev(item_count, infinite)` — auto-goes to prev page at boundary
- `next_page()` / `prev_page()` — delegates to paginator, moves selection to first item on new page
- `page()` / `total_pages()`

## Render Flow

1. Calculate chrome height (title + count + blank lines + paginator)
2. Calculate per_page from remaining height and item heights
3. Update paginator state if items/sizing changed
4. Get slice bounds from paginator
5. Render title (styles.title)
6. Render count "N items" (styles.count)
7. Render items — each gets height() rows, call item.render(area, buf, &context)
8. Render selection indicator on left edge of selected item
9. Render paginator at bottom

## Key Handling Pattern

List-first dispatch. User code handles this (not the widget):

```rust
fn handle_key(&mut self, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => self.state.select_prev(self.items.len(), false),
        KeyCode::Down | KeyCode::Char('j') => self.state.select_next(self.items.len(), false),
        KeyCode::Left | KeyCode::Char('h') => self.state.prev_page(),
        KeyCode::Right | KeyCode::Char('l') => self.state.next_page(),
        other => {
            self.items[self.state.selected()].handle_key(other);
        }
    }
}
```

## Not Included (future work)

- Filtering/search
- Status bar
- Spinner integration
- Help widget integration (exists separately)
