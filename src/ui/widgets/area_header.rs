use crate::common::ICONS;
use crate::ids::ThingsId;
use crate::ui::widgets::id_col::{render_id_prefix, split_id_and_content};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

pub struct AreaHeaderWidget<'a> {
    pub area_uuid: &'a ThingsId,
    pub title: &'a str,
    pub id_prefix_len: usize,
}

impl<'a> AreaHeaderWidget<'a> {
    fn bold() -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

impl<'a> Widget for AreaHeaderWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let (id_col, content_col) = split_id_and_content(area, self.id_prefix_len);
        render_id_prefix(self.area_uuid, self.id_prefix_len, id_col, buf);

        Line::from(vec![
            Span::raw(format!("{} ", ICONS.area)),
            Span::styled(self.title, Self::bold()),
        ])
        .render(
            Rect {
                height: 1,
                ..content_col
            },
            buf,
        );
    }
}
