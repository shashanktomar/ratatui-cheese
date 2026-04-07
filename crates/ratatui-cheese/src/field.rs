//! Shared types for form field widgets.
//!
//! Contains common building blocks used across form fields like
//! [`Input`](crate::input::Input), Select, Confirm, etc.

/// The kind of validation message to display on a form field.
///
/// Used by form field widgets to style their validation feedback.
/// `Error` typically renders in the palette's primary (accent) color,
/// while `Success` renders in green.
///
/// # Example
///
/// ```rust
/// use ratatui_cheese::field::ValidationKind;
///
/// let result: Result<(), String> = Err("Required field".into());
/// let kind = match result {
///     Ok(()) => ValidationKind::Success,
///     Err(_) => ValidationKind::Error,
/// };
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ValidationKind {
    /// Validation failed — the field has an error.
    Error,
    /// Validation passed — the field value is valid.
    Success,
}
