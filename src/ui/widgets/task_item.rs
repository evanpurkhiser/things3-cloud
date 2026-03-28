use crate::common::ICONS;
use crate::store::{Task, ThingsStore};
use crate::ui::style::dim;
use crate::ui::widgets::checklist::ChecklistWidget;
use crate::ui::widgets::id_col::{render_id_prefix, split_id_and_content};
use crate::ui::widgets::left_border::LeftBorderWidget;
use crate::ui::widgets::task_line::TaskLine;
use crate::ui::widgets::tasks::TaskOptions;
use chrono::{DateTime, Utc};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::Widget,
};

/// Full height of the detail block (the LeftBorderWidget area) for this task.
/// Returns 0 if there is nothing to show.
fn detail_height(task: &Task) -> u16 {
    let note_lines = task.notes.as_ref().map(|n| n.lines().count()).unwrap_or(0);
    let checklist = task.checklist_items.len();
    let spacer = if note_lines > 0 && checklist > 0 {
        1
    } else {
        0
    };
    (note_lines + spacer + checklist) as u16
}

/// Presentational widget for a single task (todo) row, optionally with notes
/// and checklist items shown beneath it.
///
/// Layout:
/// ```text
/// Layout::horizontal([Length(id_width), Fill(1)])
///   col 0: short id text (dim), only on row 0
///   col 1: Layout::vertical([Length(1), Length(detail_height)])
///     row 0: Layout::horizontal([Length(1), Length(1), Fill(1)])
///              [▢/◼/…][ ][LineItem: markers + title + tags + deadline]
///     row 1: LeftBorderWidget + content rendered into same area
///              notes at x=2, checklist rows at x=0 (overwriting border)
/// ```
pub struct TaskItemWidget<'a> {
    pub task: &'a Task,
    pub store: &'a ThingsStore,
    pub options: TaskOptions,
    /// Width of the shared ID column across all items in the list (0 = no IDs).
    pub id_prefix_len: usize,
    pub today: &'a DateTime<Utc>,
}

impl<'a> TaskItemWidget<'a> {
    fn checkbox_str(&self) -> &'static str {
        if self.task.is_completed() {
            ICONS.task_done
        } else if self.task.is_canceled() {
            ICONS.task_canceled
        } else if self.task.in_someday() {
            ICONS.task_someday
        } else {
            ICONS.task_open
        }
    }

    /// Render the first row: [checkbox][ ][markers + title + tags + deadline]
    fn render_task_row(&self, area: Rect, buf: &mut Buffer) {
        let [checkbox_col, content_col] = Layout::horizontal([
            Constraint::Length(1), // checkbox
            Constraint::Fill(1),   // content
        ])
        .spacing(1)
        .areas(area);

        // Checkbox
        Span::styled(self.checkbox_str(), dim()).render(checkbox_col, buf);

        let spans = TaskLine {
            task: self.task,
            store: self.store,
            today: self.today,
            show_today_markers: self.options.show_today_markers,
            show_staged_today_marker: self.options.show_staged_today_marker,
            show_tags: true,
            show_project: self.options.show_project,
            show_area: self.options.show_area,
        }
        .spans();

        Line::from(spans).render(content_col, buf);
    }

    /// Render the detail block (notes + checklist) into `area`.
    ///
    /// `area` is the detail region in the content column (below the title row).
    ///
    /// Layout per row:
    /// - Note rows: rendered inside `LeftBorderWidget` (container) at x+2
    /// - Empty spacer row: a blank `Line` inside `LeftBorderWidget`
    /// - Checklist rows: span the full width so IDs align with task IDs above
    fn render_detail_block(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let note_lines: Vec<&str> = self
            .task
            .notes
            .as_ref()
            .map(|n| n.lines().collect())
            .unwrap_or_default();

        let checklist = &self.task.checklist_items;
        let has_notes = !note_lines.is_empty();
        let has_checklist = !checklist.is_empty();

        // Build children for LeftBorderWidget: note lines + optional blank spacer.
        let mut border_children: Vec<Line> = note_lines
            .iter()
            .map(|note| Line::from(Span::styled(*note, dim())))
            .collect();

        if has_notes && has_checklist {
            border_children.push(Line::default());
        }

        let rail_height = if has_checklist && !border_children.is_empty() {
            border_children.len() as u16 + 1
        } else {
            border_children.len() as u16
        };

        if rail_height > 0 {
            LeftBorderWidget {
                children: border_children,
                total_height: rail_height,
            }
            .render(area, buf);
        }

        if has_checklist {
            let show_ids = self.id_prefix_len > 0;
            let notes_and_spacer_height =
                note_lines.len() as u16 + if has_notes && has_checklist { 1 } else { 0 };

            let cl_id_col = ChecklistWidget::id_col_width(checklist, show_ids);

            let cl_area = Rect {
                // Detail area lives in the content column. Shift checklist left
                // so its id column can extend into the task-id column when the
                // task-id width is larger than the checklist-id width.
                x: area.x.saturating_sub(cl_id_col),
                y: area.y + notes_and_spacer_height,
                width: area.width.saturating_add(cl_id_col),
                height: checklist.len() as u16,
            };
            ChecklistWidget {
                items: checklist,
                show_ids,
            }
            .render(cl_area, buf);
        }
    }

    /// Total height this widget needs.
    pub fn height(&self) -> u16 {
        let dh = if self.options.detailed {
            detail_height(self.task)
        } else {
            0
        };
        1 + dh
    }
}

impl<'a> Widget for TaskItemWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let detail_h = if self.options.detailed {
            detail_height(self.task)
        } else {
            0
        };

        // Top-level 2-column grid: [id_col | content_col]
        let (id_col, content_col) = split_id_and_content(area, self.id_prefix_len);

        // Render the short id in col 0, top row only.
        render_id_prefix(&self.task.uuid, self.id_prefix_len, id_col, buf);

        // Col 1: vertical split into task row + optional detail block.
        let col1 = content_col;

        if detail_h == 0 {
            // Simple case: just the task row.
            self.render_task_row(Rect { height: 1, ..col1 }, buf);
        } else {
            let [task_row, detail_row] =
                Layout::vertical([Constraint::Length(1), Constraint::Length(detail_h)]).areas(col1);

            self.render_task_row(task_row, buf);
            self.render_detail_block(detail_row, buf);
        }
    }
}
