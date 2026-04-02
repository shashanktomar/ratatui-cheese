use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Widget};
use ratatui_cheese::help::{Binding, Help, HelpStyles};
use ratatui_cheese::theme::Palette;

use crate::widgets::Component;
use crate::widgets::fieldset::FieldsetComponent;
use crate::widgets::help::HelpComponent;
use crate::widgets::input::InputComponent;
use crate::widgets::list::ListComponent;
use crate::widgets::paginator::PaginatorComponent;
use crate::widgets::spinner::SpinnerComponent;
use crate::widgets::tree::TreeComponent;

/// Which panel has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Sidebar,
    Detail,
}

pub struct App {
    components: Vec<Box<dyn Component>>,
    selected: usize,
    focus: Focus,
    show_help: bool,
    palette_index: usize,
    palettes: Vec<(&'static str, Palette)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            components: vec![
                Box::new(SpinnerComponent::new()),
                Box::new(HelpComponent::new()),
                Box::new(TreeComponent::new()),
                Box::new(PaginatorComponent::new()),
                Box::new(ListComponent::new()),
                Box::new(FieldsetComponent::new()),
                Box::new(InputComponent::new()),
            ],
            selected: 0,
            focus: Focus::Sidebar,
            show_help: false,
            palette_index: 0,
            palettes: Palette::presets(),
        }
    }

    /// Returns true if the app should quit.
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        // F1 toggles help globally
        if key == KeyCode::F(1) {
            self.show_help = !self.show_help;
            return false;
        }

        if self.show_help {
            self.show_help = false;
            return false;
        }

        // Tab/BackTab toggles focus between sidebar and detail
        if key == KeyCode::Tab || key == KeyCode::BackTab {
            self.focus = match self.focus {
                Focus::Sidebar => Focus::Detail,
                Focus::Detail => Focus::Sidebar,
            };
            return false;
        }

        // Esc: quit from sidebar, return to sidebar from detail
        if key == KeyCode::Esc {
            return match self.focus {
                Focus::Sidebar => true,
                Focus::Detail => {
                    self.focus = Focus::Sidebar;
                    false
                }
            };
        }

        match self.focus {
            Focus::Sidebar => self.handle_sidebar_key(key),
            Focus::Detail => self.components[self.selected].handle_key(key),
        }
        false
    }

    fn handle_sidebar_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                // handled via Esc path, but also allow q to quit from sidebar
            }
            KeyCode::Char('p') => self.next_palette(),
            KeyCode::Char('P') => self.prev_palette(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.select_prev(),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                self.focus = Focus::Detail;
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        for component in &mut self.components {
            component.tick();
        }

        let area = frame.area();
        let palette = self.palette().clone();

        let [sidebar_area, detail_area] =
            Layout::horizontal([Constraint::Length(26), Constraint::Fill(1)]).areas(area);

        if self.show_help {
            self.draw_help_overlay(frame, &palette, sidebar_area);
        } else {
            self.draw_sidebar(frame, &palette, sidebar_area);
        }

        self.draw_detail(frame, &palette, detail_area);
    }

    fn draw_sidebar(&self, frame: &mut Frame, palette: &Palette, area: Rect) {
        let border_style = if self.focus == Focus::Sidebar {
            Style::default().fg(palette.foreground)
        } else {
            Style::default().fg(palette.faint)
        };
        let block = Block::bordered()
            .title(format!(" {} ", self.palette_name()))
            .border_style(border_style)
            .padding(Padding::horizontal(1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        for (i, component) in self.components.iter().enumerate() {
            if i as u16 >= inner.height {
                break;
            }
            let style = if i == self.selected {
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.foreground)
            };
            let prefix = if i == self.selected { "▸ " } else { "  " };
            let line = Line::from(Span::styled(format!("{prefix}{}", component.name()), style));
            let line_area = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
            line.render(line_area, frame.buffer_mut());
        }

        let hint_y = inner.y + inner.height.saturating_sub(1);
        if hint_y > inner.y + self.components.len() as u16 {
            let hint = Help::default()
                .bindings(vec![Binding::new("F1", "help"), Binding::new("p", "theme")])
                .styles(HelpStyles::from_palette(palette));
            let hint_width = 20u16.min(inner.width);
            let hint_x = inner.x + (inner.width.saturating_sub(hint_width)) / 2;
            let hint_area = Rect::new(hint_x, hint_y, hint_width, 1);
            Widget::render(&hint, hint_area, frame.buffer_mut());
        }
    }

    fn draw_detail(&mut self, frame: &mut Frame, palette: &Palette, area: Rect) {
        let focused = self.focus == Focus::Detail;
        self.components[self.selected].draw(frame, palette, area, focused);
    }

    fn draw_help_overlay(&self, frame: &mut Frame, palette: &Palette, area: Rect) {
        let block = Block::bordered()
            .title(" Help ")
            .padding(Padding::new(1, 1, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let help = Help::default()
            .binding_groups(vec![vec![
                Binding::new("j/k", "select widget"),
                Binding::new("p/P", "cycle theme"),
                Binding::new("tab", "switch focus"),
                Binding::new("esc", "back / quit"),
                Binding::new("F1", "toggle help"),
            ]])
            .styles(HelpStyles::from_palette(palette))
            .show_all(true);

        let help_height = help.required_height().min(inner.height);
        let help_area = Rect::new(inner.x, inner.y, inner.width, help_height);
        Widget::render(&help, help_area, frame.buffer_mut());
    }

    fn palette(&self) -> &Palette {
        &self.palettes[self.palette_index].1
    }

    fn palette_name(&self) -> &str {
        self.palettes[self.palette_index].0
    }

    fn next_palette(&mut self) {
        self.palette_index = (self.palette_index + 1) % self.palettes.len();
    }

    fn prev_palette(&mut self) {
        self.palette_index = if self.palette_index == 0 {
            self.palettes.len() - 1
        } else {
            self.palette_index - 1
        };
    }

    fn select_next(&mut self) {
        self.selected = (self.selected + 1) % self.components.len();
    }

    fn select_prev(&mut self) {
        self.selected =
            if self.selected == 0 { self.components.len() - 1 } else { self.selected - 1 };
    }
}
