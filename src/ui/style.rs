use ratatui::style::{Modifier, Style};

pub fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}
