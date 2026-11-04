//! The shared TUI palette, matching watn's indexed colors.

use ratatui::style::{Color, Modifier, Style};

/// Cyan-blue labels and key hints.
pub const ACCENT: Color = Color::Indexed(81);
/// Primary text.
pub const BRIGHT: Color = Color::Indexed(97);
/// Description and activity body text.
pub const BODY: Color = Color::Indexed(252);
/// Amber selection and attention text.
pub const GOLD: Color = Color::Indexed(214);
/// Success and Markdown headings.
pub const GREEN: Color = Color::Indexed(114);
/// Rules and dividers.
pub const RULE: Color = Color::Indexed(240);

pub fn accent() -> Style {
    Style::new().fg(ACCENT)
}

pub fn accent_bold() -> Style {
    accent().add_modifier(Modifier::BOLD)
}

pub fn key() -> Style {
    accent_bold()
}

pub fn bright() -> Style {
    Style::new().fg(BRIGHT)
}

pub fn body() -> Style {
    Style::new().fg(BODY)
}

pub fn dim() -> Style {
    Style::new().add_modifier(Modifier::DIM)
}

pub fn border() -> Style {
    dim()
}

pub fn gold() -> Style {
    Style::new().fg(GOLD)
}

pub fn gold_bold() -> Style {
    gold().add_modifier(Modifier::BOLD)
}

pub fn green() -> Style {
    Style::new().fg(GREEN)
}

pub fn green_bold() -> Style {
    green().add_modifier(Modifier::BOLD)
}

pub fn rule() -> Style {
    Style::new().fg(RULE)
}
