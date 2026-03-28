use crate::ids::ThingsId;
use crate::ui::style::dim;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Span,
    widgets::Widget,
};

pub fn id_col_width(id_prefix_len: usize) -> u16 {
    let id_width = id_prefix_len as u16;
    id_width + if id_width > 0 { 1 } else { 0 }
}

pub fn split_id_and_content(area: Rect, id_prefix_len: usize) -> (Rect, Rect) {
    let [id_col, content_col] = Layout::horizontal([
        Constraint::Length(id_col_width(id_prefix_len)),
        Constraint::Fill(1),
    ])
    .areas(area);
    (id_col, content_col)
}

pub fn render_id_prefix(id: &ThingsId, id_prefix_len: usize, id_col: Rect, buf: &mut Buffer) {
    if id_prefix_len == 0 {
        return;
    }
    let id_raw: String = id.to_string().chars().take(id_prefix_len).collect();
    Span::styled(id_raw, dim()).render(
        Rect {
            height: 1,
            ..id_col
        },
        buf,
    );
}
