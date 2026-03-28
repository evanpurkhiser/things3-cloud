use crate::common::ICONS;
use crate::store::{Task, ThingsStore};
use crate::ui::style::dim;
use chrono::{DateTime, Utc};
use ratatui::{
    style::{Color, Style},
    text::Span,
};

pub struct TaskLine<'a> {
    pub task: &'a Task,
    pub store: &'a ThingsStore,
    pub today: &'a DateTime<Utc>,
    pub show_today_markers: bool,
    pub show_staged_today_marker: bool,
    pub show_tags: bool,
    pub show_project: bool,
    pub show_area: bool,
}

impl<'a> TaskLine<'a> {
    pub fn spans(&self) -> Vec<Span<'static>> {
        let mut spans: Vec<Span<'static>> = Vec::new();

        if self.show_today_markers {
            if self.task.evening {
                spans.push(Span::styled(ICONS.evening.to_string(), Color::Blue));
                spans.push(Span::raw(" ".to_string()));
            } else if self.task.is_today(self.today) {
                spans.push(Span::styled(ICONS.today.to_string(), Color::Yellow));
                spans.push(Span::raw(" ".to_string()));
            }
        } else if self.show_staged_today_marker && self.task.is_staged_for_today(self.today) {
            spans.push(Span::styled(ICONS.today_staged.to_string(), Color::Yellow));
            spans.push(Span::raw(" ".to_string()));
        }

        if self.task.title.is_empty() {
            spans.push(Span::styled("(untitled)".to_string(), dim()));
        } else {
            spans.push(Span::raw(self.task.title.clone()));
        }

        if self.show_tags && !self.task.tags.is_empty() {
            let tag_names: Vec<String> = self
                .task
                .tags
                .iter()
                .map(|t| self.store.resolve_tag_title(t))
                .collect();
            spans.push(Span::styled(
                format!(" [{}]", tag_names.join(", ")),
                dim(),
            ));
        }

        if self.show_project {
            if let Some(proj) = self.store.effective_project_uuid(self.task) {
                let title = self.store.resolve_project_title(&proj);
                spans.push(Span::styled(
                    format!(" {} {}", ICONS.separator, title),
                    dim(),
                ));
            } else if self.show_area
                && let Some(area) = self.store.effective_area_uuid(self.task)
            {
                let title = self.store.resolve_area_title(&area);
                spans.push(Span::styled(
                    format!(" {} {}", ICONS.separator, title),
                    dim(),
                ));
            }
        } else if self.show_area
            && let Some(area) = self.store.effective_area_uuid(self.task)
        {
            let title = self.store.resolve_area_title(&area);
            spans.push(Span::styled(
                format!(" {} {}", ICONS.separator, title),
                dim(),
            ));
        }

        if let Some(deadline) = self.task.deadline {
            let date_str = deadline.format("%Y-%m-%d").to_string();
            let dl_style = if deadline < *self.today {
                Style::from(Color::Red)
            } else {
                Style::from(Color::Yellow)
            };
            spans.push(Span::raw(format!(" {} due by ", ICONS.deadline)));
            spans.push(Span::styled(date_str, dl_style));
        }

        spans
    }
}
