pub mod fieldset;
pub mod help;
pub mod input;
pub mod list;
pub mod paginator;
pub mod select;
pub mod spinner;
pub mod tree;

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_cheese::theme::Palette;

/// A showcase component that demonstrates a library widget.
///
/// Each component owns its sample data, state, key handling, and drawing.
pub trait Component {
    /// Display name shown in the sidebar.
    fn name(&self) -> &str;

    /// Handle a key press. Called only when this component is selected.
    fn handle_key(&mut self, key: KeyCode);

    /// Draw the component into the given area.
    ///
    /// `focused` indicates whether the detail panel has focus. Components
    /// should use this to dim their border when not focused.
    fn draw(&mut self, frame: &mut Frame, palette: &Palette, area: Rect, focused: bool);

    /// Called every frame for animations. Default is no-op.
    fn tick(&mut self) {}
}
