use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui_cheese::paginator::{Paginator, PaginatorMode, PaginatorState, PaginatorStyles};
use ratatui_cheese::theme::Palette;

use super::Component;

const TOTAL_ITEMS: usize = 50;
const PER_PAGE: usize = 5;

pub struct PaginatorComponent {
    items: Vec<String>,
    state: PaginatorState,
    mode: PaginatorMode,
}

impl PaginatorComponent {
    pub fn new() -> Self {
        let items: Vec<String> = (1..=TOTAL_ITEMS).map(|i| format!("Item {i}")).collect();
        let state = PaginatorState::new(items.len(), PER_PAGE);
        Self {
            items,
            state,
            mode: PaginatorMode::Dots,
        }
    }
}

impl Component for PaginatorComponent {
    fn name(&self) -> &str {
        "Paginator"
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('l') | KeyCode::Right => self.state.next_page(),
            KeyCode::Char('h') | KeyCode::Left => self.state.prev_page(),
            KeyCode::Char('m') => {
                self.mode = match self.mode {
                    PaginatorMode::Dots => PaginatorMode::Arabic,
                    PaginatorMode::Arabic => PaginatorMode::Dots,
                };
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, palette: &Palette, area: Rect) {
        let mode_label = match self.mode {
            PaginatorMode::Dots => "Dots",
            PaginatorMode::Arabic => "Arabic",
        };

        let block = Block::bordered()
            .title(format!(" Paginator: {mode_label} "))
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let styles = PaginatorStyles::from_palette(palette);

        let [items_area, _, paginator_area, _, help_area] = Layout::vertical([
            Constraint::Length(PER_PAGE as u16 * 2), // items (text + blank line each)
            Constraint::Length(1),                   // gap
            Constraint::Length(1),                   // paginator
            Constraint::Length(1),                   // gap
            Constraint::Length(1),                   // help
        ])
        .areas(inner);

        // Items
        let (start, end) = self.state.get_slice_bounds(self.items.len());
        for (i, item) in self.items[start..end].iter().enumerate() {
            let y = items_area.y + (i as u16 * 2);
            if y >= items_area.y + items_area.height {
                break;
            }
            frame.buffer_mut().set_string(
                items_area.x,
                y,
                format!("• {item}"),
                Style::default().fg(palette.foreground),
            );
        }

        // Paginator
        let paginator = Paginator::default().mode(self.mode).styles(styles);
        StatefulWidget::render(
            &paginator,
            paginator_area,
            frame.buffer_mut(),
            &mut self.state,
        );

        // Help
        if help_area.height > 0 {
            let help = Line::from(Span::styled(
                "h/l, ←/→: page • m: toggle mode",
                Style::default().fg(palette.faint),
            ));
            help.render(help_area, frame.buffer_mut());
        }
    }
}
