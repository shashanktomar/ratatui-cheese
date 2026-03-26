use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Padding, Widget};
use ratatui_cheese::help::{Binding, Help, HelpStyles};
use ratatui_cheese::theme::Palette;

use super::Component;

pub struct HelpComponent {
    show_all: bool,
}

impl HelpComponent {
    pub fn new() -> Self {
        Self { show_all: false }
    }
}

impl Component for HelpComponent {
    fn name(&self) -> &str {
        "Help"
    }

    fn handle_key(&mut self, key: KeyCode) {
        if let KeyCode::Char('?') = key {
            self.show_all = !self.show_all;
        }
    }

    fn draw(&mut self, frame: &mut Frame, palette: &Palette, area: Rect) {
        let title = if self.show_all { " Help: Full " } else { " Help: Short " };
        let block = Block::bordered()
            .title(title)
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let help = Help::default()
            .bindings(short_bindings())
            .binding_groups(full_bindings())
            .styles(HelpStyles::from_palette(palette))
            .show_all(self.show_all);
        let help_height = help.required_height();
        let help_area = Rect::new(inner.x, inner.y, inner.width, help_height.min(inner.height));
        Widget::render(&help, help_area, frame.buffer_mut());
    }
}

fn short_bindings() -> Vec<Binding> {
    vec![Binding::new("?", "toggle full"), Binding::new("q", "quit")]
}

fn full_bindings() -> Vec<Vec<Binding>> {
    vec![
        vec![
            Binding::new("↑/k", "move up"),
            Binding::new("↓/j", "move down"),
            Binding::new("←/h", "move left"),
            Binding::new("→/l", "move right"),
        ],
        vec![Binding::new("?", "toggle"), Binding::new("q", "quit")],
    ]
}
